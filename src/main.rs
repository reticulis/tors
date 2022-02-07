use clap::{Parser, Subcommand};
use json::object;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New { task: Option<String> },
    List,
    Del { id: usize },
}

struct TaskFile {
    folder_path: String,
    file_path: String,
}

impl TaskFile {
    fn new(path: &str) -> TaskFile {
        TaskFile {
            folder_path: format!("{}/.tors", path),
            file_path: format!("{}/.tors/tasks.json", path),
        }
    }

    fn get_file(&self) -> File {
        match OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&self.file_path)
        {
            Ok(f) => f,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => self.create_dir(),
                other => panic!("Problem opening the file: {:?}", other),
            },
        }
    }

    fn create_dir(&self) -> File {
        match fs::create_dir_all(&self.folder_path) {
            Ok(()) => self.get_file(),
            Err(e) => match e.kind() {
                ErrorKind::AlreadyExists => self.get_file(),
                other => panic!("Problem creating the folder: {:?}", other),
            },
        }
    }
}

struct App {
    cli: Cli,
    task: TaskFile,
}

impl App {
    fn new(cli: Cli, task: TaskFile) -> App {
        App { cli, task }
    }

    fn command(&self) {
        match &self.cli.command {
            Commands::New { task } => self.create_task(task),
            Commands::List => self.list_tasks(),
            Commands::Del { id } => self.remove_task(id.clone()),
        };
    }

    fn create_task(&self, task: &Option<String>) {
        match task {
            Some(t) => {
                let mut buf = BufWriter::new(self.task.get_file());
                buf.write_all(
                    (object! {
                        success: false,
                        description: t.to_owned()
                    }
                    .to_string()
                        + "\n")
                        .as_bytes(),
                )
                .expect("Write failed!");
                println!("Task added!");
            }
            None => println!("Please enter a description of the task"),
        }
    }

    fn list_tasks(&self) {
        let mut buf = BufReader::new(self.task.get_file());
        let mut string = String::new();
        buf.read_to_string(&mut string).unwrap();

        match string.len() == 0 {
            true => println!("Not found tasks!"),
            false => {
                for (i, json) in string.lines().enumerate() {
                    let j = json::parse(json).unwrap();
                    println!(
                        "\
                        Task: {}\n\
                        Status: {}\n\
                        Description: {}\n",
                        i + 1,
                        j["success"],
                        j["description"]
                    );
                }
            }
        }
    }
    fn remove_task(&self, id: usize) {
        match id == 0 {
            true => {
                println!("Not found this task");
            }
            false => {
                let mut buf_reader = BufReader::new(self.task.get_file());

                let mut old_file = String::new();

                buf_reader.read_to_string(&mut old_file).unwrap();

                let mut new_file = old_file.lines().collect::<Vec<&str>>();

                match new_file.get(id - 1) {
                    Some(_) => {
                        new_file.remove(id - 1);
                        fs::write(&self.task.file_path, new_file.join("\n").as_bytes())
                            .expect("Failed delete task")
                    }
                    None => println!("Not found this task"),
                }
            }
        }
    }
}

fn main() {
    let home_path = match dirs::home_dir() {
        Some(d) => d,
        None => panic!("Failed to get path user home directory")
    };

    let cli = Cli::parse();
    let file = TaskFile::new(home_path.to_str().unwrap());

    let app = App::new(cli, file);
    app.command()
}
