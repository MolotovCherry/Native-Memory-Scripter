mod args;
mod jit_wrapper;
mod ret;
pub mod trampoline;
mod types;
mod vm;

use rustpython_vm::pymodule;

#[allow(clippy::module_inception)]
#[pymodule]
pub mod cffi {
    use std::{
        ops::Deref,
        ptr::NonNull,
        sync::{Arc, Mutex},
    };

    use cranelift::prelude::{isa::CallConv, *};
    use rustpython_vm::{
        builtins::PyTypeRef,
        function::FuncArgs,
        prelude::{PyObjectRef, VirtualMachine, *},
        pyclass, pymodule,
        types::Constructor,
        PyPayload,
    };

    use super::{
        jit_wrapper::{jit_py_wrapper, Data, JITWrapper},
        trampoline::Trampoline,
        types::Type,
    };
    use crate::utils::RawSendable;

    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(name)]
    #[derive(Debug, Clone, PyPayload)]
    pub struct Callable {
        addr: usize,
        code_size: u32,
        trampoline: Arc<Mutex<Trampoline>>,
        #[allow(clippy::type_complexity)]
        cb_mem: Arc<Mutex<Option<(JITWrapper, RawSendable<Data>)>>>,
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

            let (module, leaked_data, address, code_size) =
                jit_py_wrapper(&name, args.0, (&fn_args, ret), calling_conv, vm)?;

            let trampoline = Trampoline::new(address, (&fn_args, ret), vm)?;

            let callable = Callable {
                addr: address,
                code_size,

                cb_mem: Arc::new(Mutex::new(Some((
                    JITWrapper(module),
                    RawSendable(unsafe { NonNull::new_unchecked(leaked_data) }),
                )))),

                trampoline: Arc::new(Mutex::new(trampoline)),
            };

            Ok(callable.into_pyobject(vm))
        }
    }

    #[pyclass(with(Constructor))]
    impl Callable {
        #[pygetset]
        pub fn addr(&self) -> usize {
            self.addr
        }

        #[pygetset]
        fn code_size(&self) -> u32 {
            self.code_size
        }

        #[pymethod]
        fn call(&self, args: FuncArgs, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let lock = self.trampoline.lock().unwrap();
            lock.call(&args.args, vm)
        }
    }

    impl Drop for Callable {
        fn drop(&mut self) {
            if let Ok(mut lock) = self.cb_mem.lock() {
                if let Some((jit, leaked)) = lock.take() {
                    unsafe {
                        jit.0.free_memory();
                        _ = Box::from_raw(leaked.0.as_ptr());
                    }
                }
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

        /// systemv
        #[pyattr]
        pub(super) const SystemV: PyCallConv = PyCallConv(CallConv::SystemV);
    }
}
