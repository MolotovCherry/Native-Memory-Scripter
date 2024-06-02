use std::{
    alloc::{Layout, LayoutError},
    hint::unreachable_unchecked,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::{Arc, Mutex},
};

use cranelift::prelude::{isa::CallConv, *};
use cranelift_codegen::ir::UserFuncName;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module as _};
use rustpython_vm::prelude::*;
use tracing::error;

use crate::modules::cffi::vm::PyThreadedVirtualMachine;

use super::{cffi::Callable, types::Type, RawSendable};

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

extern "fastcall" fn __jit_cb(args: *const (), data: &Callable) {
    let iter = data.layout.iter(args);
    for arg in iter {
        println!("got arg {arg:?}");
    }

    todo!()
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

    if !matches!(args.1, Type::Void) {
        cb_sig_fn.returns.push(AbiParam::new(ret.into())); // same return type as wrapped fn
    }

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
            args_layout.size(),
            args_layout.align(),
        );

        let slot = bcx.create_sized_stack_slot(slot_data);

        let ebb = bcx.create_block();
        bcx.append_block_params_for_function_params(ebb);

        bcx.switch_to_block(ebb);
        let vals = bcx.block_params(ebb).to_vec();
        for (val, offset) in vals
            .iter()
            .copied()
            .zip(args_layout.offsets.iter().copied())
        {
            bcx.ins().stack_store(val, slot, offset as i32);
        }

        let leaked_addr = bcx.ins().iconst(types::I64, leaked_data as *const _ as i64);
        let stack_addr = bcx.ins().stack_addr(types::R64, slot, 0);

        let cb = module.declare_func_in_func(jit_callback, bcx.func);
        let params = &[stack_addr, leaked_addr];
        let call = bcx.ins().call(cb, params);
        let res = bcx.inst_results(call).to_vec();

        // return fn with same data as cb
        bcx.ins().return_(&res);

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

#[derive(Debug, Clone)]
enum Arg {
    // Floats
    F32(f32),
    F64(f64),

    // Unsigned
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    // Integers
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    // Pointer
    Ptr(*const ()),

    // Bool
    Bool(bool),

    // Strings
    // c str (null terminated) - r64
    CStr(*const i8),
    // utf16 str - r64 (length unknown)
    WStr(*const u16),

    // Characters
    // i8
    Char(char),
    // i16
    WChar(char),
}

#[derive(Debug, Clone)]
pub struct ArgLayout {
    args: Vec<Type>,
    offsets: Vec<usize>,
    size: u32,
    align: u8,
}

impl ArgLayout {
    fn new(args: &[Type]) -> Result<Self, LayoutError> {
        let (layout, offsets) = Self::layout(args)?;

        Ok(Self {
            size: layout.size() as _,
            args: args.to_vec(),
            offsets,
            align: layout.align() as _,
        })
    }

    fn size(&self) -> u32 {
        self.size
    }

    fn align(&self) -> u8 {
        self.align
    }

    // Get the layout and offsets for arg list
    fn layout(args: &[Type]) -> Result<(Layout, Vec<usize>), LayoutError> {
        let mut offsets = Vec::new();
        let mut layout = unsafe { Layout::from_size_align_unchecked(0, 1) };
        for &field in args {
            let size = field.size();

            let field = unsafe { Layout::from_size_align_unchecked(size, size) };

            let (new_layout, offset) = layout.extend(field)?;
            layout = new_layout;

            offsets.push(offset);
        }

        Ok((layout.pad_to_align(), offsets))
    }

    fn iter(&self, ptr: *const ()) -> ArgLayoutIterator {
        ArgLayoutIterator {
            ptr,
            layout: self,
            pos: 0,
        }
    }
}

struct ArgLayoutIterator<'a> {
    ptr: *const (),
    layout: &'a ArgLayout,
    pos: usize,
}

impl<'a> Iterator for ArgLayoutIterator<'a> {
    type Item = Arg;

    fn next(&mut self) -> Option<Self::Item> {
        let ty = self.layout.args.get(self.pos)?;
        let offset = self.layout.offsets[self.pos];

        let arg = match ty {
            Type::Void => {
                error!("bug! args cannot be void!");
                return None;
            }

            Type::F32(_) => {
                let arg = unsafe { *self.ptr.cast::<f32>().add(offset) };
                Arg::F32(arg)
            }

            Type::F64(_) => {
                let arg = unsafe { *self.ptr.cast::<f64>().add(offset) };
                Arg::F64(arg)
            }

            Type::U8(_) => {
                let arg = unsafe { *self.ptr.cast::<u8>().add(offset) };
                Arg::U8(arg)
            }

            Type::U16(_) => {
                let arg = unsafe { *self.ptr.cast::<u16>().add(offset) };
                Arg::U16(arg)
            }

            Type::U32(_) => {
                let arg = unsafe { *self.ptr.cast::<u32>().add(offset) };
                Arg::U32(arg)
            }

            Type::U64(_) => {
                let arg = unsafe { *self.ptr.cast::<u64>().add(offset) };
                Arg::U64(arg)
            }

            Type::U128(_) => {
                let arg = unsafe { *self.ptr.cast::<u128>().add(offset) };
                Arg::U128(arg)
            }

            Type::I8(_) => {
                let arg = unsafe { *self.ptr.cast::<i8>().add(offset) };
                Arg::I8(arg)
            }

            Type::I16(_) => {
                let arg = unsafe { *self.ptr.cast::<i16>().add(offset) };
                Arg::I16(arg)
            }

            Type::I32(_) => {
                let arg = unsafe { *self.ptr.cast::<i32>().add(offset) };
                Arg::I32(arg)
            }

            Type::I64(_) => {
                let arg = unsafe { *self.ptr.cast::<i64>().add(offset) };
                Arg::I64(arg)
            }

            Type::I128(_) => {
                let arg = unsafe { *self.ptr.cast::<i128>().add(offset) };
                Arg::I128(arg)
            }

            Type::Ptr(_) => {
                let arg = unsafe { *self.ptr.cast::<*const ()>().add(offset) };
                Arg::Ptr(arg)
            }

            Type::Bool(_) => {
                let arg = unsafe { *self.ptr.cast::<bool>().add(offset) };
                Arg::Bool(arg)
            }

            Type::CStr(_) => {
                let ptr = unsafe { *self.ptr.cast::<*const i8>().add(offset) };
                Arg::CStr(ptr)
            }

            Type::WStr(_) => {
                let ptr = unsafe { *self.ptr.cast::<*const u16>().add(offset) };
                Arg::WStr(ptr)
            }

            Type::Char(_) => {
                let arg = unsafe { *self.ptr.cast::<u8>().add(offset) };
                Arg::Char(arg as char)
            }

            Type::WChar(_) => {
                let arg = unsafe { *self.ptr.cast::<u16>().add(offset) };
                Arg::WChar(unsafe { char::from_u32_unchecked(arg as u32) })
            }
        };

        Some(arg)
    }
}
