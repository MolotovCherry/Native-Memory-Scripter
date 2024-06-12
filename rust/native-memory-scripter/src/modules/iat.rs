use rustpython_vm::pymodule;

#[allow(clippy::module_inception)]
#[pymodule]
pub mod iat {
    use mem::iat::{
        enum_iat_symbols, enum_iat_symbols_demangled, find_dll_iat_symbol,
        find_dll_iat_symbol_demangled, find_iat_symbol, find_iat_symbol_demangled, IATSymbol,
        SymbolIdent,
    };
    use rustpython_vm::{prelude::*, pyclass, PyObjectRef, PyPayload, PyResult};

    use crate::modules::{modules::modules::PyModule, Address};

    #[pyfunction(name = "enum")]
    fn enum_(module: &PyModule, vm: &VirtualMachine) -> PyResult<Vec<PyObjectRef>> {
        let symbols = enum_iat_symbols(module).map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        let symbols = symbols
            .into_iter()
            .map(|sym| PyIATSymbol(sym).into_pyobject(vm))
            .collect();

        Ok(symbols)
    }

    #[pyfunction]
    fn enum_demangled(module: &PyModule, vm: &VirtualMachine) -> PyResult<Vec<PyObjectRef>> {
        let symbols =
            enum_iat_symbols_demangled(module).map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        let symbols = symbols
            .into_iter()
            .map(|sym| PyIATSymbol(sym).into_pyobject(vm))
            .collect();

        Ok(symbols)
    }

    #[pyfunction]
    fn find_symbol_name(
        module: &PyModule,
        name: String,
        vm: &VirtualMachine,
    ) -> PyResult<Option<PyObjectRef>> {
        let name = SymbolIdent::Name(name);
        let symbols =
            find_iat_symbol(module, &name).map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        let symbol = symbols.map(|sym| PyIATSymbol(sym).into_pyobject(vm));

        Ok(symbol)
    }

    #[pyfunction]
    fn find_symbol_ordinal(
        module: &PyModule,
        ord: u16,
        vm: &VirtualMachine,
    ) -> PyResult<Option<PyObjectRef>> {
        let name = SymbolIdent::Ordinal(ord);
        let symbols =
            find_iat_symbol(module, &name).map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        let symbol = symbols.map(|sym| PyIATSymbol(sym).into_pyobject(vm));

        Ok(symbol)
    }

    #[pyfunction]
    fn find_dll_symbol_name(
        module: &PyModule,
        name: String,
        dll: String,
        vm: &VirtualMachine,
    ) -> PyResult<Option<PyObjectRef>> {
        let name = SymbolIdent::Name(name);
        let symbols = find_dll_iat_symbol(module, &dll, &name)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        let symbol = symbols.map(|sym| PyIATSymbol(sym).into_pyobject(vm));

        Ok(symbol)
    }

    #[pyfunction]
    fn find_dll_symbol_ordinal(
        module: &PyModule,
        ord: u16,
        dll: String,
        vm: &VirtualMachine,
    ) -> PyResult<Option<PyObjectRef>> {
        let name = SymbolIdent::Ordinal(ord);
        let symbols = find_dll_iat_symbol(module, &dll, &name)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        let symbol = symbols.map(|sym| PyIATSymbol(sym).into_pyobject(vm));

        Ok(symbol)
    }

    #[pyfunction]
    fn find_symbol_name_demangled(
        module: &PyModule,
        name: String,
        vm: &VirtualMachine,
    ) -> PyResult<Option<PyObjectRef>> {
        let symbols = find_iat_symbol_demangled(module, &name)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        let symbol = symbols.map(|sym| PyIATSymbol(sym).into_pyobject(vm));

        Ok(symbol)
    }

    #[pyfunction]
    fn find_dll_symbol_name_demangled(
        module: &PyModule,
        name: String,
        dll: String,
        vm: &VirtualMachine,
    ) -> PyResult<Option<PyObjectRef>> {
        let symbols = find_dll_iat_symbol_demangled(module, &dll, &name)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        let symbol = symbols.map(|sym| PyIATSymbol(sym).into_pyobject(vm));

        Ok(symbol)
    }

    #[pyattr]
    #[pyclass(name = "IATSymbol")]
    #[derive(Debug, PyPayload)]
    struct PyIATSymbol(IATSymbol);

    #[pyclass]
    impl PyIATSymbol {
        #[pygetset]
        fn name(&self) -> Option<String> {
            match self.0.identifier {
                SymbolIdent::Name(ref n) => Some(n.clone()),
                SymbolIdent::Ordinal(_) => None,
            }
        }

        #[pygetset]
        fn ordinal(&self) -> Option<u16> {
            match self.0.identifier {
                SymbolIdent::Name(_) => None,
                SymbolIdent::Ordinal(o) => Some(o),
            }
        }

        #[pygetset]
        fn dll_name(&self) -> String {
            self.0.dll_name.clone()
        }

        /// Address of original fn stored at this iat entry
        #[pygetset]
        fn fn_address(&self) -> usize {
            self.0.fn_address as _
        }

        /// You can write a u64 here to hook it somewhere else, but make sure you first make protection writeable
        #[pygetset]
        fn iat_address(&self) -> usize {
            self.0.iat_address as _
        }

        /// unsafe fn
        #[pymethod]
        fn hook(&self, address: Address, vm: &VirtualMachine) -> PyResult<()> {
            let res = unsafe { self.0.hook(address as _) };
            res.map_err(|e| vm.new_runtime_error(format!("{e}")))?;

            Ok(())
        }

        /// restore this iat entry's original fn
        ///
        /// unsafe fn
        #[pymethod]
        fn unhook(&self, vm: &VirtualMachine) -> PyResult<()> {
            let res = unsafe { self.0.unhook() };
            res.map_err(|e| vm.new_runtime_error(format!("{e}")))?;

            Ok(())
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
