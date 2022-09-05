use serde::{Serialize, Deserialize};
use chrono::{Datelike, Local, NaiveDateTime};

#[derive(Serialize, Deserialize, Default)]
pub struct Task {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) done: bool,
    pub(crate) exp_added: bool,
    pub(crate) creation_date: NaiveDateTime,
    pub(crate) preferences: Preferences,
}

#[derive(Serialize, Deserialize)]
pub struct Preferences {
    pub(crate) daily_repeat: bool,
    pub(crate) expire: NaiveDateTime,
    pub(crate) exp: u32,
    // TODO
    // Another parameters
}

impl Default for Preferences {
    fn default() -> Self {
        let now = Local::now().naive_local();

        let expire = match now.with_day(now.day() + 1) {
            Some(date) => date,
            None => match now.with_month(now.month() + 1) {
                Some(date) => date.with_day(1).unwrap(),
                None => now
                    .with_year(now.year() + 1)
                    .unwrap()
                    .with_month(1)
                    .unwrap()
                    .with_day(1)
                    .unwrap(),
            },
        };

        Self {
            daily_repeat: false,
            expire,
            exp: 25,
        }
    }
}
