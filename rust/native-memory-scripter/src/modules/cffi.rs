mod args;
mod jit_wrapper;
mod ret;
pub mod trampoline;
mod types;

use rustpython_vm::pymodule;

#[allow(clippy::module_inception)]
#[pymodule]
pub mod cffi {
    use std::{
        ops::Deref,
        sync::{Arc, Mutex},
    };

    use cranelift::prelude::{isa::CallConv, *};
    use rustpython_vm::{
        builtins::PyTypeRef,
        convert::ToPyObject,
        function::FuncArgs,
        prelude::{PyObjectRef, VirtualMachine, *},
        pyclass, pymodule,
        types::Constructor,
        PyPayload,
    };

    use super::{
        jit_wrapper::{jit_py_wrapper, DataWrapper},
        trampoline::{Hook, Trampoline},
        types::Type,
    };
    use crate::modules::{
        iat::iat::PyIATSymbol, symbols::symbols::PySymbol, vmt::vmt::PyVTable, Address,
    };

    #[pyattr]
    #[pyclass(name)]
    #[derive(Debug, Clone, PyPayload)]
    pub struct NativeCall {
        trampoline: Arc<Trampoline>,
        lock: Arc<Mutex<()>>,
    }

    #[pyclass(with(Constructor))]
    impl NativeCall {
        #[pymethod]
        fn call(&self, args: FuncArgs, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let _lock = self.lock.lock().unwrap();
            unsafe { self.trampoline.call(&args.args, vm) }
        }
    }

    impl Constructor for NativeCall {
        type Args = (PyObjectRef, FuncArgs);

