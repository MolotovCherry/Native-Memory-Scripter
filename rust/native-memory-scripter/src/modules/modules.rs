use rustpython_vm::pymodule;

#[allow(clippy::module_inception)]
#[pymodule]
pub mod modules {
    use std::ops::Deref;

    use mem::modules::Module;
    use rustpython_vm::{pyclass, PyObjectRef, PyPayload, PyResult, VirtualMachine};

    /// Load a library into the process
    #[pyfunction]
    fn load(path: String, vm: &VirtualMachine) -> PyResult<PyModule> {
        mem::modules::Module::load(path)
            .map(PyModule)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    /// Load a library into the process
    /// Unsafe, because you can cause a module to be unloaded while it's in use,
    /// and even use an existing Module calling into its address space which is gone
    ///
    /// unsafe fn
    #[pyfunction]
    fn unload(path: String, vm: &VirtualMachine) -> PyResult<()> {
        let res = unsafe { mem::modules::Module::unload_path(path) };
        res.map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    #[pyfunction]
    fn find(name: String, vm: &VirtualMachine) -> PyResult<Option<PyModule>> {
        let module = mem::modules::find_module(&name)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))?
            .map(PyModule);

        Ok(module)
    }

    #[pyfunction(name = "enum")]
    fn enum_(vm: &VirtualMachine) -> PyResult<Vec<PyObjectRef>> {
        mem::modules::enum_modules()
            .map(|modules| {
                modules
                    .into_iter()
                    .map(|module| PyModule(module).into_pyobject(vm))
                    .collect()
            })
            .map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    /// This keeps the module open as long as it exists.
    /// To unload from this, just simply let it drop or delete it
    #[pyattr]
    #[pyclass(name = "Module")]
    #[derive(Debug, PyPayload)]
    pub struct PyModule(Module);

    impl Deref for PyModule {
        type Target = Module;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[pyclass]
    impl PyModule {
        #[pygetset]
        fn base(&self) -> usize {
            self.0.base as _
        }

        #[pygetset]
        fn end(&self) -> usize {
            self.0.end as _
        }

        #[pygetset]
        fn size(&self) -> u32 {
            self.0.size
        }

        #[pygetset]
        fn path(&self) -> String {
            self.0.path.to_string_lossy().to_string()
        }

        #[pygetset]
        fn name(&self) -> String {
            self.0.name.clone()
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            format!("{:?}", self.0)
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            format!("{:?}", self.0)
        }
    }
}
