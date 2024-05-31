use rustpython_vm::pymodule;

#[derive(Debug)]
struct Sendable<T: std::fmt::Debug>(*mut T);
unsafe impl<T: std::fmt::Debug> Send for Sendable<T> {}
unsafe impl<T: std::fmt::Debug> Sync for Sendable<T> {}

#[allow(clippy::module_inception)]
#[pymodule]
pub mod cffi {
    use std::{
        fmt::Formatter,
        sync::{Arc, Mutex},
    };

    use cranelift::prelude::{isa::CallConv, types::Type as CType, *};
    use cranelift_jit::JITModule;
    use rustpython_vm::{
        builtins::PyTypeRef,
        function::FuncArgs,
        prelude::{PyRefExact, VirtualMachine, *},
        pyclass, pymodule,
        types::Constructor,
        vm::thread::ThreadedVirtualMachine,
        PyPayload,
    };

    use super::Sendable;

    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(name)]
    #[derive(Debug, PyPayload)]
    struct Callable {
        name: String,
        vm: Arc<Mutex<PyThreadedVirtualMachine>>,
        obj: PyObjectRef,
        jit: Arc<Mutex<Option<JITWrapper>>>,
        // Args, Ret types
        params: Arc<(Vec<CType>, CType)>,
        // leaked memory for the callback
        leaked: Arc<Mutex<Option<Sendable<Self>>>>,
    }

    impl Clone for Callable {
        fn clone(&self) -> Self {
            Self {
                name: self.name.clone(),
                vm: self.vm.clone(),
                obj: self.obj.clone(),
                jit: self.jit.clone(),
                params: self.params.clone(),
                leaked: self.leaked.clone(),
            }
        }
    }

    impl Constructor for Callable {
        type Args = (PyObjectRef, FuncArgs);

        fn py_new(_cls: PyTypeRef, args: Self::Args, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            // only callables are allowed
            if !args.0.is_callable() {
                return Err(vm.new_type_error("object is not callable".to_owned()));
            }

            let calling_conv = args
                .1
                .get_kwarg("conv", _call_conv::WindowsFastcall.into_ref(&vm.ctx).into());

            let calling_conv = calling_conv
                .downcast_exact::<PyCallConv>(vm)
                .map_err(|_| vm.new_type_error("conv expected CallConv".to_owned()))?;

            let ret = args
                .1
                .get_kwarg("ret", PyType(Type::Void).into_ref(&vm.ctx).into());

            let ret = ret
                .downcast_exact::<PyType>(vm)
                .map_err(|_| vm.new_type_error("ret expected Type".to_owned()))?;

            let name = args.0.class().__name__(vm).to_string();

            let fn_args = args
                .1
                .args
                .into_iter()
                .map(|a| {
                    a.downcast_exact::<PyType>(vm).map_err(|s| {
                        vm.new_type_error(format!(
                            "expected Type, found {}",
                            s.class().__name__(vm)
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let callable = jit_c_wrapper(&name, fn_args, ***ret, calling_conv.0, vm)?;

            Ok(callable.into_pyobject(vm))
        }
    }

    #[pyclass(with(Constructor))]
    impl Callable {
        /// SAFETY:
        /// Ensure that no C code will ever call this function ever again
        /// Never calling this will leak memory
        #[pymethod]
        fn free_memory(&self) {
            let mut lock = self.jit.lock().unwrap();
            if let Some(jit) = lock.take() {
                unsafe {
                    jit.0.free_memory();
                }
            }

            let mut lock = self.leaked.lock().unwrap();
            if let Some(callable) = lock.take() {
                _ = unsafe { Box::from_raw(callable.0) };
            }
        }
    }

    /// Wrapper to get debug since ThreadedVirtualMachine didn't impl it
    struct PyThreadedVirtualMachine(ThreadedVirtualMachine);
    impl std::fmt::Debug for PyThreadedVirtualMachine {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "ThreadedVirtualMachine")
        }
    }

    #[pyclass(no_attr, name = "Type")]
    #[derive(Debug, Copy, Clone, PyPayload)]
    struct PyType(Type);

    #[derive(Debug, Copy, Clone)]
    enum Type {
        Void,

        // Floats
        F32(CType),
        F64(CType),

        // Unsigned
        U8(CType),
        U16(CType),
        U32(CType),
        U64(CType),
        U128(CType),

        // Integers
        I8(CType),
        I16(CType),
        I32(CType),
        I64(CType),
        I128(CType),

        // Pointer
        Ptr32(CType),
        Ptr(CType),

        // Bool
        Bool(CType),

        // Strings
        // c str (null terminated) - r64
        Str(CType),
        // utf16 str (null terminated) - r64
        WStr(CType),

        // Characters
        // i8
        Char(CType),
        // i16
        WChar(CType),
    }

    #[pyclass]
    impl PyType {
        #[pymethod(magic)]
        fn repr(&self) -> String {
            format!("Type.{:?}", self.0)
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            format!("Type.{:?}", self.0)
        }
    }

    impl From<Type> for CType {
        fn from(val: Type) -> Self {
            match val {
                Type::F32(t)
                | Type::F64(t)
                | Type::U8(t)
                | Type::U16(t)
                | Type::U32(t)
                | Type::U64(t)
                | Type::U128(t)
                | Type::I8(t)
                | Type::I16(t)
                | Type::I32(t)
                | Type::I64(t)
                | Type::I128(t)
                | Type::Ptr32(t)
                | Type::Ptr(t)
                | Type::Bool(t)
                | Type::Str(t)
                | Type::WStr(t)
                | Type::Char(t)
                | Type::WChar(t) => t,

                _ => unreachable!("invalid type"),
            }
        }
    }

    #[allow(non_upper_case_globals)]
    #[pymodule(name = "Type")]
    pub mod _type {
        use super::*;

        #[pyattr]
        const Void: PyType = PyType(Type::Void);

        #[pyattr]
        const F32: PyType = PyType(Type::F32(types::F32));
        #[pyattr]
        const F64: PyType = PyType(Type::F64(types::F64));

        #[pyattr]
        const U8: PyType = PyType(Type::U8(types::I8));
        #[pyattr]
        const U16: PyType = PyType(Type::U16(types::I16));
        #[pyattr]
        const U32: PyType = PyType(Type::U32(types::I32));
        #[pyattr]
        const U64: PyType = PyType(Type::U64(types::I64));
        #[pyattr]
        const U128: PyType = PyType(Type::U128(types::I128));

        #[pyattr]
        const I8: PyType = PyType(Type::I8(types::I8));
        #[pyattr]
        const I16: PyType = PyType(Type::I16(types::I16));
        #[pyattr]
        const I32: PyType = PyType(Type::I32(types::I32));
        #[pyattr]
        const I64: PyType = PyType(Type::I64(types::I64));
        #[pyattr]
        const I128: PyType = PyType(Type::I128(types::I128));

        #[pyattr]
        const Ptr32: PyType = PyType(Type::Ptr32(types::R32));
        #[pyattr]
        const Ptr: PyType = PyType(Type::Ptr(types::R64));

        #[pyattr]
        const Bool: PyType = PyType(Type::Bool(types::I8));

        #[pyattr]
        const Str: PyType = PyType(Type::Str(types::R64));
        #[pyattr]
        const WStr: PyType = PyType(Type::WStr(types::R64));

        #[pyattr]
        const Char: PyType = PyType(Type::Char(types::I8));
        #[pyattr]
        const WChar: PyType = PyType(Type::WChar(types::I16));
    }

    #[pyclass(no_attr, name = "CallConv")]
    #[derive(Debug, Copy, Clone, PyPayload)]
    struct PyCallConv(CallConv);

    #[pyclass()]
    impl PyCallConv {
        #[pymethod(magic)]
        fn repr(&self) -> String {
            format!("{:?}", self.0)
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            format!("{:?}", self.0)
        }
    }

    #[allow(non_upper_case_globals)]
    #[pymodule(name = "CallConv")]
    pub mod _call_conv {
        use super::*;

        /// defaults to WindowsFastcall
        #[pyattr]
        pub(super) const C: PyCallConv = PyCallConv(CallConv::WindowsFastcall);

        /// same as cdecl on Windows
        #[pyattr]
        pub(super) const WindowsFastcall: PyCallConv = PyCallConv(CallConv::WindowsFastcall);
    }

    struct JITWrapper(JITModule);
    impl std::fmt::Debug for JITWrapper {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "JITModule")
        }
    }

    // generate a c wrapper according to specs
    fn jit_c_wrapper(
        name: &str,
        args: Vec<PyRefExact<PyType>>,
        ret: PyType,
        call_conv: CallConv,
        vm: &VirtualMachine,
    ) -> PyResult<Callable> {
        use cranelift_codegen::ir::UserFuncName;
        use cranelift_jit::{JITBuilder, JITModule};
        use cranelift_module::{default_libcall_names, Linkage, Module};
        use std::{hint::unreachable_unchecked, mem};

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
        let mut module = JITModule::new(JITBuilder::with_isa(isa, default_libcall_names()));

        let mut ctx = module.make_context();
        let mut func_ctx = FunctionBuilderContext::new();

        let mut sig_fn = module.make_signature();

        sig_fn.call_conv = call_conv;

        for arg in args {
            if matches!(ret.0, Type::Void) {
                return Err(vm.new_type_error("Void is not a valid argument type".to_owned()));
            }

            sig_fn.params.push(AbiParam::new(arg.0.into()));
        }

        if !matches!(ret.0, Type::Void) {
            sig_fn.returns.push(AbiParam::new(ret.0.into()));
        }

        let func = module
            .declare_function(name, Linkage::Local, &sig_fn)
            .unwrap();

        ctx.func.signature = sig_fn;
        ctx.func.name = UserFuncName::user(0, func.as_u32());

        {
            let mut bcx: FunctionBuilder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            let block = bcx.create_block();

            bcx.switch_to_block(block);
            bcx.append_block_params_for_function_params(block);
            let param = bcx.block_params(block)[0];
            let cst = bcx.ins().iconst(types::I32, 37);
            let add = bcx.ins().iadd(cst, param);
            bcx.ins().return_(&[add]);
            bcx.seal_all_blocks();
            bcx.finalize();
        }
        module.define_function(func, &mut ctx).unwrap();
        module.clear_context(&mut ctx);

        // Perform linking.
        module.finalize_definitions().unwrap();

        // Get a raw pointer to the generated code.
        let code = module.get_finalized_function(func);

        // Cast it to a rust function pointer type.
        let ptr_b = unsafe { mem::transmute::<_, extern "C" fn() -> u32>(code) };

        // Call it!
        let res = ptr_b();

        println!("{res}");

        todo!()
    }
}
