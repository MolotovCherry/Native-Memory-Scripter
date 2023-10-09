use std::path::{Path, PathBuf};

use anyhow::anyhow;
use directories::BaseDirs;

/// Get the larian local directory
/// `C:\Users\<user>\AppData\Local\Larian Studios`
pub fn get_larian_local_dir() -> anyhow::Result<PathBuf> {
    let local = BaseDirs::new().ok_or(anyhow!("Failed to instantiate BaseDirs"))?;

    let mut local = local.data_local_dir().to_owned();

    local.push("Larian Studios");
    if local.exists() {
        Ok(local)
    } else {
        Err(anyhow!("Larian local appdata directory does not exist"))
    }
}

/// Get the BG3 local directory
/// `C:\Users\<user>\AppData\Local\Larian Studios\Baldur's Gate 3`
pub fn get_bg3_local_dir() -> anyhow::Result<PathBuf> {
    let mut local = get_larian_local_dir()?;

    local.push("Baldur's Gate 3");

    if local.exists() {
        Ok(local)
    } else {
        Err(anyhow!("Bg3 local appdata directory does not exist"))
    }
}

/// Get the bg3 plugins directory
/// `C:\Users\<user>\AppData\Local\Larian Studios\Baldur's Gate 3\Plugins`
pub fn get_bg3_plugins_dir() -> anyhow::Result<PathBuf> {
    let mut plugins_dir = get_bg3_local_dir()?;
    plugins_dir.push("Plugins");

    if plugins_dir.exists() {
        Ok(plugins_dir)
    } else {
        Err(anyhow!("BG3 Plugins dir not found"))
    }
}

/// Get the bg3 plugins log directory
/// `C:\Users\<user>\AppData\Local\Larian Studios\Baldur's Gate 3\Plugins`
pub fn get_bg3_plugins_logs_dir() -> anyhow::Result<PathBuf> {
    let mut log_dir = get_bg3_plugins_dir()?;
    log_dir.push("logs");

    if log_dir.exists() {
        Ok(log_dir)
    } else {
        Err(anyhow!("BG3 Plugins logs dir not found"))
    }
}

/// Create a path to `C:\Users\<user>\AppData\Local\Larian Studios\Baldur's Gate 3\Plugins\<filename>`
pub fn get_plugins_filepath<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    Ok(get_bg3_plugins_dir()?.join(path))
}

/// Create a path to `C:\Users\<user>\AppData\Local\Larian Studios\Baldur's Gate 3\Plugins\logs\<filename>`
pub fn get_plugins_logs_filepath<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    Ok(get_bg3_plugins_logs_dir()?.join(path))
}
