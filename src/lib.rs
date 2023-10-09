mod backtrace;
mod config;
mod panic;
mod paths;
mod popup;

use std::{
    ffi::c_void,
    fs::{File, OpenOptions},
};

// See installation steps here: https://github.com/rdbo/libmem/tree/master/libmem-rs#installing
//use libmem::*;
use log::{error, LevelFilter};
use simplelog::{CombinedLogger, Config as SimpleLogConfig, WriteLogger};
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use crate::{
    config::Config,
    paths::get_plugins_filepath,
    popup::{display_popup, MessageBoxIcon},
};

use self::paths::get_plugins_logs_filepath;

// Dll entry point
#[no_mangle]
extern "C-unwind" fn DllMain(_hinst_dll: HINSTANCE, fdw_reason: u32, _lpv_reserved: *const c_void) {
    #[allow(clippy::single_match)]
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            // Note: While it's technically safe to panic across FFI with C-unwind ABI, I STRONGLY recommend to
            // catch and handle ALL panics. If you don't, you could crash the game by accident!
            let result = std::panic::catch_unwind(|| {
                entry();
            });

            // Just log the error out to the file
            if let Err(e) = result {
                // Note, logging this is contingent on a successful downcast!
                // It would be better for you to handle any possible panics directly,
                // and instead use the log macros, log::error!(), log::info!(), etc

                if let Ok(message) = e.downcast::<&'static str>() {
                    error!("{message}");
                } else {
                    error!("General error");
                }
            }
        }

        _ => (),
    }
}

// All of our main plugin code goes here!
fn entry() {
    // TODO: Place all your hooking code here

    // Set up a custom panic hook so we can log all panics to logfile
    panic::set_hook();

    // set up our actual log file handling
    setup_logging().expect("Failed to setup logging");

    // Show the hook was injected. DO NOT popup in production code!
    display_popup(
        "Success",
        "Plugin successfully injected",
        MessageBoxIcon::Information,
    );

    // load a config
    let config_path = get_plugins_filepath("my-config.toml").expect("Failed to load settings");
    let config = Config::load(config_path).expect("Failed to load config");

    // save config
    config.save().expect("Failed to save config");

    todo!("Implement hooking logic");
}

/// Setup logging for the plugin
///
/// NOTE: Have a particularly frustrating problem that you can't find EVEN with logging?
///       Using a Windows popup might be more helpful then.
///       DO NOT rely on popups in release mode. That will break the game!
fn setup_logging() -> anyhow::Result<()> {
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
    CombinedLogger::init(vec![WriteLogger::new(
        level,
        SimpleLogConfig::default(),
        file,
    )])?;

    Ok(())
}
