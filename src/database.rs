use anyhow::{Context, Result};
use bincode::config::Configuration;
use nanoid::nanoid;
use serde::Serialize;
use sled::{Db, Iter, Tree};

pub struct Database {
    pub(crate) database: Db,
    tree: Tree,
    pub(crate) config: Configuration,
}

impl Database {
    pub(crate) fn new() -> Result<Self> {
        let database = sled::open(
            dirs::home_dir()
                .with_context(|| "Not found $HOME path")?
                .join(".tors/database"))?;

        let tree = database.open_tree("config")?;

        Ok(Self {
            database,
            tree,
            config: bincode::config::standard(),
        })
    }
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

    pub(crate) fn add_exp(&self, exp: u32) -> Result<()> {
        let last_exp =  self.get_exp()?;

        self.tree.insert("exp", bincode::encode_to_vec(exp + last_exp, self.config)?)?;

        Ok(())
    }

    pub(crate) fn get_exp(&self) -> Result<u32> {
        Ok(match self.tree.get("exp")? {
            Some(i) => bincode::decode_from_slice(&i, self.config)?.0,
            None => 0
        })
    }
}
