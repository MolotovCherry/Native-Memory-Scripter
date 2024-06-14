use rustpython_vm::function::FuncArgs;

use crate::modules::cffi::ret::Ret;

use super::Data;

pub extern "fastcall" fn __jit_cb(args: *const (), data: &Data, ret: &mut Ret) {
    let result = data.vm.run(|vm| {
        let mut iter = unsafe { data.layout.iter(args) };

        let mut py_args = FuncArgs::default();

        let first = iter.next().unwrap().as_u64();

        py_args.prepend_arg(vm.new_pyobj(first));

        if let Err(e) = data.callable.call_with_args(py_args, vm) {
            vm.print_exception(e);
        }

        9u32
    });

    *ret = Ret { u64: 9 };
}
