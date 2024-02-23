use eyre::Result;
use libmem::*;
use rhai::plugin::*;
use rhai::{Blob, Locked, Shared};

use crate::{into_usize, IntoRhaiError};

pub fn register(engine: &Engine) -> Result<()> {
    Ok(())
}

#[allow(non_camel_case_types)]
#[export_module]
mod mem_mod {
    pub type prot_t = lm_prot_t;
    pub type process_t = lm_process_t;
    pub type thread_t = lm_thread_t;
    pub type module_t = lm_module_t;
    pub type symbol_t = Shared<lm_symbol_t>;
    pub type page_t = lm_page_t;
    pub type inst_t = lm_inst_t;
    pub type vmt_t = Shared<Locked<lm_vmt_t>>;

    #[rhai_mod(name = "prot_t")]
    pub mod prot_t_mod {
        pub const PROT_NONE: prot_t = prot_t::LM_PROT_NONE;
        pub const PROT_R: prot_t = prot_t::LM_PROT_R;
        pub const PROT_RW: prot_t = prot_t::LM_PROT_RW;
        pub const PROT_W: prot_t = prot_t::LM_PROT_W;
        pub const PROT_X: prot_t = prot_t::LM_PROT_X;
        pub const PROT_XR: prot_t = prot_t::LM_PROT_XR;
        pub const PROT_XRW: prot_t = prot_t::LM_PROT_XRW;
        pub const PROT_XW: prot_t = prot_t::LM_PROT_XW;

        #[rhai_fn(global)]
        pub fn to_string(this: &mut prot_t) -> String {
            this.to_string()
        }

        #[rhai_fn(global)]
        pub fn to_debug(this: &mut prot_t) -> String {
            format!("{this:?}")
        }
    }

    pub mod process_t_mod {
        #[rhai_fn(global)]
        pub fn to_string(this: &mut process_t) -> String {
            this.to_string()
        }

        #[rhai_fn(global)]
        pub fn to_debug(this: &mut process_t) -> String {
            format!("{this:?}")
        }

        #[rhai_fn(global, return_raw, get = "bits")]
        pub fn get_bits(
            ctx: NativeCallContext,
            this: &mut process_t,
        ) -> Result<i64, Box<EvalAltResult>> {
            this.get_bits().try_into().into_rhai_pos(ctx.position())
        }

        #[rhai_fn(global, get = "name")]
        pub fn get_name(this: &mut process_t) -> String {
            this.get_name()
        }

        #[rhai_fn(global, get = "path")]
        pub fn get_path(this: &mut process_t) -> String {
            this.get_path()
        }

        #[rhai_fn(global, get = "pid")]
        pub fn get_pid(this: &mut process_t) -> i64 {
            this.get_pid().into()
        }

        #[rhai_fn(global, get = "ppid")]
        pub fn get_ppid(this: &mut process_t) -> i64 {
            this.get_ppid().into()
        }

        #[rhai_fn(global, return_raw, get = "start_time")]
        pub fn get_start_time(
            ctx: NativeCallContext,
            this: &mut process_t,
        ) -> Result<i64, Box<EvalAltResult>> {
            // originally this is u64 in seconds, and rhai native number type is i64
            // but i64::MAX is ~2262, so we have time before it hits max.
            // nevertheless, we will error out if it goes over
            this.get_start_time()
                .try_into()
                .into_rhai_pos(ctx.position())
        }
    }

    pub mod thread_t_mod {
        #[rhai_fn(global)]
        pub fn to_string(this: &mut thread_t) -> String {
            this.to_string()
        }

        #[rhai_fn(global)]
        pub fn to_debug(this: &mut thread_t) -> String {
            format!("{this:?}")
        }

        #[rhai_fn(global, get = "tid")]
        pub fn get_tid(this: &mut thread_t) -> i64 {
            this.get_tid().into()
        }
    }

    pub mod module_t_mod {
        #[rhai_fn(global)]
        pub fn to_string(this: &mut module_t) -> String {
            this.to_string()
        }

        #[rhai_fn(global)]
        pub fn to_debug(this: &mut module_t) -> String {
            format!("{this:?}")
        }

        #[rhai_fn(global, return_raw, get = "base")]
        pub fn get_base(
            ctx: NativeCallContext,
            this: &mut module_t,
        ) -> Result<i64, Box<EvalAltResult>> {
            this.get_base().try_into().into_rhai_pos(ctx.position())
        }

        #[rhai_fn(global, return_raw, get = "end")]
        pub fn get_end(
            ctx: NativeCallContext,
            this: &mut module_t,
        ) -> Result<i64, Box<EvalAltResult>> {
            this.get_end().try_into().into_rhai_pos(ctx.position())
        }

        #[rhai_fn(global, return_raw, get = "size")]
        pub fn get_size(
            ctx: NativeCallContext,
            this: &mut module_t,
        ) -> Result<i64, Box<EvalAltResult>> {
            this.get_size().try_into().into_rhai_pos(ctx.position())
        }

        #[rhai_fn(global, get = "path")]
        pub fn get_path(this: &mut module_t) -> String {
            this.get_path()
        }

