use rustpython_vm::pymodule;

#[allow(clippy::module_inception)]
#[pymodule]
pub mod iat {
    use std::ops::Deref;

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
    fn find(module: &PyModule, name: String, vm: &VirtualMachine) -> PyResult<Option<PyObjectRef>> {
        let name = SymbolIdent::Name(name);
        let symbols =
            find_iat_symbol(module, &name).map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        let symbol = symbols.map(|sym| PyIATSymbol(sym).into_pyobject(vm));

        Ok(symbol)
    }

    #[pyfunction]
    fn find_ordinal(
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
    fn find_with_dll(
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
    fn find_with_dll_ordinal(
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
    fn find_demangled(
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
    fn find_with_dll_demangled(
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
    pub struct PyIATSymbol(IATSymbol);

    impl Deref for PyIATSymbol {
        type Target = IATSymbol;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[pyclass]
    impl PyIATSymbol {
        #[pygetset]
        fn name(&self) -> Option<String> {
            match self.0.ident {
                SymbolIdent::Name(ref n) => Some(n.clone()),
                SymbolIdent::Ordinal(_) => None,
            }
        }

        #[pygetset]
        fn ordinal(&self) -> Option<u16> {
            match self.0.ident {
                SymbolIdent::Name(_) => None,
                SymbolIdent::Ordinal(o) => Some(o),
            }
        }

        #[pygetset]
        fn dll_name(&self) -> String {
            self.0.dll.clone()
        }

        /// Address of original fn stored at this iat entry
        #[pygetset]
        fn orig_fn(&self) -> Address {
            self.0.orig_fn as _
        }

        /// You can write a u64 here to hook it somewhere else, but make sure you first make protection writeable
        #[pygetset]
        fn iat(&self) -> Address {
            self.0.entry as _
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
