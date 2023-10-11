mod backtrace;
mod config;
mod logging;
mod panic;
mod paths;
mod popup;

use std::ffi::c_void;

use bg3_plugin_lib::declare_plugin;
// See installation steps here: https://github.com/rdbo/libmem/tree/master/libmem-rs#installing
use libmem::*;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::{Foundation::HINSTANCE, System::Diagnostics::Debug::IsDebuggerPresent};

use crate::{
    config::Config,
    logging::setup_logging,
    paths::get_plugins_filepath,
    popup::{display_popup, MessageBoxIcon},
};

// Declare your plugin name and description
// This will be accessible by anyone who uses the BG3-Plugin-Lib to get the info
declare_plugin! {
    "MyPlugin",
    "Author",
    "My Plugin Description"
}

// Dll entry point
#[no_mangle]
extern "C-unwind" fn DllMain(_hinst_dll: HINSTANCE, fdw_reason: u32, _lpv_reserved: *const c_void) {
    #[allow(clippy::single_match)]
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            // Wait for debugger if in debug mode
            if cfg!(debug_assertions) {
                let is_debugger_present = || unsafe { IsDebuggerPresent().as_bool() };

                while !is_debugger_present() {
                    // 60hz polling
                    std::thread::sleep(std::time::Duration::from_millis(16));
                }
            }

            // Set up a custom panic hook so we can log all panics to logfile
            panic::set_hook();

            // Note: While it's technically safe to panic across FFI with C-unwind ABI, I STRONGLY recommend to
            // catch and handle ALL panics. If you don't, you could crash the game by accident!
            //
            // catch_unwind returns a Result with the panic info, but we actually don't need it, because
            // we set a panic_hook up at top which will log all panics to the logfile.
            // if for any reason we can't actually log the panic, we *could* popup a
            // messagebox instead (for debugging use only of course)
            _ = std::panic::catch_unwind(|| {
                // set up our actual log file handling
                setup_logging().expect("Failed to setup logging");

                entry();
            });
        }

        _ => (),
    }
}

// All of our main plugin code goes here!
//
// To log to the logfile, use the log macros: log::debug!(), log::info!(), log::warn!(), log::error!()
// Recommend to catch and handle potential panics instead of panicking; log instead, it's much cleaner
fn entry() {
    // TODO: Place all your hooking code here

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