        #[rhai_fn(global, get = "name")]
        pub fn get_name(this: &mut module_t) -> String {
            this.get_name()
        }
    }

    pub mod symbol_t_mod {
        #[rhai_fn(global)]
        pub fn to_string(this: &mut symbol_t) -> String {
            this.to_string()
        }

        #[rhai_fn(global)]
        pub fn to_debug(this: &mut symbol_t) -> String {
            format!("{this:?}")
        }

        #[rhai_fn(global, get = "name")]
        pub fn get_name(this: &mut symbol_t) -> String {
            this.get_name().clone()
        }

        #[rhai_fn(global, return_raw, get = "address")]
        pub fn get_address(
            ctx: NativeCallContext,
            this: &mut symbol_t,
        ) -> Result<i64, Box<EvalAltResult>> {
            this.get_address().try_into().into_rhai_pos(ctx.position())
        }
    }

    pub mod page_t_mod {
        #[rhai_fn(global)]
        pub fn to_string(this: &mut page_t) -> String {
            this.to_string()
        }

        #[rhai_fn(global)]
        pub fn to_debug(this: &mut page_t) -> String {
            format!("{this:?}")
        }

        #[rhai_fn(global, return_raw, get = "base")]
        pub fn get_base(
            ctx: NativeCallContext,
            this: &mut page_t,
        ) -> Result<i64, Box<EvalAltResult>> {
            this.get_base().try_into().into_rhai_pos(ctx.position())
        }

        #[rhai_fn(global, return_raw, get = "end")]
        pub fn get_end(
            ctx: NativeCallContext,
            this: &mut page_t,
        ) -> Result<i64, Box<EvalAltResult>> {
            this.get_end().try_into().into_rhai_pos(ctx.position())
        }

        #[rhai_fn(global, return_raw, get = "size")]
        pub fn get_size(
            ctx: NativeCallContext,
            this: &mut page_t,
        ) -> Result<i64, Box<EvalAltResult>> {
            this.get_size().try_into().into_rhai_pos(ctx.position())
        }

        #[rhai_fn(global, get = "prot")]
        pub fn get_prot(this: &mut page_t) -> prot_t {
            this.get_prot()
        }
    }

    pub mod inst_t_mod {
        #[rhai_fn(global)]
        pub fn to_string(this: &mut inst_t) -> String {
            this.to_string()
        }

        #[rhai_fn(global)]
        pub fn to_debug(this: &mut inst_t) -> String {
            format!("{this:?}")
        }

        #[rhai_fn(global, get = "bytes")]
        pub fn get_bytes(this: &mut inst_t) -> Blob {
            this.get_bytes().into()
        }
    }

    #[rhai_mod(name = "vmt_t")]
    pub mod vmt_t_mod {
        #[rhai_fn(global)]
        pub fn to_string(this: &mut vmt_t) -> String {
            this.borrow().to_string()
        }

        #[rhai_fn(global)]
        pub fn to_debug(this: &mut vmt_t) -> String {
            format!("{:?}", &*this.borrow())
        }

        #[rhai_fn(volatile)]
        pub fn new(vtable: i64) -> vmt_t {
            Shared::new(Locked::new(lm_vmt_t::new(vtable as *mut _)))
        }

        #[rhai_fn(global, return_raw)]
        pub fn hook(
            ctx: NativeCallContext,
            this: &mut vmt_t,
            index: i64,
            dst: i64,
        ) -> Result<(), Box<EvalAltResult>> {
            let index = into_usize(index, ctx.position())?;
            let dst = into_usize(dst, ctx.position())?;

            let mut this = this.try_borrow_mut().into_rhai_pos(ctx.position())?;

            unsafe {
                this.hook(index, dst);
            }

            Ok(())
        }

        #[rhai_fn(global, return_raw)]
        pub fn unhook(
            ctx: NativeCallContext,
            this: &mut vmt_t,
            index: i64,
        ) -> Result<(), Box<EvalAltResult>> {
            let index = into_usize(index, ctx.position())?;

            let mut this = this.try_borrow_mut().into_rhai_pos(ctx.position())?;

            unsafe {
                this.unhook(index);
            }

            Ok(())
        }

        #[rhai_fn(global, return_raw)]
        pub fn get_original(
            ctx: NativeCallContext,
            this: &mut vmt_t,
            index: i64,
        ) -> Result<Dynamic, Box<EvalAltResult>> {
            let index = into_usize(index, ctx.position())?;

            let this = this.try_borrow().into_rhai_pos(ctx.position())?;

            let original = unsafe { this.get_original(index) };

            let original: Option<i64> = original
                .map(|o| o.try_into().into_rhai_pos(ctx.position()))
                .transpose()?;

            Ok(original.map(Dynamic::from).unwrap_or(Dynamic::UNIT))
        }

        #[rhai_fn(global)]
        pub fn reset(this: &mut vmt_t) {
            unsafe { this.borrow_mut().reset() }
        }
    }

