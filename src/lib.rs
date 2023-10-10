mod backtrace;
mod config;
mod logging;
mod panic;
mod paths;
mod popup;

use std::ffi::c_void;

use log::error;
// See installation steps here: https://github.com/rdbo/libmem/tree/master/libmem-rs#installing
use bg3_plugin_lib::declare_plugin;
use libmem::*;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use crate::{
    config::Config,
    logging::setup_logging,
    paths::get_plugins_filepath,
    popup::{display_popup, MessageBoxIcon},
};

declare_plugin! {
    "MyPlugin",
    "My Plugin Description"
}

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
    let config_path = get_plugins_filepath("my-config.toml").expect("Failed to find config path");
    let config = Config::load(config_path).expect("Failed to load config");

    // save config
    config.save().expect("Failed to save config");

    todo!("Implement hooking logic");
}
