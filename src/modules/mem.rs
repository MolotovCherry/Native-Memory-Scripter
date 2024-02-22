use libmem::*;
use mlua::prelude::*;
use mlua::{FromLua, Value};

pub fn register(lua: &Lua) -> LuaResult<()> {
    let exports = lua.create_table()?;

    exports.set("alloc_memory", lua.create_function(alloc_memory)?)?;
    exports.set("alloc_memory_ex", lua.create_function(alloc_memory_ex)?)?;

    // LmProtT
    //
    let lm_prot_t = lua.create_table()?;
    lm_prot_t.set(
        "LM_PROT_NONE",
        lua.create_any_userdata(LmProtT::LmProtNone)?,
    )?;
    lm_prot_t.set("LM_PROT_R", lua.create_any_userdata(LmProtT::LmProtR)?)?;
    lm_prot_t.set("LM_PROT_RW", lua.create_any_userdata(LmProtT::LmProtRw)?)?;
    lm_prot_t.set("LM_PROT_W", lua.create_any_userdata(LmProtT::LmProtW)?)?;
    lm_prot_t.set("LM_PROT_X", lua.create_any_userdata(LmProtT::LmProtX)?)?;
    lm_prot_t.set("LM_PROT_XR", lua.create_any_userdata(LmProtT::LmProtXr)?)?;
    lm_prot_t.set("LM_PROT_XRW", lua.create_any_userdata(LmProtT::LmProtXrw)?)?;
    lm_prot_t.set("LM_PROT_XW", lua.create_any_userdata(LmProtT::LmProtXw)?)?;
    lua.register_userdata_type::<LmProtT>(|_| ())?;
    exports.set("lm_prot_t", lm_prot_t)?;

    lua.register_userdata_type::<LmProcessT>(|_| ())?;

    lua.globals().set("mem", exports)?;

    Ok(())
}

/// LM_AllocMemory(size: usize, )
/// Allocates size bytes of memory with protection flags prot in the calling process.
///
/// Parameters:
/// size: the size of the region to change the protection flags.
/// prot: the protection flags (LM_PROT_*).
///
/// Return:
/// On success, it returns address, where address is a valid lm_address_t (usize). On failure, it returns None (nil).
///
/// https://github.com/rdbo/libmem/blob/4.4.0/docs/api/rust/LM_AllocMemory.md
fn alloc_memory(lua: &Lua, args: (usize, LmProtT)) -> LuaResult<Value> {
    let address = LM_AllocMemory(args.0, args.1.into()).into_lua(lua)?;
    Ok(address)
}

/// LM_AllocMemoryEx(size: usize, )
/// Allocates size bytes of memory with protection flags prot in a remote process.
///
/// Parameters:
/// pproc: immutable pointer to a process which will have memory be allocated.
/// size: the size of the region to change the protection flags.
/// prot: the protection flags (LM_PROT_*).
///
/// Return:
/// On success, it returns address, where address is a valid lm_address_t (usize). On failure, it returns None (nil).
///
/// https://github.com/rdbo/libmem/blob/4.4.0/docs/api/rust/LM_AllocMemory.md
fn alloc_memory_ex(lua: &Lua, args: (LmProcessT, usize, LmProtT)) -> LuaResult<Value> {
    let address = LM_AllocMemoryEx(&args.0 .0, args.1, args.2.into()).into_lua(lua)?;
    Ok(address)
}

#[derive(Clone, FromLua)]
struct LmProcessT(lm_process_t);

impl LuaUserData for LmProcessT {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("pid", |_, this| Ok(this.0.get_pid()));
        fields.add_field_method_get("ppid", |_, this| Ok(this.0.get_ppid()));
        fields.add_field_method_get("bits", |_, this| Ok(this.0.get_bits()));
        fields.add_field_method_get("start_time", |_, this| Ok(this.0.get_start_time()));
        fields.add_field_method_get("path", |_, this| Ok(this.0.get_path()));
        fields.add_field_method_get("name", |_, this| Ok(this.0.get_name()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("print", |_, this, ()| {
            println!("{:?}", this.0);
            Ok(())
        });
    }
}

/// Protection flags
#[derive(Clone, Debug, FromLua)]
enum LmProtT {
    LmProtNone,
    LmProtX,
    LmProtR,
    LmProtW,
    LmProtXr,
    LmProtXw,
    LmProtRw,
    LmProtXrw,
}

impl From<LmProtT> for lm_prot_t {
    fn from(value: LmProtT) -> Self {
        match value {
            LmProtT::LmProtNone => lm_prot_t::LM_PROT_NONE,
            LmProtT::LmProtX => lm_prot_t::LM_PROT_X,
            LmProtT::LmProtR => lm_prot_t::LM_PROT_R,
            LmProtT::LmProtW => lm_prot_t::LM_PROT_W,
            LmProtT::LmProtXr => lm_prot_t::LM_PROT_XR,
            LmProtT::LmProtXw => lm_prot_t::LM_PROT_XW,
            LmProtT::LmProtRw => lm_prot_t::LM_PROT_RW,
            LmProtT::LmProtXrw => lm_prot_t::LM_PROT_XRW,
        }
    }
}

impl LuaUserData for LmProtT {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("LM_PROT_NONE", |_, _| Ok(Self::LmProtNone));
        fields.add_field_method_get("LM_PROT_X", |_, _| Ok(Self::LmProtX));
        fields.add_field_method_get("LM_PROT_R", |_, _| Ok(Self::LmProtR));
        fields.add_field_method_get("LM_PROT_W", |_, _| Ok(Self::LmProtW));
        fields.add_field_method_get("LM_PROT_XR", |_, _| Ok(Self::LmProtXr));
        fields.add_field_method_get("LM_PROT_XW", |_, _| Ok(Self::LmProtXw));
        fields.add_field_method_get("LM_PROT_RW", |_, _| Ok(Self::LmProtRw));
        fields.add_field_method_get("LM_PROT_XRW", |_, _| Ok(Self::LmProtXrw));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("print", |_, this, ()| {
            println!("{this:?}");
            Ok(())
        });
    }
}
