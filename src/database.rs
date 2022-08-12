use crate::ui::Task;
use crate::App;
use anyhow::{Context, Result};
use bincode::config::Configuration;
use sled::Db;
use uuid::Uuid;

pub struct Database {
    pub(crate) database: Db,
    pub(crate) config: Configuration,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            database: sled::open(dirs::home_dir().unwrap().join(".tors")).unwrap(),
            config: bincode::config::standard(),
        }
    }
}

impl App {
    pub(crate) fn get_task(&mut self, id: usize) -> Result<Task> {
        // TODO
        // Improve performance
        // Copy data!
        let (id, _) = self
            .tasks
            .items
            .get(id)
            .with_context(|| "Not found task!")?;

        let ivec = self
            .database
            .database
            .get(id)?
            .with_context(|| "Not found in database")?;

        let (task, _) = bincode::serde::decode_from_slice::<Task, _>(&ivec, self.database.config)?;

        Ok(task)
    }

    pub(crate) fn add_to_db(&mut self) -> Result<()> {
        let uuid = Uuid::new_v4().to_string();

        if self.contains_uuid(&uuid).is_some() {
            self.add_to_db()?;

            return Ok(());
        }

        self.insert(&uuid)?;

        Ok(())
    }

    pub(crate) fn rm_from_db(&mut self) -> Result<()> {
        if let Some((id, _)) = &self.tasks.items.get(
            self.tasks
                .state
                .selected()
                .with_context(|| "Failed get element")?,
        ) {
            self.database.database.remove(id)?;
        }

        Ok(())
    }

    pub(crate) fn update_db(&mut self) -> Result<()> {
        if let Some((id, _)) = &self
            .tasks
            .items
            .get(self.tasks.state.selected().unwrap_or(0))
        {
            self.insert(&id.to_string())?;
        }

        Ok(())
    }

    pub fn contains_uuid(&mut self, uuid: &str) -> Option<()> {
        if let Ok(None) = self.database.database.get(uuid) {
            return None;
        }

        Some(())
    }

    fn insert(&mut self, date: &str) -> Result<()> {
        self.database.database.insert(
            date,
            bincode::serde::encode_to_vec(self.task.clone(), self.database.config)?,
        )?;

        Ok(())
    }
}
