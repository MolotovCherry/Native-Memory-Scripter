mod config;
mod console;
mod interpreter;
mod logging;
mod modules;
mod paths;
mod popup;
mod utils;

use std::{ffi::c_void, panic, sync::OnceLock, thread};

use eyre::{Context, Result};
use native_plugin_lib::declare_plugin;
use tracing::error;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use config::Config;
use logging::setup_logging;
use paths::{get_dll_dir, get_dll_dir_filepath};

declare_plugin! {
    "Native Memory Scripter",
    "Cherry",
    "Easily edit process memory with dynamic scripts"
}

static MODULE_HANDLE: OnceLock<HINSTANCE> = OnceLock::new();

// Dll entry point
#[no_mangle]
extern "C-unwind" fn DllMain(
    module: HINSTANCE,
    fdw_reason: u32,
    _lpv_reserved: *const c_void,
) -> bool {
    #[allow(clippy::single_match)]
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            // IMPORTANT to run this code in another thread since we're not allowed to do much in dllmain
            // this would be a lot better if the dll loaders called an init function, but alas
            thread::spawn(move || {
                // make sure we catch panics so they don't propagate up any further
                // we already handle panic logging, so we don't care about the return value
                _ = panic::catch_unwind(move || {
                    if let Err(e) = pre_init(module) {
                        // whether this prints or not depends on which point it failed
                        error!("\nError:{e:?}");
                        return;
                    }

                    if let Err(error) = init(module) {
                        error!("\nError:{error:?}");
                    }
                });
            });
        }

        _ => (),
    }

    true
}

fn pre_init(module: HINSTANCE) -> Result<()> {
    _ = MODULE_HANDLE.set(module);

    // always spawn debug console when in debug mode
    #[cfg(debug_assertions)]
    console::alloc_console().context("failed to alloc console")?;

    let config_path = get_dll_dir_filepath(module, "native-memory-scripter.toml")
        .context("failed to get dir path")?;
    let config = Config::load(config_path).context("failed to load config")?;

    #[cfg(not(debug_assertions))]
    if config.dev.console {
        console::alloc_console().context("Failed to alloc console")?;
    }

    // set up our actual log file handling
    setup_logging(module, &config).context("failed to setup logging")?;

    Ok(())
}

fn init(module: HINSTANCE) -> Result<()> {
    let dll_dir = get_dll_dir(module).context("dll dir error")?;

    interpreter::run_scripts(&dll_dir).context("failed to run scripts")?;

    Ok(())
}
