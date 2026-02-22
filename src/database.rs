use rusqlite::Connection;

struct Task {
    id: i64,
    name: String,
    done: bool,
    list: String,
}

pub fn create_todo(list: &str, task: &str) -> Result<String, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path())?;
    match conn.execute(
        "INSERT INTO Todo(name, done, list) VALUES (?,?,?)",
        (task, 0, list),
    ) {
        Ok(0) => Err(format!("list {} not found", list).into()),
        Ok(_) => Ok(format!("Task {} added to list {}", task, list)),
        Err(err) => Err(err.into()),
    }
}

pub fn check_todo(id: i64) -> Result<String, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path())?;
    match conn.execute("UPDATE Todo SET done = 1 WHERE id = ?", [id]) {
        Ok(0) => Err(format!("no such task with id {}", id).into()),
        Ok(_) => {
            let mut tasks = select_query("id", &format!("{}", id))?;
            Ok(tasks.remove(0).name)
        }
        Err(error) => Err(error.into()),
    }
}

pub fn clear_todo(id: i64) -> Result<String, Box<dyn std::error::Error>> {
    let mut tasks = select_query("id", &format!("{}", id))?;
    let name = tasks.remove(0).name;
    let conn = Connection::open(db_path())?;
    match conn.execute("DELETE FROM Todo WHERE id = ?", [id]) {
        Ok(0) => Err(format!("no such task with id {}", id).into()),
        Ok(_) => Ok(name),
        Err(error) => Err(error.into()),
    }
}

pub fn clear_list(list: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path())?;
    match conn.execute("DELETE FROM Todo WHERE list = ?", [list]) {
        Ok(0) => Err(format!("no such list named \"{}\"", list).into()),
        Ok(_) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn display(list: &str) -> Result<String, Box<dyn std::error::Error>> {
    let tasks = select_query("list", list)?;
    let mut result = format!("\"{}\" list : \n", list);
    let mut local_id = 0;
    for task in tasks {
        local_id += 1;
        result += &format!(
            "  | {} - [{}] [id : {}] {}  \n",
            local_id,
            if task.done { "x" } else { "o" },
            task.id,
            task.name
        );
    }
    Ok(result)
}

pub fn display_all() -> Result<String, Box<dyn std::error::Error>> {
    let mut unique_lists: Vec<String> = Vec::new();
    let mut result = String::new();
    let tasks = select_all()?;
    if tasks.is_empty() {
        return Err("no task available".into());
    };
    for task in tasks {
        if !unique_lists.contains(&task.list) {
            unique_lists.push(task.list);
        }
    }
    for list in unique_lists {
        result += &display(&list)?;
    }
    Ok(result)
}

// fn is_unique_violation(err: &rusqlite::Error) -> bool {
//     matches!(err, rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation)
// }

fn select_query(by: &str, elem: &str) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path())?;
    let mut stmt = conn.prepare(&format!("SELECT * FROM Todo WHERE {} = ?", by))?;
    let tasks = stmt.query_map([elem], |row| {
        Ok(Task {
            id: row.get(0)?,
            name: row.get(1)?,
            done: row.get(2)?,
            list: row.get(3)?,
        })
    })?;
    let mut result: Vec<Task> = Vec::new();
    for task in tasks {
        let task = task?;
        result.push(task);
    }
    if result.is_empty() {
        return Err(format!("no such task where {} = {}", by, elem).into());
    };
    Ok(result)
}

fn select_all() -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path())?;
    let mut stmt = conn.prepare("SELECT * FROM Todo")?;
    let tasks = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            name: row.get(1)?,
            done: row.get(2)?,
            list: row.get(3)?,
        })
    })?;
    let mut result: Vec<Task> = Vec::new();
    for task in tasks {
        let task = task?;
        result.push(task);
    }
    if result.is_empty() {
        return Err("no task available".into());
    }
    Ok(result)
}

pub fn setup(re: bool) -> Result<(), Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    let dir = format!("{}/.local/share/haul", home);
    std::fs::create_dir_all(&dir)?;
    let conn = Connection::open(format!("{}/todo", dir))?;
    if re {
        conn.execute("DROP TABLE IF EXISTS Todo", [])?;
    }
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS Todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            done INTEGER NOT NULL DEFAULT 0,
            list TEXT NOT NULL
        );
    ",
    )?;
    Ok(())
}

fn db_path() -> String {
    let home = std::env::var("HOME").unwrap_or(".".to_string());
    format!("{}/.local/share/haul/todo", home)
}
