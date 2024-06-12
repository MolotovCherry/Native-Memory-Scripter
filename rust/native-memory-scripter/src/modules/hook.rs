use rustpython_vm::pymodule;

#[pymodule]
pub mod hook {
    use mem::hook::Trampoline;
    use rustpython_vm::{pyclass, PyObjectRef, PyPayload, PyResult, VirtualMachine};

    use crate::modules::Address;

    #[pyfunction]
    fn hook(from: Address, to: Address, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        let trampoline = unsafe { mem::hook::hook(from as _, to as _) };
        trampoline
            .map(|t| PyTrampoline(t).into_pyobject(vm))
            .map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    /// The trampoline and its data. Note that this is a raw hook. It does not
    /// let you use a python callback. If you want to do that, use the cffi module.
    ///
    /// Function will be auto-unhooked when this trampoline is dropped.
    #[pyattr]
    #[pyclass(name = "Trampoline")]
    #[derive(Debug, Clone, PyPayload)]
    struct PyTrampoline(Trampoline);

    #[pyclass]
    impl PyTrampoline {
        /// The address of the trampoline
        #[pygetset]
        fn address(&self) -> Address {
            self.0.address as _
        }

        /// Size of the code in the trampoline
        #[pygetset]
        fn size(&self) -> usize {
            self.0.size
        }

        /// Unhook the hooked function. If this is not called, function is auto-unhooked when instance is dropped
        ///
        /// unsafe fn
        #[pymethod]
        fn unhook(&self, vm: &VirtualMachine) -> PyResult<()> {
            let res = unsafe { self.0.unhook() };
            res.map_err(|e| vm.new_runtime_error(format!("{e}")))
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            format!("{:?}", self.0)
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            self.repr()
        }
    }

    impl From<&PyTrampoline> for mem::hook::Trampoline {
        fn from(t: &PyTrampoline) -> Self {
            t.clone().0
        }
    }
}
