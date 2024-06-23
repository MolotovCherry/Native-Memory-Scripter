use std::arch::asm;

use rustpython_vm::{builtins::PyBaseException, function::FuncArgs, PyRef, VirtualMachine};
use tracing::{error, trace_span};

use crate::modules::cffi::ret::Ret;

use super::Data;

pub extern "fastcall" fn __jit_cb(args: *const (), data: &Data, ret: *mut Ret) {
    // ret vals:
    // void: 0 (no write)
    // any return: ptr, write Ret to it
    // ret:struct: ptr, write struct data to it

    data.vm.run(|vm| {
        let span = trace_span!("__jit_cb");
        let mut _guard = span.enter();

        let mut py_args = FuncArgs::default();

        if !args.is_null() {
            let args = unsafe { data.layout.as_ref().unwrap().iter(args) };
            let args = args
                .zip(data.params.0.iter())
                .map(|(arg, _)| arg.to_pyobject(vm));

            py_args.args.extend(args);
        }

        drop(_guard);
        let call = data.callable.call_with_args(py_args, vm);
        _guard = span.enter();

        match call {
            Ok(obj) => {
                let res = Ret::write_ret(obj, data.params.1, ret, vm);

                // we have just entered a UB code path, so we have to crash.
                if let Err(e) = res {
                    // this is an illegal code path, but we should at the very least print something
                    illegal(e, vm);
                }
            }

            // we have just entered a UB code path, so we have to crash.
            Err(e) => {
                // this is an illegal code path, but we should at the very least print something
                illegal(e, vm);
            }
        }
    });
}

fn illegal(exc: PyRef<PyBaseException>, vm: &VirtualMachine) {
    let msg = "there is a bug in your code. either there is an uncaught exception, or you violated an invariant (e.g. you returned a bool when you specified a CStr). you must gracefully handle all exceptions and return something. program will now crash.";

    let mut data = String::new();

    if let Err(e) = vm.write_exception(&mut data, &exc) {
        error!("failed to write error: {e}");

        error!("{msg}");

        // our code path is broken, just crash here
        unsafe {
            asm!("ud2");
        }
    }

    let data = data.trim();
    error!("\n{data}");

    error!("{msg}");

    // our code path is broken, just crash here
    unsafe {
        asm!("ud2");
    }
}
