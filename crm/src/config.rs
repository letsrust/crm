use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub pk: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub sender_email: String,
    pub metadata: String,
    pub user_stats: String,
    pub notification: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config = match File::open("crm.yml") {
            Ok(reader) => serde_yaml::from_reader(reader),
            _ => bail!("Config file not found"),
        };

        Ok(config?)
    }
}
