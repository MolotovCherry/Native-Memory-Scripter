mod backtrace;
mod config;
mod console;
mod logging;
mod lua;
mod modules;
mod panic;
mod paths;

use std::{ffi::c_void, sync::OnceLock};

use eyre::{Context, Error, Result};
use mlua::prelude::*;
use native_plugin_lib::declare_plugin;
use tracing::error;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use config::Config;
use logging::setup_logging;
use lua::lua_init;
use paths::get_dll_dir_filepath;

use self::console::alloc_console;

declare_plugin! {
    "Native-Memory-Scripter",
    "Cherry",
    "Allows one to use lua scripting to modify the games memory"
}

static MODULE_HANDLE: OnceLock<HINSTANCE> = OnceLock::new();

// Dll entry point
#[no_mangle]
extern "C-unwind" fn DllMain(module: HINSTANCE, fdw_reason: u32, _lpv_reserved: *const c_void) {
    _ = MODULE_HANDLE.set(module);

    #[allow(clippy::single_match)]
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            panic::set_hook();

            let result = std::panic::catch_unwind(|| {
                // set up our actual log file handling
                setup_logging(module).context("Failed to setup logging")?;

                // always spawn debug console when in debug mode
                #[cfg(debug_assertions)]
                alloc_console()?;

                entry(module)?;

                Ok::<_, Error>(())
            });

            // If there was no panic, but error was bubbled up, then log the error
            if let Ok(Err(e)) = result {
                error!("{e}");
            }
        }

        _ => (),
    }
}

fn entry(module: HINSTANCE) -> Result<()> {
    let config_path = get_dll_dir_filepath(module, "native-memory-scriper.toml")?;
    let config = Config::load(config_path)?;

    let lua = unsafe { Lua::unsafe_new() };
    lua_init(&lua)?;

    let data = std::fs::read_to_string(r"R:\Temp\rust\debug\test.lua")?;
    lua.load(data).exec()?;

    Ok(())
}
