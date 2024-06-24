use rustpython_vm::pymodule;

#[pymodule]
pub mod log {
    use rustpython_vm::{function::FuncArgs, prelude::VirtualMachine, PyResult};

    fn process_args(args: FuncArgs, vm: &VirtualMachine) -> PyResult<String> {
        // separator
        let sep = args
            .kwargs
            .get("sep")
            .map(|s| s.str(vm).map(|s| s.to_string()))
            .transpose()?
            .unwrap_or_else(|| " ".to_owned());

        let mut strings = Vec::new();

        // *args
        for arg in args.args {
            let string = if let Ok(_str) = arg.str(vm) {
                _str.to_string()
            } else {
                arg.repr(vm)?.to_string()
            };

            strings.push(string);
        }

        Ok(strings.join(&sep))
    }

    #[pyfunction]
    fn trace(args: FuncArgs, vm: &VirtualMachine) -> PyResult<()> {
        let log = process_args(args, vm)?;

        tracing::trace!(target: "script_log", "{log}");

        Ok(())
    }

    #[pyfunction]
    fn debug(args: FuncArgs, vm: &VirtualMachine) -> PyResult<()> {
        let log = process_args(args, vm)?;

        tracing::debug!(target: "script_log", "{log}");

        Ok(())
    }

    #[pyfunction]
    fn info(args: FuncArgs, vm: &VirtualMachine) -> PyResult<()> {
        let log = process_args(args, vm)?;

        tracing::info!(target: "script_log", "{log}");

        Ok(())
    }

    #[pyfunction]
    fn warn(args: FuncArgs, vm: &VirtualMachine) -> PyResult<()> {
        let log = process_args(args, vm)?;

        tracing::warn!(target: "script_log", "{log}");

        Ok(())
    }

    #[pyfunction]
    fn error(args: FuncArgs, vm: &VirtualMachine) -> PyResult<()> {
        let log = process_args(args, vm)?;

        tracing::error!(target: "script_log", "{log}");

        Ok(())
    }
}
