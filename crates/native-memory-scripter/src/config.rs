use std::fs;
use std::path::Path;

use eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub dev: Dev,
    pub log: Log,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Dev {
    /// show the developer console
    pub console: bool,
    /// show dev mode tools
    pub dev_mode: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Log {
    /// configure logger level
    pub level: String,
    /// whether to display log targets
    pub targets: bool,
}

impl Config {
    /// Load a config file
    /// If path doesn't exist, creates and saves default config
    /// otherwise loads what's already there
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // if path doesn't exist, create default config,
        // save it, and return it
        if !path.exists() {
            let config = Self {
                dev: Dev {
                    console: false,
                    dev_mode: false,
                },
                log: Log {
                    level: "info".to_owned(),
                    targets: false,
                },
            };

            let serialized = toml::to_string_pretty(&config)?;
            fs::write(path, serialized)?;
            return Ok(config);
        }

        let data = fs::read_to_string(path)?;
        let config = toml::from_str::<Self>(&data)?;

        Ok(config)
    }
}
