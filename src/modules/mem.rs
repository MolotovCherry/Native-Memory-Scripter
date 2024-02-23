use eyre::eyre;
use libmem::*;
use mlua::prelude::*;
use mlua::{FromLua, Value, Variadic};

use crate::engine::lua_print;

pub fn register(lua: &Lua) -> LuaResult<()> {
    let exports = lua.create_table()?;

    exports.set("alloc_memory", lua.create_function(alloc_memory)?)?;
    exports.set("alloc_memory_ex", lua.create_function(alloc_memory_ex)?)?;
    exports.set("assemble", lua.create_function(assemble)?)?;
    exports.set("assemble_ex", lua.create_function(assemble_ex)?)?;
    exports.set("code_length", lua.create_function(code_length)?)?;
    exports.set("code_length_ex", lua.create_function(code_length_ex)?)?;

    // lm_prot_t
    // Protection Flags
    let lm_prot_t = lua.create_table()?;
    lm_prot_t.set("LM_PROT_NONE", lm_prot_t::LM_PROT_NONE as isize)?;
    lm_prot_t.set("LM_PROT_R", lm_prot_t::LM_PROT_R as isize)?;
    lm_prot_t.set("LM_PROT_RW", lm_prot_t::LM_PROT_RW as isize)?;
    lm_prot_t.set("LM_PROT_W", lm_prot_t::LM_PROT_W as isize)?;
    lm_prot_t.set("LM_PROT_X", lm_prot_t::LM_PROT_X as isize)?;
    lm_prot_t.set("LM_PROT_XR", lm_prot_t::LM_PROT_XR as isize)?;
    lm_prot_t.set("LM_PROT_XRW", lm_prot_t::LM_PROT_XRW as isize)?;
    lm_prot_t.set("LM_PROT_XW", lm_prot_t::LM_PROT_XW as isize)?;
    exports.set("lm_prot_t", lm_prot_t)?;

    lua.globals().set("mem", exports)?;

    Ok(())
}

/// LM_AllocMemory(size: usize, isize)
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
fn alloc_memory(lua: &Lua, args: (usize, isize)) -> LuaResult<Value> {
    let address = LM_AllocMemory(args.0, try_into_lm_prot_t(args.1)?).into_lua(lua)?;
    Ok(address)
}

/// LM_AllocMemoryEx(size: usize, isize)
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
fn alloc_memory_ex(lua: &Lua, args: (LmProcessT, usize, isize)) -> LuaResult<Value> {
    let address =
        LM_AllocMemoryEx(&args.0 .0, args.1, try_into_lm_prot_t(args.2)?).into_lua(lua)?;
    Ok(address)
}

/// LM_Assemble(code: &str)
/// Assembles a single instruction into machine code.
///
/// Parameters:
/// code: a string of the instruction to be assembled. Example: "jmp eax".
///
/// Return:
/// On success, it returns Some(instruction), where instruction is a valid lm_inst_t containing the assembled instruction. On failure, it returns None (nil).
///
/// https://github.com/rdbo/libmem/blob/4.4.0/docs/api/rust/LM_AllocMemory.md
fn assemble(lua: &Lua, args: (String,)) -> LuaResult<Value> {
    LM_Assemble(&args.0).map(LmInstT).into_lua(lua)
}

/// LM_AssembleEx(code: &str, usize, usize)
/// Assembles a single instruction into machine code.
///
/// Parameters:
/// code: a string of the instructions to be assembled. Example: "mov eax, ebx ; jmp eax".
/// bits: the bits of the architecture to be assembled. It can be 32 or 64.
/// runtime_addr: the runtime address to resolve the functions (for example, relative jumps will be resolved using this address).
///
/// Return:
/// On success, it returns Some(instructions), where instructions is a vector of bytes containing the assembled instructions. On failure, it returns None (nil).
///
/// https://github.com/rdbo/libmem/blob/4.4.0/docs/api/rust/LM_AllocMemory.md
fn assemble_ex(lua: &Lua, args: (String, lm_size_t, lm_address_t)) -> LuaResult<Value> {
    LM_AssembleEx(&args.0, args.1, args.2).into_lua(lua)
}

/// LM_CodeLength(usize, usize)
/// Gets the minimum instruction aligned length for minlength bytes from code in the calling process.
///
/// Parameters:
/// code: virtual address of the code to get the minimum aligned length from.
/// minlength: the minimum length to align to an instruction length.
///
/// Return:
/// On success, it returns Some(length), where length is an lm_size_t containing the minimum instruction aligned
/// length for minlength bytes from code. On failure, it returns None.
fn code_length(lua: &Lua, args: (lm_address_t, lm_size_t)) -> LuaResult<Value> {
    // SAFETY: All on scripters end. Read the documentation!
    let result = unsafe { LM_CodeLength(args.0, args.1).into_lua(lua)? };
    Ok(result)
}

/// LM_CodeLengthEx
/// Gets the minimum instruction aligned length for minlength bytes from code in a remote process.
///
/// Parameters:
/// pproc: immutable reference to a valid process to get the aligned length from.
/// code: virtual address of the code to get the minimum aligned length from.
/// minlength: the minimum length to align to an instruction length.
///
/// Return:
/// On success, it returns Some(length), where length is an lm_size_t containing the minimum instruction
/// aligned length for minlength bytes from code. On failure, it returns None.
fn code_length_ex(lua: &Lua, args: (LmProcessT, lm_address_t, lm_size_t)) -> LuaResult<Value> {
    let result = LM_CodeLengthEx(&args.0 .0, args.1, args.2).into_lua(lua)?;
    Ok(result)
}

fn try_into_lm_prot_t(flag: isize) -> LuaResult<lm_prot_t> {
    let flag = match flag {
        0b000 => lm_prot_t::LM_PROT_NONE,
        0b001 => lm_prot_t::LM_PROT_X,
        0b010 => lm_prot_t::LM_PROT_R,
        0b100 => lm_prot_t::LM_PROT_W,
        0b011 => lm_prot_t::LM_PROT_XR,
        0b101 => lm_prot_t::LM_PROT_XW,
        0b110 => lm_prot_t::LM_PROT_RW,
        0b111 => lm_prot_t::LM_PROT_XRW,
        _ => return Err(eyre!("{flag} is not a valid protection flag").into_lua_err()),
    };

    Ok(flag)
}

#[derive(Clone, FromLua)]
struct LmInstT(lm_inst_t);

impl LuaUserData for LmInstT {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("bytes", |_, this| Ok(this.0.get_bytes().to_owned()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("print", |lua, this, ()| {
            let mut var = Variadic::new();
            var.push(format!("lm_inst_t: {}", this.0).into_lua(lua)?);
            lua_print(lua, var)?;
            Ok(())
        });
    }
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
        methods.add_method("print", |lua, this, ()| {
            let mut var = Variadic::new();
            var.push(format!("{}", this.0).into_lua(lua)?);
            lua_print(lua, var)?;
            Ok(())
        });
    }
}
