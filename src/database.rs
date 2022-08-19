use anyhow::Result;
use bincode::config::Configuration;
use nanoid::nanoid;
use serde::Serialize;
use sled::{Db, Iter};

pub struct Database {
    pub(crate) database: Db,
    pub(crate) config: Configuration,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            database: sled::open(dirs::home_dir().unwrap().join(".tors/database")).unwrap(),
            config: bincode::config::standard(),
        }
    }
}

impl Database {
    pub(crate) fn add<T: Serialize>(&self, value: &T) -> Result<()> {
        let nanoid = nanoid!();

        self.insert(nanoid, value)?;

        Ok(())
    }

    pub(crate) fn insert<K: AsRef<[u8]>, T: Serialize>(&self, key: K, value: &T) -> Result<()> {
        self.database
            .insert(key, bincode::serde::encode_to_vec(value, self.config)?)?;

        Ok(())
    }

    pub(crate) fn remove<K: AsRef<[u8]>>(&self, key: K) -> Result<()> {
        self.database.remove(key)?;

        Ok(())
    }

    // pub(crate) fn contains<K: AsRef<[u8]>>(&mut self, key: K) -> bool {
    //     self.database.contains_key(key).unwrap_or(false)
    // }

    pub(crate) fn iter(&self) -> Iter {
        self.database.iter()
    }
}
