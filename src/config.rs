use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

pub struct Config {
    pub(crate) values: Values,
    file: File,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Values {
    pub exp: u32,
}

impl Default for Config {
    fn default() -> Self {
        let path = dirs::home_dir().unwrap().join(".tors/config.json");

        let (config_file, file) = match File::open(&path) {
            Ok(mut file) => {
                let config_file = Config::read_config(&mut file).unwrap();

                (config_file, file)
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => (Values::default(), Config::new_config(path).unwrap()),
                _ => panic!("{}", e.to_string()),
            },
        };

        Self {
            file,
            values: config_file,
        }
    }
}

impl Config {
    pub fn add_exp(&mut self, exp: u32) -> Result<()> {
        self.values.exp += exp;

        self.update_config()
    }

    fn new_config(path: PathBuf) -> Result<File> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();

        let data = serde_json::to_string(&Values::default())?;

        file.write_all(data.as_bytes())?;

        Ok(file)
    }

    fn read_config(file: &mut File) -> Result<Values> {
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        Ok(serde_json::from_str(&data)?)
    }

    fn update_config(&mut self) -> Result<()> {
        let data = serde_json::to_string(&self.values)?;

        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(data.as_bytes())?;

        Ok(())
    }
}
