mod config;
mod console;
mod engine;
mod logging;
mod paths;

use std::{ffi::c_void, sync::OnceLock};

use eyre::Result;
use native_plugin_lib::declare_plugin;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use config::Config;
//use engine::init;
use logging::setup_logging;
use paths::get_dll_dir_filepath;

declare_plugin! {
    "Native Memory Scripter",
    "Cherry",
    "Easily edit process memory with dynamic scripts"
}

static MODULE_HANDLE: OnceLock<HINSTANCE> = OnceLock::new();

// Dll entry point
#[no_mangle]
extern "C-unwind" fn DllMain(module: HINSTANCE, fdw_reason: u32, _lpv_reserved: *const c_void) {
    #[allow(clippy::single_match)]
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            _ = MODULE_HANDLE.set(module);

            _ = std::panic::catch_unwind(|| {
                // always spawn debug console when in debug mode
                #[cfg(debug_assertions)]
                console::alloc_console().expect("failed to alloc console");

                let config_path = get_dll_dir_filepath(module, "native-memory-scripter.toml")
                    .expect("failed to get dir path");
                let config = Config::load(config_path).expect("failed to load config");

                #[cfg(not(debug_assertions))]
                if config.console {
                    console::alloc_console().expect("Failed to alloc console");
                }

                // set up our actual log file handling
                setup_logging(module, &config.log_level).expect("Failed to setup logging");

                entry(module).expect("entry failure");
            });
        }

        _ => (),
    }
}

fn entry(module: HINSTANCE) -> Result<()> {
    Ok(())
}
