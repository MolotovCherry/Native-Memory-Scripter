use std::{
    alloc::{self, Layout},
    cell::OnceCell,
    hint::unreachable_unchecked,
    mem::{self, MaybeUninit},
    sync::OnceLock,
};

use cranelift::prelude::{codegen::ir::UserFuncName, isa::CallConv, *};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module as _};
use mutation::{hook::Trampoline, iat::IATSymbol, memory};
use rustpython_vm::{
    convert::ToPyObject,
    prelude::{PyObjectRef, PyResult, VirtualMachine},
};
use tracing::{info, trace, trace_span};

use crate::modules::Address;

use self::codegen::ir::ArgumentPurpose;
use super::{args::ArgMemory, cffi::VTableHook, jit::JitWrapper, ret::Ret, types::Type};

#[derive(Debug)]
pub enum Hook {
    // regular jmp hook
    Jmp(Trampoline),
    // import address table hook
    #[allow(clippy::upper_case_acronyms)]
    IAT(IATSymbol),
    // vtable index hook
    Vmt(VTableHook),
    // no hook, just call this address plz
    Addr(*const u8),
}

impl Hook {
    pub fn trampoline_address(&self) -> Address {
        match self {
            Hook::Jmp(t) => t.address as _,
            _ => unreachable!(),
        }
    }

