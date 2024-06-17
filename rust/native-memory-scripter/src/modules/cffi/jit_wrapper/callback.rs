use rustpython_vm::{convert::ToPyObject, function::FuncArgs};
use tracing::error;

use crate::modules::cffi::{args::Arg, ret::Ret};

use super::Data;

pub extern "fastcall" fn __jit_cb(args: *const (), data: &Data, ret: *mut Ret) {
    // ret vals:
    // void: 0 (no write)
    // any return: ptr, write Ret to it
    // structreturn: ptr, write struct data to it

    data.vm.run(|vm| {
        let mut py_args = FuncArgs::default();

        if !args.is_null() {
            let args = unsafe { data.layout.as_ref().unwrap().iter(args) };
            let args = args.zip(data.params.0.iter()).map(|(arg, _)| {
                if let Arg::SArg(size, ptr) = arg {
                    let data = unsafe { mem::memory::read_bytes(ptr.cast(), size as _) };

                    data.to_pyobject(vm)
                } else {
                    arg.to_pyobject(vm)
                }
            });

            py_args.args.extend(args);
        }

        match data.callable.call_with_args(py_args, vm) {
            Ok(obj) => {
                let res = Ret::write_ret(obj, data.params.1, ret, vm);

                if let Err(e) = res {
                    Ret::write_default_ret(data.params.1, ret);

                    let mut data = String::new();

                    if let Err(e) = vm.write_exception(&mut data, &e) {
                        error!("failed to write error: {e}");
                        return;
                    }

                    let data = data.trim();
                    error!("\n{data}");
                }
            }

            Err(e) => {
                // potential UB! but we have no choice, we at least return *something* to try to prevent UB
                // SAFETY: Catch exceptions in your callback code!
                Ret::write_default_ret(data.params.1, ret);

                let mut data = String::new();

                if let Err(e) = vm.write_exception(&mut data, &e) {
                    error!("failed to write error: {e}");
                    return;
                }

                let data = data.trim();
                error!("\n{data}");
            }
        }
    });
}