    /// alloc_memory(size: i64, prot: prot_t)
    /// Allocates size bytes of memory with protection flags prot in the calling process.
    ///
    /// Parameters:
    /// size: the size of the region to change the protection flags.
    /// prot: the protection flags (PROT_*).
    ///
    /// Return:
    /// On success, it returns address, where address is a valid i64. On failure, it returns unit.
    ///
    /// https://github.com/rdbo/libmem/blob/4.4.0/docs/api/rust/LM_AllocMemory.md
    #[rhai_fn(volatile, return_raw)]
    pub fn alloc_memory(
        ctx: NativeCallContext,
        size: i64,
        prot: prot_t,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        let size = into_usize(size, ctx.position())?;

        let Some(address) = LM_AllocMemory(size, prot) else {
            return Ok(Dynamic::UNIT);
        };

        let address: i64 = address.try_into().into_rhai_pos(ctx.position())?;

        Ok(address.into())
    }

    /// alloc_memory_ex(pproc: process_t, size: i64, prot: prot_t)
    /// Allocates size bytes of memory with protection flags prot in a remote process.
    ///
    /// Parameters:
    /// pproc: immutable pointer to a process which will have memory be allocated.
    /// size: the size of the region to change the protection flags.
    /// prot: the protection flags (LM_PROT_*).
    ///
    /// Return:
    /// On success, it returns address, where address is a valid lm_address_t (i64). On failure, it returns unit.
    ///
    /// https://github.com/rdbo/libmem/blob/4.4.0/docs/api/rust/LM_AllocMemory.md
    fn alloc_memory_ex(
        ctx: NativeCallContext,
        pproc: lm_process_t,
        size: i64,
        prot: prot_t,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        let size = into_usize(size, ctx.position())?;

        let address: Option<i64> = LM_AllocMemoryEx(&pproc, size, prot)
            .map(|n| n.try_into().into_rhai_pos(ctx.position()))
            .transpose()?;

        Ok(address.map(Dynamic::from).unwrap_or(Dynamic::UNIT))
    }

    /// assemble(code: &str)
    /// Assembles a single instruction into machine code.
    ///
    /// Parameters:
    /// code: a string of the instruction to be assembled. Example: "jmp eax".
    ///
    /// Return:
    /// On success, it returns a valid inst_t containing the assembled instruction. On failure, it returns unit.
    ///
    /// https://github.com/rdbo/libmem/blob/4.4.0/docs/api/rust/LM_AllocMemory.md
    fn assemble(code: &str) -> Dynamic {
        LM_Assemble(code)
            .map(Dynamic::from)
            .unwrap_or(Dynamic::UNIT)
    }

    // /// LM_AssembleEx(code: &str, usize, usize)
    // /// Assembles a single instruction into machine code.
    // ///
    // /// Parameters:
    // /// code: a string of the instructions to be assembled. Example: "mov eax, ebx ; jmp eax".
    // /// bits: the bits of the architecture to be assembled. It can be 32 or 64.
    // /// runtime_addr: the runtime address to resolve the functions (for example, relative jumps will be resolved using this address).
    // ///
    // /// Return:
    // /// On success, it returns Some(instructions), where instructions is a vector of bytes containing the assembled instructions. On failure, it returns unit.
    // ///
    // /// https://github.com/rdbo/libmem/blob/4.4.0/docs/api/rust/LM_AllocMemory.md
    // fn assemble_ex(lua: &Lua, args: (String, lm_size_t, lm_address_t)) -> LuaResult<Value> {
    //     LM_AssembleEx(&args.0, args.1, args.2).into_lua(lua)
    // }

    // /// LM_CodeLength(usize, usize)
    // /// Gets the minimum instruction aligned length for minlength bytes from code in the calling process.
    // ///
    // /// Parameters:
    // /// code: virtual address of the code to get the minimum aligned length from.
    // /// minlength: the minimum length to align to an instruction length.
    // ///
    // /// Return:
    // /// On success, it returns Some(length), where length is an lm_size_t containing the minimum instruction aligned
    // /// length for minlength bytes from code. On failure, it returns None.
    // fn code_length(lua: &Lua, args: (lm_address_t, lm_size_t)) -> LuaResult<Value> {
    //     // SAFETY: All on scripters end. Read the documentation!
    //     let result = unsafe { LM_CodeLength(args.0, args.1).into_lua(lua)? };
    //     Ok(result)
    // }

    // /// LM_CodeLengthEx
    // /// Gets the minimum instruction aligned length for minlength bytes from code in a remote process.
    // ///
    // /// Parameters:
    // /// pproc: immutable reference to a valid process to get the aligned length from.
    // /// code: virtual address of the code to get the minimum aligned length from.
    // /// minlength: the minimum length to align to an instruction length.
    // ///
    // /// Return:
    // /// On success, it returns Some(length), where length is an lm_size_t containing the minimum instruction
    // /// aligned length for minlength bytes from code. On failure, it returns None.
    // fn code_length_ex(lua: &Lua, args: (LmProcessT, lm_address_t, lm_size_t)) -> LuaResult<Value> {
    //     let result = LM_CodeLengthEx(&args.0 .0, args.1, args.2).into_lua(lua)?;
    //     Ok(result)
    // }
}
