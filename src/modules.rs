mod mem;
mod utils;

use mlua::prelude::*;

pub fn register(lua: &Lua) -> LuaResult<()> {
    utils::register(lua)?;
    mem::register(lua)?;

    Ok(())
}
