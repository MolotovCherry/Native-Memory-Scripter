mod mem;
mod utils;

pub fn register(lua: &Lua) -> Result<()> {
    utils::register(lua)?;
    mem::register(lua)?;

    Ok(())
}
