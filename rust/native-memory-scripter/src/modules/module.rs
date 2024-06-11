    /// Load a library into the process
    #[pyfunction]
    fn load_module(path: String, vm: &VirtualMachine) -> PyResult<PyModule> {
        mem::module::Module::load(&path)
            .map(PyModule)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

#[pyfunction]
fn find_module(name: String, vm: &VirtualMachine) -> PyResult<Option<PyModule>> {
    let module = mem::module::find_module(&name)
        .map_err(|e| vm.new_runtime_error(format!("{e}")))?
        .map(PyModule);

    Ok(module)
}

#[pyfunction]
fn enum_modules(vm: &VirtualMachine) -> PyResult<Vec<PyObjectRef>> {
    mem::module::enum_modules()
        .map(|modules| {
            modules
                .into_iter()
                .map(|module| PyModule(module).into_pyobject(vm))
                .collect()
        })
        .map_err(|e| vm.new_runtime_error(format!("{e}")))
}

/// Dropping this will unload the module
#[pyattr]
#[pyclass(name = "Module")]
#[derive(Debug, PyPayload)]
struct PyModule(Module);

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
        format!("{self:?}")
    }

    #[pymethod(magic)]
    fn str(&self) -> String {
        format!("{self:?}")
    }
}
