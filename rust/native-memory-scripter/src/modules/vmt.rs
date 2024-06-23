use rustpython_vm::pymodule;

#[allow(clippy::module_inception)]
#[pymodule]
pub mod vmt {
    use std::{fmt::Debug, ops::Deref};

    use mem::vtable::VTable;
    use rustpython_vm::{
        builtins::PyTypeRef, pyclass, types::Constructor, PyObjectRef, PyPayload, PyResult,
        VirtualMachine,
    };
    use tracing::trace;

    use crate::modules::Address;

    #[pyattr]
    #[pyclass(name = "VTable")]
    #[derive(PyPayload)]
    pub struct PyVTable(VTable);

    impl Drop for PyVTable {
        fn drop(&mut self) {
            trace!("dropping VTable");
        }
    }

    impl Deref for PyVTable {
        type Target = VTable;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Debug for PyVTable {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl Constructor for PyVTable {
        type Args = Address;

        fn py_new(_cls: PyTypeRef, args: Self::Args, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let vmt = VTable::new(args as _);
            let slf = Self(vmt).into_pyobject(vm);

            Ok(slf)
        }
    }

    #[pyclass(with(Constructor))]
    impl PyVTable {
        /// Hook an index of the virtual method table
        ///
        /// unsafe fn
        #[pymethod]
        fn hook(&self, index: usize, dst: Address, vm: &VirtualMachine) -> PyResult<()> {
            let res = unsafe { self.0.hook(index, dst as _) };
            res.map_err(|e| vm.new_runtime_error(e.to_string()))
        }

        #[pymethod]
        fn unhook(&self, index: usize, vm: &VirtualMachine) -> PyResult<()> {
            let res = unsafe { self.0.unhook(index) };
            res.map_err(|e| vm.new_runtime_error(e.to_string()))
        }

        #[pymethod]
        fn get_original(&self, index: usize) -> Option<usize> {
            let res = self.0.get_original(index);
            res.map(|ptr| ptr as _)
        }

        #[pymethod]
        fn reset(&self, vm: &VirtualMachine) -> PyResult<()> {
            let res = unsafe { self.0.reset() };
            res.map_err(|e| vm.new_runtime_error(e.to_string()))
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            format!("{self:?}")
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            self.repr()
        }
    }
}
