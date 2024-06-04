use std::{
    hint::unreachable_unchecked,
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::{Arc, Mutex},
};

use cranelift::prelude::{codegen::ir::UserFuncName, isa::CallConv, *};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module as _};
use rustpython_vm::function::FuncArgs;
use rustpython_vm::prelude::*;

use super::{args::ArgLayout, cffi::Callable, ret::Ret, types::Type};
use crate::{modules::cffi::vm::PyThreadedVirtualMachine, utils::RawSendable};

pub struct JITWrapper(pub JITModule);

impl std::fmt::Debug for JITWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JITModule")
    }
}

impl Deref for JITWrapper {
    type Target = JITModule;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for JITWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

extern "fastcall" fn __jit_cb(args: *const (), data: &Callable, ret: &mut Ret) {
    let vm = &*data.vm.lock().unwrap();

    let result = vm.0.shared_run(|vm| {
        let mut iter = unsafe { data.layout.iter(args) };

        let mut py_args = FuncArgs::default();

        let first = iter.next().unwrap().as_u8();
        let second = iter.next().unwrap().as_u64();

        py_args.prepend_arg(vm.new_pyobj(second));
        py_args.prepend_arg(vm.new_pyobj(first));

        let res = data.py_cb.call_with_args(py_args, vm).unwrap();
        9u32
    });

    *ret = Ret { u32: result };
}

// generate a c wrapper according to specs
pub fn jit_py_wrapper(
    name: &str,
    obj: PyObjectRef,
    args: (Vec<Type>, Type),
    call_conv: CallConv,
    vm: &VirtualMachine,
) -> PyResult<Callable> {
    let mut flag_builder = settings::builder();
    flag_builder.set("use_colocated_libcalls", "false").unwrap();
    flag_builder.set("is_pic", "true").unwrap();
    flag_builder.set("enable_float", "true").unwrap();
    flag_builder
        .set("enable_llvm_abi_extensions", "true")
        .unwrap();
    flag_builder.set("enable_jump_tables", "true").unwrap();
    flag_builder.set("opt_level", "speed").unwrap();

    // SAFETY: We are always on a supported platform. Win x64;
    let isa_builder = cranelift_native::builder_with_options(true)
        .unwrap_or_else(|_| unsafe { unreachable_unchecked() });

    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .unwrap();

    let mut builder = JITBuilder::with_isa(isa, default_libcall_names());
    // add external function to symbols
    builder.symbol("__jit_cb", __jit_cb as *const u8);
    let mut module = JITModule::new(builder);

    let mut ctx = module.make_context();
    let mut func_ctx = FunctionBuilderContext::new();

    let mut sig_fn = module.make_signature();

    sig_fn.call_conv = call_conv;

    for arg in args.0.iter().copied() {
        if matches!(arg, Type::Void) {
            return Err(vm.new_type_error("void is not a valid argument type".to_owned()));
        }

        sig_fn.params.push(AbiParam::new(arg.into()));
    }

    let ret = args.1;

    if !matches!(args.1, Type::Void) {
        sig_fn.returns.push(AbiParam::new(ret.into()));
    }

    //
    // jit callback
    //

    let mut cb_sig_fn = module.make_signature();
    cb_sig_fn.call_conv = CallConv::WindowsFastcall;
    cb_sig_fn.params.push(AbiParam::new(types::R64));
    cb_sig_fn.params.push(AbiParam::new(types::I64));
    cb_sig_fn.params.push(AbiParam::new(types::R64));

    // declare and import link to static_fucntion
    let jit_callback = module
        .declare_function("__jit_cb", Linkage::Import, &cb_sig_fn)
        .unwrap();

    //
    // create callback and leak it
    //

    let module = Arc::new(Mutex::new(Some(JITWrapper(module))));

    let args_layout = ArgLayout::new(&args.0).unwrap();

    let params = (
        args.0.into_iter().map(|ty| ty.into()).collect(),
        if !matches!(args.1, Type::Void) {
            Some(args.1.into())
        } else {
            None
        },
    );

    let mut data = Callable {
        vm: Arc::new(Mutex::new(PyThreadedVirtualMachine(vm.new_thread()))),
        py_cb: obj,
        jit: module.clone(),
        params: Arc::new(params),
        layout: args_layout.clone(),
        leaked: Arc::new(Mutex::new(None)),
        // dangling pointer until we fill this with real pointer
        fn_addr: RawSendable(NonNull::dangling()),
        code_size: 0,
    };

    // since we cloned this, we clone the dangling pointer, but we don't need this address anyways
    let leaked_data = Box::leak(Box::new(data.clone()));

    // leak callback and set a pointer to it inside callable so it can be freed later
    {
        let mut lock = data.leaked.lock().unwrap();
        *lock = Some(RawSendable(NonNull::new(leaked_data).unwrap()));
    }

    let mut module = module.lock().unwrap();
    let module = module.as_mut().unwrap();

    let leaked_data = &*leaked_data;

    //
    // jit function
    //

    let func = module
        .declare_function(&format!("__jit_native_py{name}"), Linkage::Local, &sig_fn)
        .unwrap();

    ctx.func.signature = sig_fn;
    ctx.func.name = UserFuncName::user(0, func.as_u32());

    {
        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

        // for struct
        let slot_data = StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            args_layout.size() as _,
            args_layout.align() as _,
        );
        let arg_slot = bcx.create_sized_stack_slot(slot_data);

        let ebb = bcx.create_block();
        bcx.append_block_params_for_function_params(ebb);

        bcx.switch_to_block(ebb);
        let vals = bcx.block_params(ebb).to_vec();
        for (val, offset) in vals
            .iter()
            .copied()
            .zip(args_layout.offsets().iter().copied())
        {
            bcx.ins().stack_store(val, arg_slot, offset as i32);
        }

        let slot_data = StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            mem::size_of::<Ret>() as u32,
            mem::align_of::<Ret>() as u8,
        );
        let ret_slot = bcx.create_sized_stack_slot(slot_data);
        let ret_addr = bcx.ins().stack_addr(types::R64, ret_slot, 0);

        let leaked_addr = bcx.ins().iconst(types::I64, leaked_data as *const _ as i64);
        let stack_addr = bcx.ins().stack_addr(types::R64, arg_slot, 0);

        let cb = module.declare_func_in_func(jit_callback, bcx.func);
        let params = &[stack_addr, leaked_addr, ret_addr];
        let call = bcx.ins().call(cb, params);

        bcx.inst_results(call);

        if matches!(args.1, Type::Void) {
            // no data returned
            // return fn with same data as cb
            bcx.ins().return_(&[]);
        } else {
            // load data from return stack as the return type we wanted
            let val = bcx
                .ins()
                .load(args.1.into(), MemFlags::trusted(), ret_addr, 0);

            // data is returned
            // return fn with same data as cb
            bcx.ins().return_(&[val]);
        }

        bcx.seal_all_blocks();
        bcx.finalize();
    }

    module.define_function(func, &mut ctx).unwrap();
    let code_size = ctx.compiled_code().unwrap().code_info().total_size;
    module.clear_context(&mut ctx);

    //
    // / jit function
    //

    // Perform linking.
    module.finalize_definitions().unwrap();

    // Get a raw pointer to the generated code.
    let code = module.get_finalized_function(func);

    data.fn_addr = RawSendable(NonNull::new(code as *const _ as *mut _).unwrap());
    data.code_size = code_size;

    Ok(data)
}
