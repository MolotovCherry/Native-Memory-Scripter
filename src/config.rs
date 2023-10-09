use std::path::Path;
use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Need to figure out how to make a proper config?
/// quicktype.io can help you by converting json to Rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // This is not part of the config, but rather used for
    // at runtime to remember where to save to
    #[serde(skip)]
    path: PathBuf,

    opt1: bool,
    opt2: String,
    // etc
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let data = fs::read_to_string(&path)?;
        let mut config = toml::from_str::<Self>(&data)?;

        // set the plugin config path
        config.path = path.as_ref().to_owned();

        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let serialized = toml::to_string_pretty(self)?;
        fs::write(&self.path, serialized)?;

        Ok(())
    }
}
