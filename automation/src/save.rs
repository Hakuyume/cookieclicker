use chrono::{DateTime, Utc};
use std::fmt;

pub struct Save {
    timestamp: DateTime<Utc>,
    code: String,
    data: cookieclicker_save::Save,
}

impl fmt::Debug for Save {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Save")
            .field("timestamp", &self.timestamp)
            .field("cookies_baked_all_time", &self.cookies_baked_all_time())
            .finish()
    }
}

impl Save {
    pub fn new(timestamp: DateTime<Utc>, code: String) -> Result<Self, cookieclicker_save::Error> {
        let data = cookieclicker_save::decode(&code)?;
        Ok(Self {
            timestamp,
            code,
            data,
        })
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn cookies_baked_all_time(&self) -> f64 {
        self.data.miscellaneous_game_data.cookies_baked
            + self
                .data
                .miscellaneous_game_data
                .cookies_forfeited_by_ascending
    }
}
