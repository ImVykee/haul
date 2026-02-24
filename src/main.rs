use clap::{Parser, Subcommand};
mod config_parser;
use directories::ProjectDirs;
use rusqlite::Connection;
mod database;

const DEFAULT_CONFIG: &str = include_str!("../config");

#[derive(Parser)]
#[command(name = "todo")]
#[command(about = "A simple CLI todo list manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        task: String,
        list: String,
        date: Option<String>,
    },
    List {
        list: String,
    },
    Remove {
        #[arg(short = 'l', long = "list")]
        list: Option<String>,
        #[arg(short = 't', long = "todo")]
        id: Option<i64>,
    },
    Done {
        id: i64,
    },
    Listall,
    Reinstall,
    Configpath,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup(false)?;
    let statement = Cli::parse();
    match statement.command {
        Commands::Add { task, list, date } => match database::create_todo(&list, &task, date) {
            Ok(msg) => println!("{}", msg),
            Err(error) => handle_error(error),
        },
        Commands::List { list } => match database::display(&list) {
            Ok(result) => println!("{}", result),
            Err(error) => handle_error(error),
        },
        Commands::Remove { list, id } => match (list, id) {
            (Some(list), None) => match database::clear_list(&list) {
                Ok(_) => println!("List \"{}\" removed", list),
                Err(error) => handle_error(error),
            },
            (None, Some(id)) => match database::clear_todo(id) {
                Ok(task) => println!("Task \"{}\" removed", task),
                Err(error) => handle_error(error),
            },
            _ => eprintln!("Provide either -l or -t"),
        },
        Commands::Done { id } => match database::check_todo(id) {
            Ok(task) => println!("Checked task \"{}\"", task),
            Err(error) => handle_error(error),
        },
        Commands::Listall => match database::display_all() {
            Ok(result) => println!("{}", result),
            Err(error) => handle_error(error),
        },
        Commands::Reinstall => match setup(true) {
            Ok(_) => println!("Reinstalled database"),
            Err(error) => handle_error(error),
        },
        Commands::Configpath => match config_parser::config_path() {
            Ok(path) => println!("{}", path.to_str().ok_or("config not found")?),
            Err(error) => handle_error(error),
        },
    }
    Ok(())
}

fn handle_error(error: Box<dyn std::error::Error + 'static>) {
    eprintln!("Error : {}", error)
}

fn setup(re: bool) -> Result<(), Box<dyn std::error::Error>> {
    let dirs = ProjectDirs::from("", "", "haul").unwrap();
    std::fs::create_dir_all(dirs.data_dir())?;
    std::fs::create_dir_all(dirs.config_dir())?;
    if !dirs.config_dir().join("config").exists() {
        std::fs::write(dirs.config_dir().join("config"), DEFAULT_CONFIG)?;
    }
    let conn = Connection::open(dirs.data_dir().join("todo"))?;
    if re {
        conn.execute("DROP TABLE IF EXISTS Todo", [])?;
    }
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS Todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            done INTEGER NOT NULL DEFAULT 0,
            list TEXT NOT NULL,
            date TEXT
        );
    ",
    )?;
    Ok(())
}
