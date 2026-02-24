use crate::config_parser::{Config, parse_config};
use chrono::{Local, NaiveDate};
use colored::*;
use directories::ProjectDirs;
use rusqlite::Connection;

struct Task {
    id: i64,
    name: String,
    done: bool,
    list: String,
    date: Option<String>,
}

enum Status {
    Late,
    Yesterday,
    Today,
    Tomorrow,
    Future,
}

pub fn create_todo(
    list: &str,
    task: &str,
    date: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path())?;
    let date_format = parse_config()?.date.format;
    if let Some(date) = &date
        && NaiveDate::parse_from_str(date, &date_format).is_err()
    {
        return Err("invalid date format".into());
    }
    let formated_date = if let Some(date) = &date {
        Some(standardize_date_format(date, &date_format)?)
    } else {
        None
    };
    match conn.execute(
        "INSERT INTO Todo(name, done, list, date) VALUES (?,?,?,?)",
        (task, 0, list, formated_date),
    ) {
        Ok(0) => Err(format!("list {} not found", list).into()),
        Ok(_) => Ok(format!("Task {} added to list {}", task, list)),
        Err(err) => Err(err.into()),
    }
}

fn standardize_date_format(
    date: &str,
    current_format: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let parsed = NaiveDate::parse_from_str(date, current_format)?;
    Ok(parsed.format("%d/%m/%Y").to_string())
}

fn display_date(date: &str, wanted_format: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parsed = NaiveDate::parse_from_str(date, "%d/%m/%Y")?;
    Ok(parsed.format(wanted_format).to_string())
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
    let config = parse_config()?;
    let tasks = select_query("list", list)?;
    let mut result = match config.ui.compact {
        false => format!("  ┌──────────●[{}]\n", list.bold()),
        true => format!("  ┌─[{}]\n", list.bold()),
    };
    let mut local_id = 0;
    for task in &tasks {
        local_id += 1;
        let formatted = match config.ui.compact {
            true => format_compact(&config, task, local_id, tasks.len())?,
            false => format_task(&config, task, local_id, tasks.len())?,
        };
        result += &formatted;
    }
    Ok(result)
}

fn format_task(
    config: &Config,
    task: &Task,
    local_id: usize,
    tasks_len: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let date = match task.date {
        Some(ref d) => format_date(config, d)?,
        None => String::new(),
    };
    let done = task.done;
    let name = &task.name;
    let id = task.id;
    let closing = "  └──────────●\n";
    Ok(format!(
        "  │ {} ─ [{}] {} \n  │         └─[id: {}] {} \n{}\n",
        local_id,
        if done { "x" } else { " " },
        name,
        id,
        date,
        if local_id == tasks_len {
            closing
        } else {
            "  │"
        },
    ))
}

fn format_compact(
    config: &Config,
    task: &Task,
    local_id: usize,
    tasks_len: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let date = match task.date {
        Some(ref d) => format!(" {}", format_date(config, d)?),
        None => String::new(),
    };
    let done = task.done;
    let name = &task.name;
    let id = task.id;
    let closing = "└";
    Ok(format!(
        "  {} [{}{}] ─ [{}] {}\n{}",
        if local_id == tasks_len {
            closing
        } else {
            "│"
        },
        id,
        date,
        if done { "x" } else { " " },
        name,
        if local_id == tasks_len { "\n" } else { "" }
    ))
}

fn format_date(config: &Config, date: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("{:?}", config.date.format);
    println!("{:?}", date);
    let d = display_date(date, &config.date.format)?;
    let status = check_date(&d)?;
    if config.ui.colors {
        let color_date = match status {
            Status::Late => d.red(),
            Status::Yesterday => d.yellow(),
            Status::Today => d.green(),
            Status::Tomorrow => d.cyan(),
            Status::Future => d.normal(),
        };
        Ok(format!("| due: {}", color_date))
    } else {
        Ok(format!("| due: {}", d))
    }
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

fn check_date(date: &str) -> Result<Status, Box<dyn std::error::Error>> {
    let tempdate = date;
    let config = parse_config()?;
    let date_format = config.date.format;
    let task_date = NaiveDate::parse_from_str(tempdate, &date_format)?;
    let today = Local::now().date_naive();
    let diff = task_date.signed_duration_since(today).num_days() as i32;
    let status = match diff {
        n if n < -1 => Status::Late,
        -1 => Status::Yesterday,
        0 => Status::Today,
        n if n <= config.date.warn_days_before => Status::Tomorrow,
        _ => Status::Future,
    };
    Ok(status)
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
            date: row.get(4)?,
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
            date: row.get(4)?,
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

fn db_path() -> String {
    let dirs = ProjectDirs::from("", "", "haul").unwrap();
    dirs.data_dir().to_str().unwrap().to_string() + "/todo"
}
