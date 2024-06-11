#[pyfunction]
fn demangle_symbol(symbol_name: String) -> Option<String> {
    mem::symbol::demangle_symbol(&symbol_name)
}

#[pyfunction]
fn enum_symbols(module: PyRef<PyModule>, vm: &VirtualMachine) -> PyResult<Vec<PyObjectRef>> {
    mem::symbol::enum_symbols(&module.0)
        .map(|symbols| {
            symbols
                .into_iter()
                .map(|symbol| PySymbol(symbol).into_pyobject(vm))
                .collect()
        })
        .map_err(|e| vm.new_runtime_error(format!("{e}")))
}

#[pyfunction]
fn enum_symbols_demangled(
    module: PyRef<PyModule>,
    vm: &VirtualMachine,
) -> PyResult<Vec<PyObjectRef>> {
    mem::symbol::enum_symbols_demangled(&module.0)
        .map(|symbols| {
            symbols
                .into_iter()
                .map(|symbol| PySymbol(symbol).into_pyobject(vm))
                .collect()
        })
        .map_err(|e| vm.new_runtime_error(format!("{e}")))
}

#[pyfunction]
fn find_symbol_address_demangled(
    module: PyRef<PyModule>,
    demangled_symbol_name: String,
    vm: &VirtualMachine,
) -> PyResult<Option<usize>> {
    let res = mem::symbol::find_symbol_address(&module.0, &demangled_symbol_name)
        .map_err(|e| vm.new_runtime_error(format!("{e}")))?;

    Ok(res.map(|sym| sym.address as _))
}

#[pyfunction]
fn find_symbol_address(
    module: PyRef<PyModule>,
    symbol_name: String,
    vm: &VirtualMachine,
) -> PyResult<Option<usize>> {
    let res = mem::symbol::find_symbol_address(&module.0, &symbol_name)
        .map_err(|e| vm.new_runtime_error(format!("{e}")))?;

    Ok(res.map(|sym| sym.address as _))
}

#[pyattr]
#[pyclass(name = "Symbol")]
#[derive(PyPayload)]
struct PySymbol(Symbol);

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
    fn address(&self) -> usize {
        self.0.address as _
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
