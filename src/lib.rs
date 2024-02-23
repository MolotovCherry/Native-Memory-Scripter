mod config;
mod console;
mod engine;
mod logging;
//mod modules;

mod paths;

use std::{
    ffi::c_void,
    sync::{Mutex, OnceLock},
};

use eyre::{eyre, Result};
use native_plugin_lib::declare_plugin;
use tracing::error;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use config::Config;
//use engine::init;
use logging::setup_logging;
use paths::get_dll_dir_filepath;

use self::console::alloc_console;

declare_plugin! {
    "Native-Memory-Scripter",
    "Cherry",
    "Allows one to use lua scripting to modify the games memory"
}

static MODULE_HANDLE: OnceLock<HINSTANCE> = OnceLock::new();
static RUNNING_SCRIPT: OnceLock<Mutex<String>> = OnceLock::new();

// Dll entry point
#[no_mangle]
extern "C-unwind" fn DllMain(module: HINSTANCE, fdw_reason: u32, _lpv_reserved: *const c_void) {
    #[allow(clippy::single_match)]
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            _ = MODULE_HANDLE.set(module);
            _ = RUNNING_SCRIPT.set(Mutex::new(String::new()));

            _ = std::panic::catch_unwind(|| {
                // always spawn debug console when in debug mode
                #[cfg(debug_assertions)]
                alloc_console().expect("Failed to alloc console");

                // set up our actual log file handling
                setup_logging(module).expect("Failed to setup logging");

                entry(module).expect("entry failure");
            });
        }

        _ => (),
    }
}

fn entry(module: HINSTANCE) -> Result<()> {
    let config_path =
        get_dll_dir_filepath(module, "native-memory-scriper.toml").map_err(|e| eyre!("{e}"))?;
    let config = Config::load(config_path)?;

    error!("bar");
    panic!("foobar");

    Ok(())
}

fn get_running_script() -> String {
    RUNNING_SCRIPT
        .get()
        .map(|m| m.lock().unwrap().clone())
        .unwrap_or_default()
}
