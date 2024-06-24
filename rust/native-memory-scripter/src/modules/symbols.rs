use rustpython_vm::pymodule;

#[pymodule]
pub mod symbols {
    use std::fmt::Debug;

    use mem::symbols::Symbol;
    use rustpython_vm::{pyclass, PyObjectRef, PyPayload, PyRef, PyResult, VirtualMachine};

    use crate::modules::{modules::modules::PyModule, Address};

    #[pyfunction]
    fn demangle(name: String) -> Option<String> {
        mem::symbols::demangle_symbol(&name)
    }

    /// Return a list of all symbols for a module
    #[pyfunction(name = "enum")]
    fn enum_(module: PyRef<PyModule>, vm: &VirtualMachine) -> PyResult<Vec<PyObjectRef>> {
        mem::symbols::enum_symbols(&module)
            .map(|symbols| {
                symbols
                    .into_iter()
                    .map(|symbol| PySymbol(symbol).into_pyobject(vm))
                    .collect()
            })
            .map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    /// Return a list of all demangled symbols for a module
    #[pyfunction]
    fn enum_demangled(module: PyRef<PyModule>, vm: &VirtualMachine) -> PyResult<Vec<PyObjectRef>> {
        mem::symbols::enum_symbols_demangled(&module)
            .map(|symbols| {
                symbols
                    .into_iter()
                    .map(|symbol| PySymbol(symbol).into_pyobject(vm))
                    .collect()
            })
            .map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    /// Find the address of a symbol in a module. Case sensitive.
    #[pyfunction]
    fn find(
        module: PyRef<PyModule>,
        name: String,
        vm: &VirtualMachine,
    ) -> PyResult<Option<Address>> {
        let res = mem::symbols::find_symbol_address(&module, &name)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        Ok(res.map(|sym| sym.address as _))
    }

    /// Find the address of a demangled symbol in a module. Case sensitive.
    #[pyfunction]
    fn find_demangled(
        module: PyRef<PyModule>,
        name: String,
        vm: &VirtualMachine,
    ) -> PyResult<Option<Address>> {
        let res = mem::symbols::find_symbol_address(&module, &name)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        Ok(res.map(|sym| sym.address as _))
    }

    #[pyattr]
    #[pyclass(name = "Symbol")]
    #[derive(PyPayload)]
    pub struct PySymbol(Symbol);

    impl Debug for PySymbol {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    #[pyclass]
    impl PySymbol {
        #[pygetset]
        fn name(&self) -> String {
            self.0.name.clone()
        }

        #[pygetset]
        pub fn address(&self) -> Address {
            self.0.address as _
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
