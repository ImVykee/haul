use clap::{Parser, Subcommand};
mod database;

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    database::setup(false)?;
    let statement = Cli::parse();
    match statement.command {
        Commands::Add { task, list } => match database::create_todo(&list, &task) {
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
        Commands::Reinstall => match database::setup(true) {
            Ok(_) => (),
            Err(error) => handle_error(error),
        },
    }
    Ok(())
}

fn handle_error(error: Box<dyn std::error::Error + 'static>) {
    eprintln!("Error : {}", error)
}
