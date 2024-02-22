use eyre::Result;
use mlua::prelude::*;
use mlua::{Function, Variadic};
use owo_colors::OwoColorize;
use tracing::trace;

pub fn lua_init(lua: &Lua) -> Result<()> {
    crate::modules::register(lua)?;

    let print = lua.create_function(|_, args: Variadic<LuaValue>| {
        let text = args
            .iter()
            .map(|v| match v {
                LuaNil => "nil".to_string(),
                LuaValue::Boolean(b) => b.to_string(),
                LuaValue::LightUserData(l) => format!("cdata<void *>: {:p}", l.0),
                LuaValue::Integer(i) => i.to_string(),
                LuaValue::Number(n) => n.to_string(),
                LuaValue::String(s) => s.to_string_lossy().to_string(),
                LuaValue::Table(t) => format!("table: {:p}", t.to_pointer()),
                LuaValue::Function(f) => format!("function: {:p}", &f),
                LuaValue::Thread(t) => format!("thread: {:p}", &t),
                LuaValue::UserData(u) => {
                    trace!(?u, "print UserData");
                    if let Ok(metatable) = u.get_metatable() {
                        trace!(?metatable, "got metatable");
                        if let Ok(print_func) = metatable.get::<Function>("print") {
                            trace!(?print_func, "got print func");
                            if let Ok(text) = print_func.call::<_, String>(args.clone()) {
                                trace!(%text, "print got string");
                                return text;
                            }
                        }
                    }

                    format!("userdata: {:p}", &u)
                }
                LuaValue::Error(e) => e.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\t");

        println!("{} {text}", "[Script]".bold().bright_green());
        Ok(())
    })?;

    lua.globals().set("print", print)?;

    Ok(())
}
