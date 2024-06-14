use std::{hint::unreachable_unchecked, mem::MaybeUninit, sync::OnceLock};

use cranelift::prelude::{codegen::ir::UserFuncName, isa::CallConv, *};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module as _};
use mem::iat::IATSymbol;
use rustpython_vm::{
    convert::ToPyObject,
    prelude::{PyObjectRef, PyResult, VirtualMachine},
};
use tracing::info;

use super::{args::ArgMemory, cffi::VTableHook, jit_wrapper::JITWrapper, ret::Ret, types::Type};

#[derive(Debug)]
pub enum Hook {
    // regular jmp hook
    Jmp(mem::hook::Trampoline),
    // import address table hook
    #[allow(clippy::upper_case_acronyms)]
    IAT(IATSymbol),
    // vtable index hook
    Vmt(VTableHook),
    // no hook, just call this address plz
    Addr(*const u8),
}

#[derive(Debug)]
pub struct Trampoline {
    hook: Hook,
    arg_mem: ArgMemory,
    args: (Vec<Type>, Type),
    _jit: OnceLock<JITWrapper>,
    conv: CallConv,
    jit_call: OnceLock<extern "fastcall" fn(*const (), *mut Ret)>,
}

unsafe impl Send for Trampoline {}
unsafe impl Sync for Trampoline {}

impl Trampoline {
    pub fn new(hook: Hook, args: (&[Type], Type), conv: CallConv) -> PyResult<Self> {
        let arg_mem = ArgMemory::new(args.0);

        let slf = Self {
            hook,
            arg_mem,
            conv,
            args: (args.0.to_vec(), args.1),
            _jit: OnceLock::new(),
            jit_call: OnceLock::new(),
        };

        Ok(slf)
    }

    pub fn call(&self, args: &[PyObjectRef], vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        self.arg_mem.fill(args, vm)?;

        let fn_ = if let Some(&_fn) = self.jit_call.get() {
            _fn
        } else {
            let _fn = self.compile()?;
            _ = self.jit_call.set(_fn);

            _fn
        };

        let mut ret = MaybeUninit::<Ret>::uninit();
        fn_(self.arg_mem.mem(), ret.as_mut_ptr());

        // we have nothing to write in the void case
        if matches!(self.args.1, Type::Void) {
            return Ok(None::<()>.to_pyobject(vm));
        }

        let ret = unsafe { ret.assume_init().to_pyobject(self.args.1, vm) };

        Ok(ret)
    }

    /// Compile the jit trampoline wrapper
    fn compile(&self) -> PyResult<extern "fastcall" fn(*const (), *mut Ret)> {
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

        for &arg in &self.args.0 {
            let ty: types::Type = arg.into();
            tp_sig_fn.params.push(AbiParam::new(ty));
        }

        if !matches!(self.args.1, Type::Void) {
            tp_sig_fn.returns.push(AbiParam::new(self.args.1.into()));
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

        // raw args - arg_mem, ret
        sig_fn.params.push(AbiParam::new(types::I64));
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

        let arg_memory = params[0];
        let ret = params[1];

        let mut arg_values = Vec::new();
        for (&ty, &offset) in self.args.0.iter().zip(self.arg_mem.offsets()) {
            let ty: types::Type = ty.into();

            let value = bcx
                .ins()
                .load(ty, MemFlags::trusted(), arg_memory, offset as i32);

            arg_values.push(value);
        }

        let trampoline_fn = module.declare_func_in_func(trampoline_fn, bcx.func);
        let call = bcx.ins().call(trampoline_fn, &arg_values);

        // only write to return memory if it's not void
        if !matches!(self.args.1, Type::Void) {
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
        let _fn = unsafe {
            std::mem::transmute::<*const u8, extern "fastcall" fn(*const (), *mut Ret)>(code)
        };

        info!("defined {jitpoline_name}() ({code:?}) -> {tramp_name}()");

        _ = self._jit.set(JITWrapper::new(module));

        Ok(_fn)
    }
}
