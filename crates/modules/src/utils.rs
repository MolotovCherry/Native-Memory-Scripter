pub mod console;

use eyre::Result;
use konst::{primitive::parse_i64, unwrap_ctx};
use rhai::plugin::*;
use windows::Win32::Foundation::ERROR_ACCESS_DENIED;

use crate::IntoRhaiError;
use console::{alloc_console, free_console};

pub fn register(engine: &Engine) -> Result<()> {
    Ok(())
}

#[export_module]
mod utils_mod {
    /// This plugin's version
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    /// This plugin's major version
    pub const VERSION_MAJOR: i64 = unwrap_ctx!(parse_i64(env!("CARGO_PKG_VERSION_MAJOR")));
    /// This plugin's minor version
    pub const VERSION_MINOR: i64 = unwrap_ctx!(parse_i64(env!("CARGO_PKG_VERSION_MINOR")));
    /// This plugin's patch version
    pub const VERSION_PATCH: i64 = unwrap_ctx!(parse_i64(env!("CARGO_PKG_VERSION_PATCH")));
    /// This plugin's pre version
    pub const VERSION_PRE: &str = env!("CARGO_PKG_VERSION_PRE");

    /// Whether this plugin was compiled in DEBUG mode or not
    pub const DEBUG: bool = cfg!(debug_assertions);

    /// Shows the debug console
    #[rhai_fn(volatile, return_raw)]
    fn show_console(ctx: NativeCallContext) -> Result<(), Box<EvalAltResult>> {
        if let Err(e) = alloc_console() {
            if e != ERROR_ACCESS_DENIED.into() {
                return Err(e).into_rhai_pos(ctx.position());
            }
        }

        Ok(())
    }

    /// Hides the debug console
    #[rhai_fn(volatile, return_raw)]
    fn hide_console(ctx: NativeCallContext) -> Result<(), Box<EvalAltResult>> {
        free_console().into_rhai_pos(ctx.position())?;

        Ok(())
    }

    /// Gets the path to the folder the dll is located in
    #[rhai_fn(return_raw)]
    fn dll_dir(ctx: NativeCallContext) -> Result<String, Box<EvalAltResult>> {
        let path = crate::get_dll_folder()
            .into_rhai_pos(ctx.position())?
            .to_string_lossy()
            .into_owned();

        Ok(path)
    }
}
