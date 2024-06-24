use rustpython_vm::pymodule;

#[pymodule]
pub mod iat {
    use std::ops::Deref;

    use mem::iat::{
        enum_iat_symbols, enum_iat_symbols_demangled, find_dll_iat_symbol,
        find_dll_iat_symbol_demangled, find_iat_symbol, find_iat_symbol_demangled, IATSymbol,
        SymbolIdent,
    };
    use rustpython_vm::{
        function::FuncArgs, prelude::*, pyclass, PyObjectRef, PyPayload, PyResult,
    };
    use tracing::{trace, trace_span};

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
    fn find(
        module: &PyModule,
        args: FuncArgs,
        vm: &VirtualMachine,
    ) -> PyResult<Option<PyObjectRef>> {
        // 2 arg mode : module + name (str) / ordinal (u16)
        // 3 arg mode : module + dll name (str) + name (str) / ordinal (u16)

        let len = args.args.len();

        if ![1, 2].contains(&len) {
            return Err(vm.new_runtime_error("this fn only supports 2 or 3 args".to_owned()));
        }

        let dll_name = if len == 2 {
            let name = args.args[0].try_to_value::<String>(vm)?;
            Some(name)
        } else {
            None
        };

        let name = {
            let idx = match len {
                1 => 0,
                2 => 1,
                _ => unreachable!(),
            };

            let _str = args.args[idx].try_to_value::<String>(vm);
            let _ord = args.args[idx].try_to_value::<u16>(vm);

            if let Ok(_str) = _str {
                SymbolIdent::Name(_str)
            } else if let Ok(_ord) = _ord {
                SymbolIdent::Ordinal(_ord)
            } else {
                return Err(vm.new_type_error("name field only supports str or u16".to_owned()));
            }
        };

        let res = if len == 1 {
            find_iat_symbol(module, &name).map_err(|e| vm.new_runtime_error(e.to_string()))?
        } else if let Some(dll_name) = dll_name {
            find_dll_iat_symbol(module, &dll_name, &name)
                .map_err(|e| vm.new_runtime_error(e.to_string()))?
        } else {
            unreachable!()
        };

        let symbol = res.map(|sym| PyIATSymbol(sym).into_pyobject(vm));

        Ok(symbol)
    }

    #[pyfunction]
    fn find_demangled(
        module: &PyModule,
        args: FuncArgs,
        vm: &VirtualMachine,
    ) -> PyResult<Option<PyObjectRef>> {
        // 2 arg mode : module + name (str)
        // 3 arg mode : module + dll name (str) + name (str)

        let len = args.args.len();

        if ![1, 2].contains(&len) {
            return Err(vm.new_runtime_error("this fn only supports 2 or 3 args".to_owned()));
        }

        let dll_name = if len == 2 {
            let name = args.args[0].try_to_value::<String>(vm)?;
            Some(name)
        } else {
            None
        };

        let name = {
            let _str = args.args[len - 1].try_to_value::<String>(vm);

            if let Ok(_str) = _str {
                _str
            } else {
                return Err(vm.new_type_error("name field only supports str".to_owned()));
            }
        };

        let res = if len == 1 {
            find_iat_symbol_demangled(module, &name)
                .map_err(|e| vm.new_runtime_error(e.to_string()))?
        } else if let Some(dll_name) = dll_name {
            find_dll_iat_symbol_demangled(module, &dll_name, &name)
                .map_err(|e| vm.new_runtime_error(e.to_string()))?
        } else {
            unreachable!()
        };

        let symbol = res.map(|sym| PyIATSymbol(sym).into_pyobject(vm));

        Ok(symbol)
    }

    #[pyattr]
    #[pyclass(name = "IATSymbol")]
    #[derive(Debug, PyPayload)]
    pub struct PyIATSymbol(IATSymbol);

    impl Drop for PyIATSymbol {
        fn drop(&mut self) {
            let span = trace_span!("drop");
            let _guard = span.enter();
            trace!(address = ?self.0.entry, "dropping IATSymbol");
        }
    }

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
