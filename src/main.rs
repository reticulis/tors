use std::fs;
use std::fs::File;
use std::io::{BufReader, ErrorKind, Read};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New { tasks: Option<String> },
    List
}

struct TaskFile {
    folder: String,
    file: String
}

impl TaskFile {
    fn new(path: &str) -> TaskFile {
        TaskFile {
            folder: format!("{}/.tors", path),
            file: format!("{}/.tors/tasks.json", path)
        }
    }
    fn get_file(&self) -> File {
        match File::open(&self.file) {
            Ok(f) => f,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => self.create_dir(),
                other => panic!("Problem opening the file: {:?}", other)
            }
        }
    }

    fn create_dir(&self) -> File {
        match fs::create_dir_all(&self.folder) {
            Ok(_) => self.create_tasks_file(),
            Err(e) => match e.kind() {
                ErrorKind::AlreadyExists => self.create_tasks_file(),
                other => panic!("Problem creating the folder: {:?}", other)
            }
        }
    }

    fn create_tasks_file(&self) -> File {
        match File::create(&self.file) {
            Ok(f) => f,
            Err(e) => panic!("Problem creating the file: {:?}", e)
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let home = match dirs::home_dir() {
        Some(d) => d,
        None => panic!("Failed to get path user home directory")
    };

    let file = TaskFile::new(home.to_str().unwrap());

    let mut buf = BufReader::new(file.get_file());
    let mut string = String::new();
    buf.read_to_string(&mut string).unwrap();

    match cli.command {
        Commands::New { tasks } => {
            println!("'myapp add' was used, name is: {:?}", tasks)
        },
        Commands::List => {
            println!("siema")
        }
    };
    println!("Hello, world!");
}
