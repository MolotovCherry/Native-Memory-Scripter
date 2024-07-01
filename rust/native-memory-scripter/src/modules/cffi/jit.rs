mod callback;

use std::{hint::unreachable_unchecked, sync::OnceLock};

use cranelift::prelude::{codegen::ir::UserFuncName, isa::CallConv, *};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module as _};
use mem::{
    memory::{Alloc, MemError},
    Prot,
};
use rustpython_vm::prelude::*;
use rustpython_vm::vm::thread::ThreadedVirtualMachine;
use tracing::{info, trace_span};

use crate::{
    modules::{cffi::ret::RetMemory, Address},
    utils::RawSendable,
};

use self::{callback::__jit_cb, codegen::ir::ArgumentPurpose};
use super::{args::ArgLayout, types::Type};

pub struct JitWrapper(OnceLock<JITModule>);

impl JitWrapper {
    pub fn new(module: JITModule) -> Self {
        Self(OnceLock::from(module))
    }
}

impl std::fmt::Debug for JitWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Jit")
    }
}

impl Drop for JitWrapper {
    fn drop(&mut self) {
        if let Some(jit) = self.0.take() {
            unsafe {
                jit.free_memory();
            }
        }
    }
}

struct DataWrapper(OnceLock<RawSendable<Data>>);

impl DataWrapper {
    fn new(data: *mut Data) -> Self {
        Self(OnceLock::from(RawSendable::new(data)))
    }
}

impl std::fmt::Debug for DataWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Data")
    }
}

impl Drop for DataWrapper {
    fn drop(&mut self) {
        if let Some(data) = self.0.take() {
            unsafe {
                drop(Box::from_raw(data.as_ptr()));
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

#[derive(Debug)]
pub struct Jit {
    _jit: JitWrapper,
    _data: DataWrapper,
    address: *const u8,
    size: u32,
    jit_alloc: OnceLock<Alloc>,
    _ret_mem: Option<RetMemory>,
}

unsafe impl Send for Jit {}
unsafe impl Sync for Jit {}

impl Jit {
    // generate a c wrapper according to specs
    pub fn new(
        name: &str,
        obj: PyObjectRef,
        args: (&[Type], Type),
        call_conv: CallConv,
        vm: &VirtualMachine,
    ) -> PyResult<Self> {
        let span = trace_span!("jit");
        let _guard = span.enter();

        if args.0.iter().any(|i| i.is_void()) {
            return Err(vm.new_type_error("Void cannot be used as an arg".to_owned()));
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
        // tip: Jacob Lifshay + bjorn3
        if !args.1.is_void() {
            match call_conv {
                CallConv::WindowsFastcall if args.1.is_struct_indirect() => {
                    let arg = AbiParam::special(args.1.into(), ArgumentPurpose::StructReturn);
                    sig_fn.params.push(arg);
                }

                _ => sig_fn.returns.push(AbiParam::new(args.1.into())),
            }
        }

        for arg in args.0.iter().copied() {
            let arg = AbiParam::new(arg.into());
            sig_fn.params.push(arg);
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

        let mut ret_mem = if !args.1.is_struct_indirect() && !args.1.is_void() {
            Some(RetMemory::new())
        } else {
            None
        };

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
            let ret_addr = if args.1.is_struct_indirect() {
                // sret return
                vals.remove(0)
            } else if args.1.is_void() {
                // null ptr. we don't use it, so no need to do anything
                bcx.ins().iconst(types::I64, 0)
            } else {
                // regular return
                bcx.ins()
                    .iconst(types::I64, ret_mem.as_mut().unwrap().mem() as i64)
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

            if args.1.is_void() || args.1.is_struct_indirect() {
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
        let data = DataWrapper::new(leaked_data as *mut _);

        let slf = Self {
            _jit: JitWrapper::new(module),
            _data: data,
            address: code,
            size: code_size,
            jit_alloc: OnceLock::new(),
            _ret_mem: ret_mem,
        };

        Ok(slf)
    }

    // try to alloc within ± 2gb of address and jmp to actual jit code
    // this usually allows us to write only 5 bytes for a jmp
    pub fn alloc_near(&self, address: Address) -> Result<(), MemError> {
        if self.jit_alloc.get().is_some() {
            return Ok(());
        }

        let gb = 1024 * 1024 * 1024;

        let begin = address.saturating_sub(gb * 2);
        let end = address.saturating_add(gb * 2);

        #[rustfmt::skip]
        let mut jmp64 = [
            // jmp [rip]
            0xFF, 0x25, 0x00, 0x00, 0x00, 0x00,
            // addr
            0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90,
        ];

        jmp64[6..].copy_from_slice(&(self.address as usize).to_le_bytes());

        let alloc = mem::memory::alloc_in(begin as _, end as _, jmp64.len(), 0, Prot::XRW)?;

        unsafe {
            mem::memory::write_bytes(&jmp64, alloc.addr());
        }

        unsafe {
            mem::memory::prot(alloc.addr() as _, jmp64.len(), Prot::XR)?;
        }

        self.jit_alloc.set(alloc).unwrap();

        Ok(())
    }

    pub fn alloc_address(&self) -> Option<*const u8> {
        // must set alloc_jit_near first
        self.jit_alloc.get().map(|e| e.addr() as *const _)
    }

    pub fn address(&self) -> *const u8 {
        self.address
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}
