use std::fs::{File, OpenOptions};

use log::LevelFilter;
use simplelog::{CombinedLogger, Config, WriteLogger};

use crate::paths::get_plugins_logs_filepath;

/// Setup logging for the plugin
///
/// NOTE: Have a particularly frustrating problem that you can't find EVEN with logging?
///       Using a Windows popup might be more helpful then.
///       DO NOT rely on popups in release mode. That will break the game!
pub fn setup_logging() -> anyhow::Result<()> {
    // get the file path to `C:\Users\<user>\AppData\Local\Larian Studios\Baldur's Gate 3\Plugins\logs\my-plugin.log`
    let log_path = get_plugins_logs_filepath("my-plugin.log")?;

    // either create log, or append to it if it already exists
    let file = if log_path.exists() {
        OpenOptions::new().write(true).append(true).open(log_path)?
    } else {
        File::create(log_path)?
    };

    // Log as debug level if compiled in debug, otherwise use info for releases
    let level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    // enable logging
    CombinedLogger::init(vec![WriteLogger::new(level, Config::default(), file)])?;

    Ok(())
}
