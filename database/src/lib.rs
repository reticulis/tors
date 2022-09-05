mod achievements;

use crate::achievements::Achievements;
use anyhow::{Context, Result};
use bincode::config::Configuration;
use nanoid::nanoid;
use num_integer::Roots;
use serde::{Serialize, Deserialize};
use sled::{Db, Iter};

pub struct Database {
    pub db: Db,
    pub config: Configuration,
    pub account: Account,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db = sled::open(
            dirs::home_dir()
                .with_context(|| "Not found $HOME path")?
                .join(".tors/"),
        )?;

        let config = bincode::config::standard();

        let account = match db.get("account")? {
            Some(v) => bincode::serde::decode_from_slice(&v, config)?.0,
            None => Account::default(),
        };

        Ok(Self {
            db,
            account,
            config,
        })
    }

    pub fn add<T: Serialize>(&self, value: &T) -> Result<()> {
        let nanoid = nanoid!();

        self.insert(nanoid, value)?;

        Ok(())
    }

    pub fn insert<K: AsRef<[u8]>, T: Serialize>(&self, key: K, value: &T) -> Result<()> {
        self.db
            .insert(key, bincode::serde::encode_to_vec(value, self.config)?)?;

        Ok(())
    }

    pub fn remove<K: AsRef<[u8]>>(&self, key: K) -> Result<()> {
        self.db.remove(key)?;

        Ok(())
    }

    // pub fn contains<K: AsRef<[u8]>>(&mut self, key: K) -> bool {
    //     self.database.contains_key(key).unwrap_or(false)
    // }

    pub fn iter(&self) -> Iter {
        self.db.iter()
    }

    pub fn add_exp(&mut self, exp: u32) -> Result<()> {
        self.account.exp += exp;
        self.account.lvl = (self.account.exp/10).sqrt().saturating_sub(1);

        self.db.insert(
            "account",
            bincode::serde::encode_to_vec(&self.account, self.config)?,
        )?;

        Ok(())
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Account {
    pub lvl: u32,
    pub exp: u32,
    pub achievements: Vec<Achievements>,
}
