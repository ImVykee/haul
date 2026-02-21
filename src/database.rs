use rusqlite::Connection;

struct Task {
    id: i64,
    name: String,
    done: bool,
}

pub fn create_list(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("src/todo")?;
    match conn.execute("INSERT INTO List(name) VALUES (?)", [name]) {
        Ok(_) => println!("List {} created", name),
        Err(err) if is_unique_violation(&err) => eprintln!("List {} already exists", name),
        Err(err) => eprintln!("error: {}", err),
    }
    Ok(())
}

pub fn create_todo(list: &str, task: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("src/todo")?;
    let id_list = conn.query_row("SELECT id_list FROM List WHERE name = ?", [list], |row| {
        row.get::<_, i64>(0)
    })?;
    match conn.execute(
        "INSERT INTO Todo(title, done, id_list) VALUES (?,?,?)",
        (task, 0, id_list),
    ) {
        Ok(0) => eprintln!("List {} not found", list),
        Ok(_) => println!("Task {} added to list {}", task, list),
        Err(err) => eprintln!("error : {}", err),
    };
    Ok(())
}

pub fn check_todo(id: i64) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("src/todo")?;
    conn.execute("UPDATE Todo SET done = 1 WHERE id = ?", [id])?;
    Ok(())
}

pub fn clear_todo(id: i64) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("src/todo")?;
    conn.execute("DELETE FROM Todo WHERE id = ?", [id])?;
    Ok(())
}

pub fn clear_list(list: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("src/todo")?;
    conn.execute(
        "DELETE FROM Todo WHERE id_list = (SELECT id_list FROM List WHERE name = ?)",
        [list],
    )?;
    Ok(())
}

pub fn delete_list(list: &str) -> Result<(), Box<dyn std::error::Error>> {
    clear_list(list)?;
    let conn = Connection::open("src/todo")?;
    conn.execute("DELETE FROM List WHERE name = ?", [list])?;
    Ok(())
}

pub fn display(list: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("src/todo")?;
    let mut stmt = conn
        .prepare("SELECT * FROM Todo WHERE id_list = (SELECT id_list FROM List WHERE name = ?)")?;
    println!("\"{}\" list :", list);
    let tasks = stmt.query_map([list], |row| {
        Ok(Task {
            id: row.get(0)?,
            name: row.get(1)?,
            done: row.get(2)?,
        })
    })?;
    for task in tasks {
        let task = task?;
        println!(
            "| {} - {} [{}]",
            task.id,
            task.name,
            if task.done { "x" } else { "o" }
        );
    }
    Ok(())
}

fn is_unique_violation(err: &rusqlite::Error) -> bool {
    matches!(err, rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation)
}
