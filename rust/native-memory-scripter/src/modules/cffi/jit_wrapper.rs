mod callback;

use std::{
    hint::unreachable_unchecked,
    mem,
    sync::{Arc, Mutex},
};

use cranelift::prelude::{codegen::ir::UserFuncName, isa::CallConv, *};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module as _};
use rustpython_vm::prelude::*;
use rustpython_vm::vm::thread::ThreadedVirtualMachine;
use tracing::info;

use crate::utils::RawSendable;

use self::{callback::__jit_cb, codegen::ir::ArgumentPurpose};
use super::{args::ArgLayout, ret::Ret, types::Type};

#[derive(Clone)]
pub struct JITWrapper(Arc<Mutex<Option<JITModule>>>);

impl JITWrapper {
    pub fn new(module: JITModule) -> Self {
        Self(Arc::new(Mutex::new(Some(module))))
    }
}

impl std::fmt::Debug for JITWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JITWrapper")
    }
}

impl Drop for JITWrapper {
    fn drop(&mut self) {
        let count = Arc::strong_count(&self.0);
        // only drop if this is the last clone
        if count == 1 {
            let mut lock = self.0.lock().unwrap();
            if let Some(jit) = lock.take() {
                unsafe {
                    jit.free_memory();
                }
            }
        }
    }
}

#[allow(clippy::complexity)]
#[derive(Clone)]
pub struct DataWrapper(Arc<Mutex<Option<(JITModule, RawSendable<Data>)>>>);

impl DataWrapper {
    fn new(module: JITModule, data: *mut Data) -> Self {
        Self(Arc::new(Mutex::new(Some((module, RawSendable::new(data))))))
    }
}

impl std::fmt::Debug for DataWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Data")
    }
}

impl Drop for DataWrapper {
    fn drop(&mut self) {
        let count = Arc::strong_count(&self.0);
        // only drop if this is the last clone
        if count == 1 {
            let mut lock = self.0.lock().unwrap();
            if let Some((jit, data)) = lock.take() {
                unsafe {
                    jit.free_memory();
                    drop(Box::from_raw(data.as_ptr()));
                }
            }
        }
    }
}

struct Data {
    vm: ThreadedVirtualMachine,
    callable: PyObjectRef,
    params: (Vec<Type>, Type),
    layout: Option<ArgLayout>,
}

