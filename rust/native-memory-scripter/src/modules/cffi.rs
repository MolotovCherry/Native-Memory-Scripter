mod args;
mod jit;
pub mod jitpoline;
mod ret;
mod types;

use rustpython_vm::pymodule;

#[allow(clippy::module_inception)]
#[pymodule]
pub mod cffi {
    use std::{
        ops::Deref,
        ptr,
        sync::{Arc, Mutex},
    };

    use cranelift::prelude::{isa::CallConv, *};
    use rustpython_vm::{
        builtins::PyTypeRef,
        convert::ToPyObject as _,
        function::FuncArgs,
        prelude::{PyObjectRef, VirtualMachine, *},
        pyclass, pymodule,
        types::{Callable, Constructor, Unconstructible},
        PyPayload,
    };
    use tracing::{trace, trace_span};

    use super::{
        jit::Jit,
        jitpoline::{Hook, Jitpoline},
        types::Type,
    };
    use crate::modules::{
        iat::iat::PyIATSymbol, symbols::symbols::PySymbol, vmt::vmt::PyVTable, Address,
    };

    #[pyattr]
    #[pyclass(name)]
    #[derive(Debug, Clone, PyPayload)]
    pub struct NativeCall {
        jitpoline: Arc<Jitpoline>,
        lock: Arc<Mutex<()>>,
    }

    impl Drop for NativeCall {
        fn drop(&mut self) {
            let span = trace_span!("drop");
            let _guard = span.enter();

            let address = self
                .jitpoline
                .jitpoline_address()
                .map(|a| a as *const ())
                .unwrap_or(ptr::null());

            trace!(?address, "dropping NativeCall");
        }
    }

    #[pyclass(with(Constructor, Callable))]
    impl NativeCall {}

    impl Callable for NativeCall {
        type Args = FuncArgs;

