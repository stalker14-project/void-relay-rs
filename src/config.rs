use std::{fs, path::PathBuf, str::FromStr};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    discord_bot_token: String,
    ss14_database: String,
    api_host: String,
    guild_id: String,
}

impl Config {
    pub fn token(&self) -> &str {
        &self.discord_bot_token
    }

    pub fn cstr(&self) -> &str {
        &self.ss14_database
    }

    pub fn host(&self) -> &str {
        &self.api_host
    }

    pub fn guild(&self) -> &str {
        &self.guild_id
    }
}

// assume str as path
impl FromStr for Config {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path_buf = PathBuf::from_str(s).unwrap();
        let data = fs::read_to_string(path_buf)?;
        let data: Config = serde_json::from_str(&data)?;
        
        Ok(data)
    }
}