// generate a c wrapper according to specs
pub fn jit_py_wrapper(
    name: &str,
    obj: PyObjectRef,
    args: (&[Type], Type),
    call_conv: CallConv,
    vm: &VirtualMachine,
) -> PyResult<(DataWrapper, *const u8, u32)> {
    // arguments cannot be StructReturn
    if args.0.iter().any(|i| i.is_sret()) {
        return Err(vm.new_type_error("StructReturn cannot be used as an arg".to_owned()));
    }

    // arguments cannot be StructReturn
    if args.0.iter().any(|i| i.is_void()) {
        return Err(vm.new_type_error("Void cannot be used as an arg".to_owned()));
    }

    // return cannot be StructArg
    if args.1.is_sarg() {
        return Err(vm.new_type_error("StructArg cannot be used as a return value".to_owned()));
    }

    let mut flag_builder = settings::builder();
    flag_builder.set("use_colocated_libcalls", "false").unwrap();
    flag_builder.set("is_pic", "true").unwrap();
    flag_builder.set("enable_float", "true").unwrap();
    flag_builder
        .set("enable_llvm_abi_extensions", "true")
        .unwrap();
    flag_builder.set("enable_jump_tables", "true").unwrap();
    flag_builder.set("opt_level", "speed").unwrap();

    let jit_cb_name = format!("__jit_cb_{name}");
    let name = format!("__jit_native_py_{name}");

    // SAFETY: We are always on a supported platform. Win x64;
    let isa_builder = cranelift_native::builder_with_options(true)
        .unwrap_or_else(|_| unsafe { unreachable_unchecked() });

    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .unwrap();

    let mut builder = JITBuilder::with_isa(isa, default_libcall_names());
    // add external function to symbols
    builder.symbol(&jit_cb_name, __jit_cb as *const u8);
    let mut module = JITModule::new(builder);

    let mut ctx = module.make_context();
    let mut func_ctx = FunctionBuilderContext::new();

    let mut sig_fn = module.make_signature();

    sig_fn.call_conv = call_conv;

    // the reason this is placed first is because sret MUST be placed first
    if !args.1.is_void() {
        match args.1 {
            Type::StructReturn(_) => {
                let arg = AbiParam::special(args.1.into(), ArgumentPurpose::StructReturn);
                sig_fn.params.push(arg);
            }

            _ => {
                sig_fn.returns.push(AbiParam::new(args.1.into()));
            }
        }
    }

    for arg in args.0.iter().copied() {
        match arg {
            // structs larger than 64bits are passed by-reference rather than at a fixed stack offset
            // so we have to use a ptr if > 64-bits, sarg otherwise. Thanks bjorn3!
            Type::StructArg(_) => {
                // the from impl automatically adjusts the used type based on size
                sig_fn.params.push(AbiParam::new(arg.into()));
            }

            _ => {
                sig_fn.params.push(AbiParam::new(arg.into()));
            }
        }
    }

    //
    // jit callback
    //

    let mut cb_sig_fn = module.make_signature();
    cb_sig_fn.call_conv = CallConv::WindowsFastcall;
    cb_sig_fn.params.push(AbiParam::new(types::I64));
    cb_sig_fn.params.push(AbiParam::new(types::I64));
    cb_sig_fn.params.push(AbiParam::new(types::I64));

    // declare and import fn

    let jit_callback = module
        .declare_function(&jit_cb_name, Linkage::Import, &cb_sig_fn)
        .unwrap();

    //
    // create data and leak it
    //

    let args_layout = ArgLayout::new(args.0);

    let data = Data {
        vm: vm.new_thread(),
        callable: obj,
        params: (args.0.to_vec(), args.1),
        layout: args_layout.clone(),
    };

    // since we cloned this, we clone the dangling pointer, but we don't need this address anyways
    let leaked_data = Box::leak(Box::new(data));

    //
    // jit function
    //

    let func = module
        .declare_function(&name, Linkage::Local, &sig_fn)
        .unwrap();

    ctx.func.signature = sig_fn;
    ctx.func.name = UserFuncName::user(0, func.as_u32());

    {
        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

        let ebb = bcx.create_block();
        bcx.append_block_params_for_function_params(ebb);

        bcx.switch_to_block(ebb);
        let mut vals = bcx.block_params(ebb).to_vec();

        // this special return is an arg. let's place it in the return ptr instead
        let ret_arg = if args.1.is_sret() {
            Some(vals.remove(0))
        } else {
            None
        };

        // for struct
        let arg_slot = if let Some(args_layout) = args_layout {
            let slot_data = StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                args_layout.size() as _,
                args_layout.align() as _,
            );
            let arg_slot = bcx.create_sized_stack_slot(slot_data);

            for (val, offset) in vals
                .iter()
                .copied()
                .zip(args_layout.offsets().iter().copied())
            {
                bcx.ins().stack_store(val, arg_slot, offset as i32);
            }

            Some(arg_slot)
        } else {
            None
        };

        let ret_addr = if args.1.is_void() {
            // null ptr. we don't use it, so no need to do anything
            bcx.ins().iconst(types::I64, 0)
        } else if let Some(ret) = ret_arg {
            ret
        } else {
            let slot_data = StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                mem::size_of::<Ret>() as u32,
                mem::align_of::<Ret>() as u8,
            );
            let ret_slot = bcx.create_sized_stack_slot(slot_data);
            bcx.ins().stack_addr(types::I64, ret_slot, 0)
        };

        let leaked_addr = bcx.ins().iconst(types::I64, leaked_data as *const _ as i64);
        let stack_addr = if let Some(arg_slot) = arg_slot {
            bcx.ins().stack_addr(types::I64, arg_slot, 0)
        } else {
            bcx.ins().iconst(types::I64, 0)
        };

        let cb = module.declare_func_in_func(jit_callback, bcx.func);
        let params = &[stack_addr, leaked_addr, ret_addr];
        let call = bcx.ins().call(cb, params);

        bcx.inst_results(call);

        if args.1.is_void() || args.1.is_sret() {
            // no data returned. in the case of structreturn, we already wrote to the ptr in the function body
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

    info!("defined {name}() ({code:?}) -> {jit_cb_name}()");

    // we cast leaked data to a raw pointer so that a mutable reference does not exist anymore and we can call the callback with &Data
    let data = DataWrapper::new(module, leaked_data as *mut _);
    Ok((data, code, code_size))
}
