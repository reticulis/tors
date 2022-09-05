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
}

impl Database {
    pub fn new() -> Result<Self> {
        let db = sled::open(
            dirs::home_dir()
                .with_context(|| "Not found $HOME path")?
                .join(".tors/"),
        )?;

        let config = bincode::config::standard();

        Ok(Self {
            db,
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

    pub fn account(&self) -> Result<Account> {
        let data = self.db.get("account")?.with_context(|| "Failed get account field")?;

        Ok(bincode::serde::decode_from_slice(&data, self.config)?.0)
    }

    pub fn add_exp(&self, exp: u32) -> Result<()> {
        let mut account = self.account()?;

        account.exp += exp;
        account.lvl = (account.exp/10).sqrt().saturating_sub(1);

        self.db.insert(
            "account",
            bincode::serde::encode_to_vec(account, self.config)?,
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
