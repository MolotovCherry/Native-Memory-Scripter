use rustpython_vm::function::FuncArgs;

use crate::modules::cffi::ret::Ret;

use super::Data;

pub extern "fastcall" fn __jit_cb(args: *const (), data: &Data, ret: &mut Ret) {
    let result = data.vm.shared_run(|vm| {
        let mut iter = unsafe { data.layout.iter(args) };

        let mut py_args = FuncArgs::default();

        let first = iter.next().unwrap().as_u64();
        let second = iter.next().unwrap().as_u64();
        let third = iter.next().unwrap().as_u64();
        let fourth = iter.next().unwrap().as_u64();
        let fifth = iter.next().unwrap().as_u128();

        py_args.prepend_arg(vm.new_pyobj(fifth));
        py_args.prepend_arg(vm.new_pyobj(fourth));
        py_args.prepend_arg(vm.new_pyobj(third));
        py_args.prepend_arg(vm.new_pyobj(second));
        py_args.prepend_arg(vm.new_pyobj(first));

        if let Err(e) = data.callable.call_with_args(py_args, vm) {
            vm.print_exception(e);
        }

        9u32
    });

    *ret = Ret { u64: 9 };
}