        fn call(zelf: &Py<Self>, args: FuncArgs, vm: &VirtualMachine) -> PyResult {
            let _lock = zelf.lock.lock().unwrap();
            unsafe { zelf.jitpoline.call(&args.args, vm) }
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
                .get_kwarg("conv", _conv::WindowsFastcall.into_pyobject(vm));

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
            let jitpoline = Jitpoline::new(hook, (&fn_args, ret), calling_conv)?;

            let call = Self {
                jitpoline: Arc::new(jitpoline),
                lock: Arc::default(),
            };

            Ok(call.to_pyobject(vm))
        }
    }

    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(name = "Callable")]
    #[derive(Debug, PyPayload)]
    pub struct PyCallable {
        jit: Jit,
        jitpoline: Mutex<Option<Jitpoline>>,
        params: (Vec<Type>, Type),
        call_conv: CallConv,
    }

    impl Drop for PyCallable {
        fn drop(&mut self) {
            let span = trace_span!("drop");
            let _guard = span.enter();
            trace!(address = ?self.address() as *const (), "dropping Callable");
        }
    }

    unsafe impl Send for PyCallable {}
    unsafe impl Sync for PyCallable {}

    impl Constructor for PyCallable {
        type Args = (PyObjectRef, FuncArgs);

        fn py_new(_cls: PyTypeRef, args: Self::Args, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            // only callables are allowed
            if !args.0.is_callable() {
                return Err(vm.new_type_error("object is not callable".to_owned()));
            }

            let calling_conv = args
                .1
                .get_kwarg("conv", _conv::WindowsFastcall.into_pyobject(vm));

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

            let jit = Jit::new(&name, args.0, (&fn_args, ret), calling_conv, vm)?;

            let callable = PyCallable {
                jit,
                params: (fn_args, ret),
                jitpoline: Mutex::default(),
                call_conv: calling_conv,
            };

            Ok(callable.into_pyobject(vm))
        }
    }

    impl Callable for PyCallable {
        type Args = FuncArgs;

        fn call(zelf: &Py<Self>, args: FuncArgs, vm: &VirtualMachine) -> PyResult {
            let lock = zelf.jitpoline.lock().unwrap();

            let Some(jitpoline) = &*lock else {
                return Err(vm.new_runtime_error(
                    "cannot call jitpoline because no function was hooked".to_owned(),
                ));
            };

            unsafe { jitpoline.call(&args.args, vm) }
        }
    }

    #[pyclass(with(Constructor, Callable))]
    impl PyCallable {
        #[pygetset]
        pub fn address(&self) -> Address {
            self.jit.address() as _
        }

        #[pygetset]
        fn code_size(&self) -> u32 {
            self.jit.size()
        }

        #[pygetset]
        pub fn trampoline_address(&self) -> Address {
            let lock = self.jitpoline.lock().unwrap();
            lock.as_ref().unwrap().trampoline_address()
        }

        #[pygetset]
        pub fn trampoline_size(&self) -> usize {
            let lock = self.jitpoline.lock().unwrap();
            lock.as_ref().unwrap().trampoline_size()
        }

        #[pygetset]
        pub fn jitpoline_address(&self) -> Option<Address> {
            let lock = self.jitpoline.lock().unwrap();
            lock.as_ref().and_then(|f| f.jitpoline_address())
        }

        #[pymethod]
        fn hook(&self, from: PyObjectRef, vm: &VirtualMachine) -> PyResult<bool> {
            let mut lock = self.jitpoline.lock().unwrap();
            if lock.is_some() {
                return Err(vm.new_runtime_error(
                    "this callable is already hooking something. create a new callable to hook something else"
                        .to_owned(),
                ));
            }

            let address = if let Ok(addr) = from.try_to_value::<Address>(vm) {
                addr
            } else if let Ok(addr) = from.downcast_exact::<PySymbol>(vm) {
                addr.address()
            } else {
                return Err(vm.new_type_error("only supported types are int and Symbol".to_owned()));
            };

            let res = unsafe { mem::hook::hook(address as _, self.jit.address()) };
            let trampoline = res.map_err(|e| vm.new_runtime_error(format!("{e}")))?;

            let hook = Hook::Jmp(trampoline);
            let jitpoline = Jitpoline::new(hook, (&self.params.0, self.params.1), self.call_conv)?;

            *lock = Some(jitpoline);

            Ok(true)
        }

        #[pymethod]
        fn hook_iat(&self, entry: PyRef<PyIATSymbol>, vm: &VirtualMachine) -> PyResult<bool> {
            let mut lock = self.jitpoline.lock().unwrap();
            if lock.is_some() {
                return Err(vm.new_runtime_error(
                    "this callable is already hooking something. create a new callable to hook something else"
                        .to_owned(),
                ));
            }

            let res = unsafe { entry.hook(self.jit.address().cast()) };
            res.map_err(|e| vm.new_runtime_error(e.to_string()))?;

            let hook = Hook::IAT((**entry).clone());
            let jitpoline = Jitpoline::new(hook, (&self.params.0, self.params.1), self.call_conv)?;

            *lock = Some(jitpoline);

            Ok(true)
        }

        #[pymethod]
        fn hook_vmt(
            &self,
            vtable: PyRef<PyVTable>,
            index: usize,
            vm: &VirtualMachine,
        ) -> PyResult<bool> {
            let mut lock = self.jitpoline.lock().unwrap();
            if lock.is_some() {
                return Err(vm.new_runtime_error(
                    "this callable is already hooking something. create a new callable to hook something else"
                        .to_owned(),
                ));
            }

            let res = unsafe { vtable.hook(index, self.jit.address().cast()) };
            res.map_err(|e| vm.new_runtime_error(e.to_string()))?;

            let hook = Hook::Vmt(VTableHook(index, vtable));
            let jitpoline = Jitpoline::new(hook, (&self.params.0, self.params.1), self.call_conv)?;

            *lock = Some(jitpoline);

            Ok(true)
        }

        /// unsafe fn
        #[pymethod]
        fn unhook(&self, vm: &VirtualMachine) -> PyResult<()> {
            let lock = self.jitpoline.lock().unwrap();
            if let Some(jitpoline) = &*lock {
                unsafe {
                    jitpoline.unhook(vm)?;
                }
            }

            Ok(())
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
            let span = trace_span!("drop");
            let _guard = span.enter();
            trace!(index = self.index(), "dropping VTableHook");

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

    impl Unconstructible for PyType {}

    #[pyclass(with(Unconstructible))]
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

        /// Only valid in argument position
        #[pyfunction(name = "Struct")]
        fn _struct(size: u32, vm: &VirtualMachine) -> PyResult<PyType> {
            if size == 0 {
                return Err(vm.new_value_error("StructArg size must be > 0".to_owned()));
            }

            Ok(PyType(Type::Struct(size)))
        }
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

    impl Unconstructible for PyCallConv {}

    #[pyclass(with(Unconstructible))]
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
    #[pymodule(name = "Conv")]
    pub mod _conv {
        use super::*;

        /// defaults to WindowsFastcall
        #[pyattr]
        pub(super) const C: PyCallConv = PyCallConv(CallConv::WindowsFastcall);

        /// same as cdecl on Windows
        #[pyattr]
        pub(super) const WindowsFastcall: PyCallConv = PyCallConv(CallConv::WindowsFastcall);

        /// same as cdecl on Windows
        #[pyattr]
        pub(super) const Stdcall: PyCallConv = PyCallConv(CallConv::WindowsFastcall);
    }

    #[pyattr]
    #[pyclass(name)]
    #[derive(Debug, PyPayload)]
    pub struct WStr(Vec<u16>);

    impl Drop for WStr {
        fn drop(&mut self) {
            let span = trace_span!("drop");
            let _guard = span.enter();
            trace!(data = self.as_str(), "dropping WStr");
        }
    }

    impl Constructor for WStr {
        type Args = FuncArgs;

        fn py_new(
            _cls: PyTypeRef,
            mut args: Self::Args,
            vm: &VirtualMachine,
        ) -> PyResult<PyObjectRef> {
            let with_null = args
                .kwargs
                .swap_remove("null")
                .map(|p| p.try_to_bool(vm))
                .transpose()?;

            let len: Option<usize> = args
                .kwargs
                .swap_remove("len")
                .map(|l| l.try_to_value(vm))
                .transpose()?;

            // first positional arg determines flavor of constructor
            let obj = args.args.first().ok_or_else(|| {
                vm.new_runtime_error("constructor requires 1 pos param, usize|str".to_owned())
            })?;

            if let Ok(address) = obj.try_to_value::<Address>(vm) {
                let null = with_null.unwrap_or(false);

                if !null && len.is_none() {
                    return Err(vm.new_runtime_error(
                        "null or len kwarg is required to get the wstr".to_owned(),
                    ));
                } else if null && len.is_some() {
                    return Err(vm.new_runtime_error(
                        "only use one of null or len kwarg, not both".to_owned(),
                    ));
                }

                let lossy = args
                    .kwargs
                    .swap_remove("lossy")
                    .map(|p| p.try_to_bool(vm))
                    .transpose()?
                    .unwrap_or(false);

                let slice = if null {
                    let mut ptr = address as *const u16;
                    let mut offset = 0usize;
                    while unsafe { *ptr } != 0 {
                        unsafe {
                            ptr = ptr.add(1);
                        };

                        offset += 1;
                    }

                    unsafe { std::slice::from_raw_parts(address as *const u16, offset) }
                } else {
                    let len = len.unwrap();

                    unsafe { std::slice::from_raw_parts(address as *const u16, len) }
                };

                // verify this is a valid string - ignore check if it's lossy
                if !lossy && String::from_utf16(slice).is_err() {
                    return Err(vm.new_runtime_error("this is not a valid utf16 string".to_owned()));
                }

                let zelf = Self(slice.to_vec());

                Ok(zelf.to_pyobject(vm))
            } else if let Ok(string) = obj.try_to_value::<String>(vm) {
                let mut data = string.encode_utf16().collect::<Vec<_>>();

                if with_null.unwrap_or(false) {
                    data.push(0);
                }

                let zelf = Self(data);

                Ok(zelf.to_pyobject(vm))
            } else {
                Err(vm.new_runtime_error("constructor param must be usize|str".to_owned()))
            }
        }
    }

    #[pyclass(with(Constructor))]
    impl WStr {
        /// The byte size of the string
        #[pygetset]
        fn size(&self) -> usize {
            self.0.len() * 2
        }

        /// The address to the data buffer
        #[pygetset]
        fn address(&self) -> Address {
            self.0.as_ptr() as _
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            format!("WStr({})", self.as_str())
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            self.as_str()
        }

        /// The byte size of the string
        fn as_str(&self) -> String {
            String::from_utf16_lossy(&self.0)
        }

        pub fn as_ptr(&self) -> *const u16 {
            self.0.as_ptr() as _
        }
    }
}
