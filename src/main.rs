use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Write};
use clap::{Parser, Subcommand};
use json::object;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New { task: Option<String> },
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
        match OpenOptions::new().read(true).append(true).open(&self.file) {
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
            Ok(_) => self.get_file(),
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

    let task_file = TaskFile::new(home.to_str().unwrap());
    let file = task_file.get_file();

    match cli.command {
        Commands::New { task } => {
            match task {
                Some(task) => {
                    let mut buf = BufWriter::new(file);
                    buf
                        .write_all((
                            object! {
                                success: false,
                                description: task
                            }.to_string() + "\n")
                            .as_bytes()
                        )
                        .expect("Write failed!");
                },
                None => println!("Please enter a description of the task")
            }
        },
        Commands::List => {
            let buf = BufReader::new(file);
            for (i, json) in buf.lines().enumerate() {
                match json {
                    Ok(json) => {
                        let j = json::parse(&*json).unwrap();
                        println!("\
                        Task: {}\n\
                        Status: {}\n\
                        Description: {}\n",
                                 i+1, j["success"], j["description"]);
                    },
                    Err(e) => panic!("Problem reading the file {}", e)
                }
            }
        }
    };
}
