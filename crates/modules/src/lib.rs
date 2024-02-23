mod mem;
mod utils;

use std::{
    ffi::OsString, fmt::Display, os::windows::ffi::OsStringExt, path::PathBuf, sync::OnceLock,
};

use eyre::{eyre, Result};
use rhai::{Engine, EvalAltResult, Position};

pub use utils::console::{alloc_console, free_console};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{GetLastError, HMODULE, MAX_PATH},
        System::LibraryLoader::{
            GetModuleFileNameW, GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
        },
    },
};

pub fn register(engine: &Engine) -> Result<()> {
    utils::register(engine)?;
    //mem::register(engine)?;

    Ok(())
}

/// Get current DLL module handle
fn get_current_module() -> Result<&'static HMODULE> {
    static MODULE: OnceLock<HMODULE> = OnceLock::new();

    let mut error = Ok(());
    let module = MODULE.get_or_init(|| {
        let mut h_module: HMODULE = HMODULE::default();
        error = unsafe {
            GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
                PCWSTR(get_current_module as *const _),
                &mut h_module as *mut _,
            )
        };

        h_module
    });

    error.map(|_| module).map_err(|e| e.into())
}

/// Get path to dll `<dll_dir>`
fn get_dll_folder() -> Result<&'static PathBuf> {
    static PATH: OnceLock<PathBuf> = OnceLock::new();

    let path = if PATH.get().is_none() {
        const PATH_SIZE: usize = (MAX_PATH * 2) as usize;

        // create pre-allocated stack array of correct size
        let mut path = vec![0; PATH_SIZE];
        // returns how many bytes written
        let written_len = unsafe { GetModuleFileNameW(*get_current_module()?, &mut path) as usize };

        // bubble up error if there was any, for example, ERROR_INSUFFICIENT_BUFFER
        unsafe {
            GetLastError()?;
        }

        let path = PathBuf::from(OsString::from_wide(&path[..written_len]));

        let dll_folder = path
            .parent()
            .ok_or(eyre!("failed to get parent of dll"))?
            .to_path_buf();

        PATH.get_or_init(|| dll_folder)
    } else {
        PATH.get().unwrap()
    };

    Ok(path)
}

#[inline(always)]
fn into_usize(val: i64, pos: Position) -> Result<usize, Box<EvalAltResult>> {
    val.try_into().into_rhai_pos(pos)
}

#[inline(always)]
fn into_u64(val: i64, pos: Position) -> Result<u64, Box<EvalAltResult>> {
    val.try_into().into_rhai_pos(pos)
}

// for converting errors to Box<EvalAltResult>
trait IntoRhaiError<T, E> {
    fn into_rhai(self) -> Result<T, Box<EvalAltResult>>
    where
        E: Display;

    fn into_rhai_msg(self, msg: &str) -> Result<T, Box<EvalAltResult>>;

    fn into_rhai_msg_pos(self, msg: &str, position: Position) -> Result<T, Box<EvalAltResult>>;

    fn into_rhai_pos(self, position: Position) -> Result<T, Box<EvalAltResult>>
    where
        E: Display;
}

trait IntoRhaiOption<T> {
    fn into_rhai(self, msg: &str) -> Result<T, Box<EvalAltResult>>;

    fn into_rhai_pos(self, msg: &str, position: Position) -> Result<T, Box<EvalAltResult>>;
}

impl<T, E> IntoRhaiError<T, E> for Result<T, E> {
    fn into_rhai(self) -> Result<T, Box<EvalAltResult>>
    where
        E: Display,
    {
        self.map_err(|e| EvalAltResult::ErrorRuntime(e.to_string().into(), Position::NONE).into())
    }

    fn into_rhai_pos(self, position: Position) -> Result<T, Box<EvalAltResult>>
    where
        E: Display,
    {
        self.map_err(|e| EvalAltResult::ErrorRuntime(e.to_string().into(), position).into())
    }

    fn into_rhai_msg(self, msg: &str) -> Result<T, Box<EvalAltResult>> {
        self.map_err(|_| EvalAltResult::ErrorRuntime(msg.to_owned().into(), Position::NONE).into())
    }

    fn into_rhai_msg_pos(self, msg: &str, position: Position) -> Result<T, Box<EvalAltResult>> {
        self.map_err(|_| EvalAltResult::ErrorRuntime(msg.to_owned().into(), position).into())
    }
}

impl<T> IntoRhaiOption<T> for Option<T> {
    fn into_rhai(self, msg: &str) -> Result<T, Box<EvalAltResult>> {
        self.ok_or_else(|| EvalAltResult::ErrorRuntime(msg.into(), Position::NONE).into())
    }

    fn into_rhai_pos(self, msg: &str, position: Position) -> Result<T, Box<EvalAltResult>> {
        self.ok_or_else(|| EvalAltResult::ErrorRuntime(msg.into(), position).into())
    }
}
