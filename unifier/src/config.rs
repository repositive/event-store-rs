use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    connections: HashMap<String, ConfigConnection>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigConnection {
    pub source_db_uri: String,
    pub dest_db_uri: String,
    pub domains: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let file = std::fs::read_to_string("./connections.toml").map_err(|e| e.to_string())?;

        let config = toml::from_str(&file).map_err(|e| e.to_string())?;

        Ok(config)
    }

    /// Get a config by key
    pub fn get(&self, key: &str) -> Option<&ConfigConnection> {
        self.connections.get(key)
    }
}
