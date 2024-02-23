use std::{
    error::Error,
    ffi::OsString,
    fs,
    os::windows::prelude::OsStringExt,
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use windows::Win32::{
    Foundation::{GetLastError, HINSTANCE, MAX_PATH},
    System::LibraryLoader::GetModuleFileNameW,
};

/// Get the larian local directory
/// `C:\Users\<user>\AppData\Local\Larian Studios`
pub fn get_larian_local_dir() -> Result<PathBuf, Box<dyn Error + Send + Sync + 'static>> {
    let local = BaseDirs::new().ok_or("Failed to instantiate BaseDirs")?;

    let mut local = local.data_local_dir().to_owned();

    local.push("Larian Studios");
    if local.exists() {
        Ok(local)
    } else {
        Err("Larian local appdata directory does not exist".into())
    }
}

/// Get the BG3 local directory
/// `C:\Users\<user>\AppData\Local\Larian Studios\Baldur's Gate 3`
pub fn get_bg3_local_dir() -> Result<PathBuf, Box<dyn Error + Send + Sync + 'static>> {
    let mut local = get_larian_local_dir()?;

    local.push("Baldur's Gate 3");

    if local.exists() {
        Ok(local)
    } else {
        Err("Bg3 local appdata directory does not exist".into())
    }
}

/// Get the bg3 plugins directory
/// `C:\Users\<user>\AppData\Local\Larian Studios\Baldur's Gate 3\Plugins`
pub fn get_bg3_plugins_dir() -> Result<PathBuf, Box<dyn Error + Send + Sync + 'static>> {
    let mut plugins_dir = get_bg3_local_dir()?;
    plugins_dir.push("Plugins");

    if plugins_dir.exists() {
        Ok(plugins_dir)
    } else {
        Err("BG3 Plugins dir not found".into())
    }
}

/// Create a path to `C:\Users\<user>\AppData\Local\Larian Studios\Baldur's Gate 3\Plugins\<filename>`
pub fn get_plugins_filepath<P: AsRef<Path>>(
    path: P,
) -> Result<PathBuf, Box<dyn Error + Send + Sync + 'static>> {
    Ok(get_bg3_plugins_dir()?.join(path))
}

/// Get path to dll `<dll_dir>\myplugin.dll`
pub fn get_dll_path(module: HINSTANCE) -> Result<PathBuf, Box<dyn Error + Send + Sync + 'static>> {
    const PATH_SIZE: usize = (MAX_PATH * 2) as usize;

    // create pre-allocated stack array of correct size
    let mut path = vec![0; PATH_SIZE];
    // returns how many bytes written
    let written_len = unsafe { GetModuleFileNameW(module, &mut path) as usize };

    // bubble up error if there was any, for example, ERROR_INSUFFICIENT_BUFFER
    unsafe {
        GetLastError()?;
    }

    let path = OsString::from_wide(&path[..written_len]);
    Ok(PathBuf::from(path))
}

/// Get path to dll's parent dir
pub fn get_dll_dir(module: HINSTANCE) -> Result<PathBuf, Box<dyn Error + Send + Sync + 'static>> {
    let dll_folder = get_dll_path(module)?
        .parent()
        .ok_or("Failed to get parent of dll")?
        .to_path_buf();

    Ok(dll_folder)
}

/// Get path to `<dll_dir>\logs\`
/// Also creates `logs` dir if it doesn't exist
pub fn get_dll_logs_dir(
    module: HINSTANCE,
) -> Result<PathBuf, Box<dyn Error + Send + Sync + 'static>> {
    let mut logs_dir = get_dll_dir(module)?;
    logs_dir.push("logs");

    if !logs_dir.exists() {
        fs::create_dir(&logs_dir)?;
    }

    Ok(logs_dir)
}

/// Get path to `<dll_dir>\<filename>`
pub fn get_dll_dir_filepath<P: AsRef<Path>>(
    module: HINSTANCE,
    path: P,
) -> Result<PathBuf, Box<dyn Error + Send + Sync + 'static>> {
    Ok(get_dll_dir(module)?.join(path))
}

/// Get path to `<dll_dir>\logs\<filename>`
/// Also creates `logs` dir if it doesn't exist
pub fn get_dll_logs_filepath<P: AsRef<Path>>(
    module: HINSTANCE,
    path: P,
) -> Result<PathBuf, Box<dyn Error + Send + Sync + 'static>> {
    let logs_dir = get_dll_logs_dir(module)?;
    Ok(logs_dir.join(path))
}
