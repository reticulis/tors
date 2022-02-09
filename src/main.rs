use clap::{Parser, Subcommand};
use colored::*;
use json::{object, JsonValue};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};

const NOT_FOUND: &str = "Not found tasks!";
const NOT_FOUND_T: &str = "Not found this task!";
const ADDED: &str = "Task added!";
const REMOVED: &str = "Task removed!";

#[derive(Parser)]
#[clap(version = "0.0.3-alpha")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add new task
    New { description: String },
    /// List tasks
    Ls,
    /// Delete task
    Rm { id: usize },
    /// Select a completed task
    Dn { id: usize },
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
        match fs::create_dir(&self.folder_path) {
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
            Commands::New { description: task } => self.create_task(task),
            Commands::Ls => self.list_tasks(),
            Commands::Rm { id } => self.remove_task(id.clone()),
            Commands::Dn { id } => self.completed_task(id.clone()),
        };
    }

    fn create_task(&self, task: &String) {
        let mut buf = BufWriter::new(self.task.get_file());
        buf.write_all(
            (object! {
                success: false,
                description: task.to_owned()
            }
            .to_string()
                + "\n")
                .as_bytes(),
        )
        .expect("Write failed!");
        println!("{}", ADDED.green());
    }

    fn list_tasks(&self) {
        fn color(b: bool) -> [ColoredString; 3] {
            match b {
                true => ["Task".blue(), "Status".blue(), "Description".blue()],
                false => ["Task".green(), "Status".green(), "Description".green()],
            }
        }

        fn show(color: [ColoredString; 3], json: JsonValue, i: usize) {
            println!(
                "{}: {}\n{}: {}\n{}: {}\n",
                color[0],
                i + 1,
                color[1],
                json["success"],
                color[2],
                json["description"]
            )
        }

        let mut buf = BufReader::new(self.task.get_file());
        let mut string = String::new();
        buf.read_to_string(&mut string).unwrap();

        match string.len() == 0 {
            true => println!("{}", NOT_FOUND.red()),
            false => {
                for (i, json) in string.lines().enumerate() {
                    let j = json::parse(json).expect("Failed parse JSON from file!");
                    match j["success"] == true {
                        true => show(color(true), j, i),
                        false => show(color(false), j, i),
                    }
                }
            }
        }
    }
    fn remove_task(&self, id: usize) {
        match self.get_task(id) {
            Ok(mut v) => match v.get(id - 1) {
                Some(_) => {
                    v.remove(id - 1);
                    fs::write(&self.task.file_path, v.join("\n").as_bytes())
                        .expect("Failed delete task");
                    println!("{}", REMOVED.green());
                }
                None => println!("{}", NOT_FOUND_T.red()),
            },
            Err(e) => println!("{}", e.red()),
        }
    }
    fn completed_task(&self, id: usize) {
        match self.get_task(id) {
            Ok(mut v) => match v.get(id - 1) {
                Some(_) => {
                    let mut j = json::parse(v[id - 1].as_str()).unwrap();
                    j["success"] = true.into();
                    v[id - 1] = j.to_string();
                    fs::write(&self.task.file_path, v.join("\n").as_bytes())
                        .expect("Failed delete task")
                }
                None => println!("{}", NOT_FOUND_T.red()),
            },
            Err(e) => println!("{}", e.red()),
        }
    }
    fn get_task(&self, id: usize) -> Result<Vec<String>, &str> {
        match id == 0 {
            true => Err(NOT_FOUND_T),
            false => Ok({
                let mut buf_reader = BufReader::new(self.task.get_file());

                let mut old_file = String::new();

                buf_reader.read_to_string(&mut old_file).unwrap();

                old_file.lines().map(str::to_owned).collect::<Vec<String>>()
            }),
        }
    }
}

fn main() {
    let home_path = match dirs::home_dir() {
        Some(d) => d,
        None => panic!("Failed to get path user home directory"),
    };

    let cli = Cli::parse();
    let file = TaskFile::new(home_path.to_str().unwrap());

    let app = App::new(cli, file);
    app.command()
}