    pub fn trampoline_size(&self) -> usize {
        match self {
            Hook::Jmp(t) => t.size,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Jitpoline {
    hook: Hook,
    arg_mem: ArgMemory,
    sret_mem: Option<(*mut u8, Layout)>,
    args: (Vec<Type>, Type),
    _jit: OnceLock<JitWrapper>,
    conv: CallConv,
    jitpoline: OnceLock<extern "fastcall" fn(*mut ())>,
}

unsafe impl Send for Jitpoline {}
unsafe impl Sync for Jitpoline {}

impl Drop for Jitpoline {
    fn drop(&mut self) {
        if let Some((mem, layout)) = self.sret_mem.take() {
            unsafe {
                alloc::dealloc(mem, layout);
            }
        }
    }
}

impl Jitpoline {
    pub fn new(hook: Hook, args: (&[Type], Type), conv: CallConv) -> PyResult<Self> {
        let span = trace_span!("jitpoline");
        let _guard = span.enter();

        trace!(?hook, ?args, ?conv, "new");

        let arg_mem = ArgMemory::new(args.0);

        let sret_mem = if let Type::Struct(size) = args.1 {
            assert!(size > 0, "size is 0");

            // 8 byte aligned data
            let align = size
                .max(8)
                .checked_next_power_of_two()
                .expect("align overflowed");

            let layout =
                unsafe { Layout::from_size_align_unchecked(size as usize, align as usize) };

            let alloc = unsafe { alloc::alloc(layout) };

            Some((alloc, layout))
        } else {
            None
        };

        let slf = Self {
            hook,
            arg_mem,
            sret_mem,
            conv,
            args: (args.0.to_vec(), args.1),
            _jit: OnceLock::new(),
            jitpoline: OnceLock::new(),
        };

        Ok(slf)
    }

    pub unsafe fn call(&self, args: &[PyObjectRef], vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        let span = trace_span!("jitpoline");
        let mut _guard = span.enter();

        self.arg_mem.fill(args, vm)?;

        let jitpoline = if let Some(&jitpoline) = self.jitpoline.get() {
            jitpoline
        } else {
            drop(_guard);
            let jitpoline = self.compile()?;
            _ = self.jitpoline.set(jitpoline);
            _guard = span.enter();

            jitpoline
        };

        let ret = if let Type::Struct(size) = self.args.1 {
            let (mem, _) = unsafe { self.sret_mem.unwrap_unchecked() };

            drop(_guard);
            jitpoline(mem.cast());
            _guard = span.enter();

            let data = unsafe { memory::read_bytes(mem.cast(), size as _) };
            data.to_pyobject(vm)
        } else {
            let mut ret = MaybeUninit::<Ret>::uninit();

            drop(_guard);
            jitpoline(ret.as_mut_ptr().cast());
            _guard = span.enter();

            // we have nothing to write in the void case
            if self.args.1.is_void() {
                return Ok(None::<()>.to_pyobject(vm));
            }

            unsafe { ret.assume_init().to_pyobject(self.args.1, vm) }
        };

        Ok(ret)
    }

    pub unsafe fn unhook(&self, vm: &VirtualMachine) -> PyResult<()> {
        let span = trace_span!("jitpoline");
        let _guard = span.enter();

        match &self.hook {
            Hook::Jmp(j) => {
                let res = unsafe { j.unhook() };
                res.map_err(|e| vm.new_runtime_error(e.to_string()))?;
            }

            Hook::IAT(i) => {
                let res = unsafe { i.unhook() };
                res.map_err(|e| vm.new_runtime_error(e.to_string()))?;
            }

            Hook::Vmt(v) => {
                let res = unsafe { v.unhook(v.index()) };
                res.map_err(|e| vm.new_runtime_error(e.to_string()))?;
            }

            // direct addr, no unhooking to do
            Hook::Addr(_) => (),
        }

        Ok(())
    }

    pub fn trampoline_address(&self) -> Address {
        self.hook.trampoline_address()
    }

    pub fn trampoline_size(&self) -> usize {
        self.hook.trampoline_size()
    }

    pub fn jitpoline_address(&self) -> Option<Address> {
        self.jitpoline.get().map(|&f| f as Address)
    }

    /// Compile the jit trampoline wrapper
    fn compile(&self) -> PyResult<extern "fastcall" fn(*mut ())> {
        let span = trace_span!("jitpoline");
        let _guard = span.enter();

        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "true").unwrap();
        flag_builder.set("enable_float", "true").unwrap();
        flag_builder
            .set("enable_llvm_abi_extensions", "true")
            .unwrap();
        flag_builder.set("enable_jump_tables", "true").unwrap();
        flag_builder.set("opt_level", "speed").unwrap();

        let hook_address = match &self.hook {
            Hook::Jmp(h) => h.address,
            Hook::IAT(i) => i.orig_fn as _,
            Hook::Vmt(v) => v.get_original(v.index()).unwrap().cast(),
            Hook::Addr(ptr) => *ptr,
        };

        let tramp_name = format!("__trampoline_{:?}", hook_address);
        let jitpoline_name = format!("__jitpoline_{:?}", hook_address);

        // SAFETY: We are always on a supported platform. Win x64;
        let isa_builder = cranelift_native::builder_with_options(true)
            .unwrap_or_else(|_| unsafe { unreachable_unchecked() });

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        let mut builder = JITBuilder::with_isa(isa, default_libcall_names());
        builder.symbol(&tramp_name, hook_address);
        let mut module = JITModule::new(builder);

        let mut ctx = module.make_context();
        let mut func_ctx = FunctionBuilderContext::new();

        //
        // trampoline fn
        //

        let mut tp_sig_fn = module.make_signature();
        tp_sig_fn.call_conv = self.conv;

        if !self.args.1.is_void() {
            match self.conv {
                CallConv::WindowsFastcall if self.args.1.is_struct_indirect() => {
                    let arg = AbiParam::special(self.args.1.into(), ArgumentPurpose::StructReturn);
                    tp_sig_fn.params.push(arg);
                }

                _ => tp_sig_fn.returns.push(AbiParam::new(self.args.1.into())),
            }
        }

        for &arg in &self.args.0 {
            let arg = AbiParam::new(arg.into());
            tp_sig_fn.params.push(arg);
        }

        // declare and import fn
        let trampoline_fn = module
            .declare_function(&tramp_name, Linkage::Import, &tp_sig_fn)
            .unwrap();

        //
        // jit function details
        //

        let mut sig_fn = module.make_signature();
        sig_fn.call_conv = CallConv::WindowsFastcall;

        // raw args - ret
        sig_fn.params.push(AbiParam::new(types::I64));

        let func = module
            .declare_function(&jitpoline_name, Linkage::Local, &sig_fn)
            .unwrap();

        ctx.func.signature = sig_fn;
        ctx.func.name = UserFuncName::user(0, func.as_u32());

        //
        // create jit function
        //

        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

        let ebb = bcx.create_block();
        bcx.append_block_params_for_function_params(ebb);

        bcx.switch_to_block(ebb);
        let params = bcx.block_params(ebb);

        let ret = params[0];

        let mut arg_values = Vec::new();

        // add return ptr as first arg if we're using sret
        if self.args.1.is_struct_indirect() {
            arg_values.push(ret);
        }

        // base arg_mem address
        let arg_mem = OnceCell::new();

        for (&ty, &offset) in self.args.0.iter().zip(self.arg_mem.offsets()) {
            let value = if ty.is_struct_indirect() {
                if offset == 0 {
                    *arg_mem.get_or_init(|| bcx.ins().iconst(types::I64, self.arg_mem.mem() as i64))
                } else {
                    let arg_mem = (self.arg_mem.mem() as usize) + offset;
                    bcx.ins().iconst(types::I64, arg_mem as i64)
                }
            } else {
                let mem = *arg_mem
                    .get_or_init(|| bcx.ins().iconst(types::I64, self.arg_mem.mem() as i64));

                bcx.ins()
                    .load(ty.into(), MemFlags::trusted(), mem, offset as i32)
            };

            arg_values.push(value);
        }

        let trampoline_fn = module.declare_func_in_func(trampoline_fn, bcx.func);
        let call = bcx.ins().call(trampoline_fn, &arg_values);

        // only write to return memory if it's not void
        if !self.args.1.is_void() && !self.args.1.is_struct_indirect() {
            let res = bcx.inst_results(call)[0];
            bcx.ins().store(MemFlags::trusted(), res, ret, 0);
        }

        bcx.ins().return_(&[]);

        //
        // Finish blocks and finalization
        //

        bcx.seal_all_blocks();
        bcx.finalize();

        module.define_function(func, &mut ctx).unwrap();
        module.clear_context(&mut ctx);

        // Perform linking.
        module.finalize_definitions().unwrap();

        // Get a raw pointer to the generated code.
        let code = module.get_finalized_function(func);
        let _fn = unsafe { mem::transmute::<*const u8, extern "fastcall" fn(*mut ())>(code) };

        info!("defined {jitpoline_name}() ({code:?}) -> {tramp_name}()");

        _ = self._jit.set(JitWrapper::new(module));

        Ok(_fn)
    }
}
