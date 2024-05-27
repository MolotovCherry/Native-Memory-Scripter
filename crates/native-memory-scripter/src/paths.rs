use std::{
    ffi::OsString,
    os::windows::prelude::OsStringExt,
    path::{Path, PathBuf},
};

use windows::{
    core::HRESULT,
    Win32::{
        Foundation::{GetLastError, HINSTANCE, MAX_PATH, WIN32_ERROR},
        System::LibraryLoader::GetModuleFileNameW,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("{0} dir not found")]
    NotFound(String),
    #[error("{0}")]
    Windows(HRESULT),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

trait ToError {
    fn into_err(self) -> Result<(), HRESULT>;
}

impl ToError for WIN32_ERROR {
    fn into_err(self) -> Result<(), HRESULT> {
        if self.is_err() {
            Err(self.to_hresult())
        } else {
            Ok(())
        }
    }
}

impl From<HRESULT> for PathError {
    fn from(value: HRESULT) -> Self {
        Self::Windows(value)
    }
}

/// Get path to dll `<dll_dir>\myplugin.dll`
pub fn get_dll_path(module: HINSTANCE) -> Result<PathBuf, PathError> {
    const PATH_SIZE: usize = (MAX_PATH * 2) as usize;

    // create pre-allocated stack array of correct size
    let mut path = vec![0; PATH_SIZE];
    // returns how many bytes written
    let written_len = unsafe { GetModuleFileNameW(module, &mut path) as usize };

    // bubble up error if there was any, for example, ERROR_INSUFFICIENT_BUFFER
    unsafe {
        GetLastError().into_err()?;
    }

    let path = OsString::from_wide(&path[..written_len]);
    Ok(PathBuf::from(path))
}

/// Get path to dll's parent dir
pub fn get_dll_dir(module: HINSTANCE) -> Result<PathBuf, PathError> {
    let dll_folder = get_dll_path(module)?
        .parent()
        .ok_or_else(|| PathError::NotFound("parent".into()))?
        .to_path_buf();

    Ok(dll_folder)
}

/// Get path to `<dll_dir>\<filename>`
pub fn get_dll_dir_filepath<P: AsRef<Path>>(
    module: HINSTANCE,
    path: P,
) -> Result<PathBuf, PathError> {
    Ok(get_dll_dir(module)?.join(path))
}
