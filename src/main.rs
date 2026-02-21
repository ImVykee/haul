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
    }, // filter by done/undone tasks
    Remove {
        #[arg(short = 'l', long = "list")]
        list: Option<String>,
        #[arg(short = 't', long = "todo")]
        id: Option<i64>,
    },
    Done {
        id: i64,
    },
    Create {
        list_name: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let statement = Cli::parse();
    match statement.command {
        Commands::Add { task, list } => database::create_todo(&list, &task)?,
        Commands::List { list } => database::display(&list)?,
        Commands::Remove { list, id } => match (list, id) {
            (Some(list), None) => database::delete_list(&list)?,
            (None, Some(id)) => database::clear_todo(id)?,
            _ => eprintln!("provide either -l or -t"),
        },
        Commands::Done { id } => database::check_todo(id)?,
        Commands::Create { list_name } => database::create_list(&list_name)?,
    }
    Ok(())
}
