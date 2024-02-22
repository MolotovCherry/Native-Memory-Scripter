use eyre::eyre;
use mlua::prelude::*;
use windows::Win32::Foundation::ERROR_ACCESS_DENIED;

use crate::{
    console::{alloc_console, free_console},
    paths,
};

pub fn register(lua: &Lua) -> LuaResult<()> {
    let exports = lua.create_table()?;

    // This plugin's version
    exports.set("VERSION", env!("CARGO_PKG_VERSION"))?;
    // Whether this plugin was compiled in DEBUG mode or not
    exports.set("DEBUG", cfg!(debug_assertions))?;
    // dll_dir property
    exports.set("dll_dir", lua.create_string(&dll_dir()?)?)?;
    exports.set("show_console", lua.create_function(show_console)?)?;
    exports.set("hide_console", lua.create_function(hide_console)?)?;

    lua.globals().set("utils", exports)?;

    Ok(())
}

/// Shows the debug console
fn show_console(_: &Lua, _args: ()) -> LuaResult<()> {
    if let Err(e) = alloc_console() {
        if e != ERROR_ACCESS_DENIED.into() {
            return Err(e.into_lua_err());
        }
    }

    Ok(())
}

/// Hides the debug console
fn hide_console(_: &Lua, _args: ()) -> LuaResult<()> {
    free_console().into_lua_err()?;

    Ok(())
}

/// Gets the absolute path to the directory the dll plugin is located in
fn dll_dir() -> LuaResult<String> {
    let dir = paths::get_dll_dir(
        *crate::MODULE_HANDLE
            .get()
            .ok_or(eyre!("MODULE_HANDLE not set"))
            .into_lua_err()?,
    )
    .map(|p| p.to_string_lossy().to_string())
    .into_lua_err()?;

    Ok(dir)
}