        fn py_new(_cls: PyTypeRef, args: Self::Args, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            // usize address or symbol is OK for first param
            let address = if let Ok(addr) = args.0.try_to_value::<usize>(vm) {
                Some(addr)
            } else if let Ok(sym) = args.0.downcast_exact::<PySymbol>(vm) {
                Some(sym.address())
            } else {
                None
            };

            let Some(address) = address else {
                return Err(vm.new_runtime_error(
                    "first param must be a usize address or Symbol".to_owned(),
                ));
            };

            let calling_conv = args
                .1
                .get_kwarg("conv", _call_conv::WindowsFastcall.into_pyobject(vm));

            let call_conv = calling_conv
                .downcast_exact::<PyCallConv>(vm)
                .map_err(|_| vm.new_type_error("conv expected CallConv".to_owned()))?;
            let calling_conv = ****call_conv;

            let ret = args
                .1
                .get_kwarg("ret", PyType(Type::Void).into_pyobject(vm));

            let ret = ret
                .downcast_exact::<PyType>(vm)
                .map_err(|_| vm.new_type_error("ret expected Type".to_owned()))?;
            let ret = (***ret).0;

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

            let hook = Hook::Addr(address as _);
            let trampoline = Trampoline::new(hook, (&fn_args, ret), calling_conv)?;

            let call = Self {
                trampoline: Arc::new(trampoline),
                lock: Arc::default(),
            };

            Ok(call.to_pyobject(vm))
        }
    }

    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(name)]
    #[derive(Debug, Clone, PyPayload)]
    pub struct Callable {
        addr: *const u8,
        code_size: u32,
        trampoline: Arc<Mutex<Option<Trampoline>>>,
        params: (Vec<Type>, Type),
        call_conv: CallConv,
        #[allow(clippy::type_complexity)]
        _cb_mem: DataWrapper,
    }

    unsafe impl Send for Callable {}
    unsafe impl Sync for Callable {}

    impl Constructor for Callable {
        type Args = (PyObjectRef, FuncArgs);

        fn py_new(_cls: PyTypeRef, args: Self::Args, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            // only callables are allowed
            if !args.0.is_callable() {
                return Err(vm.new_type_error("object is not callable".to_owned()));
            }

            let calling_conv = args
                .1
                .get_kwarg("conv", _call_conv::WindowsFastcall.into_pyobject(vm));

            let call_conv = calling_conv
                .downcast_exact::<PyCallConv>(vm)
                .map_err(|_| vm.new_type_error("conv expected CallConv".to_owned()))?;
            let calling_conv = ****call_conv;

            let ret = args
                .1
                .get_kwarg("ret", PyType(Type::Void).into_pyobject(vm));

            let ret = ret
                .downcast_exact::<PyType>(vm)
                .map_err(|_| vm.new_type_error("ret expected Type".to_owned()))?;
            let ret = (***ret).0;

            let name = args.0.get_attr("__name__", vm)?.str(vm)?.to_string();

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

            let (module, address, code_size) =
                jit_py_wrapper(&name, args.0, (&fn_args, ret), calling_conv, vm)?;

            let callable = Callable {
                addr: address,
                code_size,
                params: (fn_args, ret),
                _cb_mem: module,
                trampoline: Arc::default(),
                call_conv: calling_conv,
            };

            Ok(callable.into_pyobject(vm))
        }
    }

    #[pyclass(with(Constructor))]
    impl Callable {
        #[pygetset]
        pub fn addr(&self) -> Address {
            self.addr as _
        }

        #[pygetset]
        fn code_size(&self) -> u32 {
            self.code_size
        }

        #[pymethod]
        fn hook(&self, from: Address, vm: &VirtualMachine) -> PyResult<bool> {
            let mut lock = self.trampoline.lock().unwrap();
            if lock.is_some() {
                return Err(vm.new_runtime_error(
                    "this callable is already hooking something. create a new callable to hook something else"
                        .to_owned(),
                ));
            }

            let res = unsafe { mem::hook::hook(from as _, self.addr) };
            let trampoline = res.map_err(|e| vm.new_runtime_error(format!("{e}")))?;

            let hook = Hook::Jmp(trampoline);
            let trampoline =
                Trampoline::new(hook, (&self.params.0, self.params.1), self.call_conv)?;

            *lock = Some(trampoline);

            Ok(true)
        }

        #[pymethod]
        fn hook_iat(&self, entry: PyRef<PyIATSymbol>, vm: &VirtualMachine) -> PyResult<bool> {
            let mut lock = self.trampoline.lock().unwrap();
            if lock.is_some() {
                return Err(vm.new_runtime_error(
                    "this callable is already hooking something. create a new callable to hook something else"
                        .to_owned(),
                ));
            }

            let res = unsafe { entry.hook(self.addr.cast()) };
            res.map_err(|e| vm.new_runtime_error(e.to_string()))?;

            let hook = Hook::IAT((**entry).clone());
            let trampoline =
                Trampoline::new(hook, (&self.params.0, self.params.1), self.call_conv)?;

            *lock = Some(trampoline);

            Ok(true)
        }

        #[pymethod]
        fn hook_vmt(
            &self,
            vtable: PyRef<PyVTable>,
            index: usize,
            vm: &VirtualMachine,
        ) -> PyResult<bool> {
            let mut lock = self.trampoline.lock().unwrap();
            if lock.is_some() {
                return Err(vm.new_runtime_error(
                    "this callable is already hooking something. create a new callable to hook something else"
                        .to_owned(),
                ));
            }

            let res = unsafe { vtable.hook(index, self.addr as _) };
            res.map_err(|e| vm.new_runtime_error(e.to_string()))?;

            let hook = Hook::Vmt(VTableHook(index, vtable));
            let trampoline =
                Trampoline::new(hook, (&self.params.0, self.params.1), self.call_conv)?;

            *lock = Some(trampoline);

            Ok(true)
        }

        /// unsafe fn
        #[pymethod]
        fn unhook(&self, vm: &VirtualMachine) -> PyResult<()> {
            let lock = self.trampoline.lock().unwrap();
            if let Some(trampoline) = &*lock {
                unsafe {
                    trampoline.unhook(vm)?;
                }
            }

            Ok(())
        }

        #[pymethod]
        fn call_trampoline(&self, args: FuncArgs, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let lock = self.trampoline.lock().unwrap();

            let Some(trampoline) = &*lock else {
                return Err(vm.new_runtime_error(
                    "cannot call trampoline because no function was hooked".to_owned(),
                ));
            };

            unsafe { trampoline.call(&args.args, vm) }
        }
    }

    //
    // VTableHook
    // This will auto-unhook the index when dropped. Won't affect drop of the actual vtable itself
    // since the vtable is refcounted
    //
    #[derive(Debug)]
    pub struct VTableHook(usize, PyRef<PyVTable>);

    impl VTableHook {
        pub fn index(&self) -> usize {
            self.0
        }
    }

    impl Deref for VTableHook {
        type Target = PyRef<PyVTable>;

        fn deref(&self) -> &Self::Target {
            &self.1
        }
    }

    impl Drop for VTableHook {
        fn drop(&mut self) {
            let _ = unsafe { self.1.unhook(self.0) };
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
        const Ptr: PyType = PyType(Type::Ptr(types::I64));

        #[pyattr]
        const Bool: PyType = PyType(Type::Bool(types::I8));

        #[pyattr]
        const CStr: PyType = PyType(Type::CStr(types::I64));
        #[pyattr]
        const WStr: PyType = PyType(Type::WStr(types::I64));

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
