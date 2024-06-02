mod jit;
mod types;
mod vm;

use std::fmt::Debug;
use std::ptr::NonNull;

use rustpython_vm::pymodule;

#[derive(Debug, Copy, Clone)]
pub struct RawSendable<T: Debug>(NonNull<T>);
unsafe impl<T: Debug> Send for RawSendable<T> {}
unsafe impl<T: Debug> Sync for RawSendable<T> {}

#[allow(clippy::module_inception)]
#[pymodule]
pub mod cffi {
    use std::{
        ops::Deref,
        sync::{Arc, Mutex},
    };

    use cranelift::prelude::{isa::CallConv, types::Type as CType, *};
    use rustpython_vm::{
        builtins::PyTypeRef, function::FuncArgs, prelude::*, pyclass, pymodule, types::Constructor,
        PyPayload,
    };

    use super::{
        jit::{jit_py_wrapper, ArgLayout, JITWrapper},
        types::Type,
        vm::PyThreadedVirtualMachine,
    };

    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(name)]
    #[derive(Debug, Clone, PyPayload)]
    pub struct Callable {
        pub vm: Arc<Mutex<PyThreadedVirtualMachine>>,
        pub py_cb: PyObjectRef,
        pub jit: Arc<Mutex<Option<JITWrapper>>>,
        // Args, Ret types
        pub params: Arc<(Vec<CType>, Option<CType>)>,
        pub layout: ArgLayout,
        // leaked memory for the callback
        pub leaked: Arc<Mutex<Option<super::RawSendable<Self>>>>,
        pub fn_addr: super::RawSendable<()>,
        pub code_size: u32,
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

            let call_conv = calling_conv
                .downcast_exact::<PyCallConv>(vm)
                .map_err(|_| vm.new_type_error("conv expected CallConv".to_owned()))?;
            let calling_conv = ****call_conv;

            let ret = args
                .1
                .get_kwarg("ret", PyType(Type::Void).into_ref(&vm.ctx).into());

            let ret = ret
                .downcast_exact::<PyType>(vm)
                .map_err(|_| vm.new_type_error("ret expected Type".to_owned()))?;
            let ret = (***ret).0;

            let name = args.0.class().__name__(vm).to_string();

            let fn_args = args
                .1
                .args
                .into_iter()
                .map(|a| {
                    a.downcast_exact::<PyType>(vm).map(|t| ****t).map_err(|s| {
                        vm.new_type_error(format!(
                            "expected Type, found {}",
                            s.class().__name__(vm)
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let callable = jit_py_wrapper(&name, args.0, (fn_args, ret), calling_conv, vm)?;

            Ok(callable.into_pyobject(vm))
        }
    }

    #[pyclass(with(Constructor))]
    impl Callable {
        #[pygetset]
        fn addr(&self) -> usize {
            self.fn_addr.0.as_ptr() as _
        }

        #[pygetset]
        fn code_size(&self) -> u32 {
            self.code_size
        }

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
                _ = unsafe { Box::from_raw(callable.0.as_ptr()) };
            }
        }
    }

    //
    // Type
    //

    #[pyclass(no_attr, name = "Type")]
    #[derive(Debug, Copy, Clone, PyPayload)]
    struct PyType(Type);

    impl Deref for PyType {
        type Target = Type;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
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
        const I8: PyType = PyType(Type::I8(types::I8));
        #[pyattr]
        const I16: PyType = PyType(Type::I16(types::I16));
        #[pyattr]
        const I32: PyType = PyType(Type::I32(types::I32));
        #[pyattr]
        const I64: PyType = PyType(Type::I64(types::I64));

        #[pyattr]
        const Ptr: PyType = PyType(Type::Ptr(types::R64));

        #[pyattr]
        const Bool: PyType = PyType(Type::Bool(types::I8));

        #[pyattr]
        const CStr: PyType = PyType(Type::CStr(types::R64));
        #[pyattr]
        const WStr: PyType = PyType(Type::WStr(types::R64));

        #[pyattr]
        const Char: PyType = PyType(Type::Char(types::I8));
        #[pyattr]
        const WChar: PyType = PyType(Type::WChar(types::I16));
    }

    //
    // CallConv
    //

    #[pyclass(no_attr, name = "CallConv")]
    #[derive(Debug, Copy, Clone, PyPayload)]
    struct PyCallConv(CallConv);

    impl Deref for PyCallConv {
        type Target = CallConv;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

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
}
