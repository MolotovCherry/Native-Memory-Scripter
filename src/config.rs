use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Need to figure out how to make a proper config?
/// quicktype.io can help you by converting json to Rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    opt1: bool,
    opt2: String,
    // etc
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let data = fs::read_to_string(path)?;
        let instance = toml::from_str::<Self>(&data)?;

        Ok(instance)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let serialized = toml::to_string_pretty(self)?;
        fs::write(path, serialized)?;

        Ok(())
    }
}
