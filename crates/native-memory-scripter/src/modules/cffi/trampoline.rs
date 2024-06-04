use std::ptr::NonNull;

use rustpython_vm::prelude::{PyResult, VirtualMachine};

use crate::utils::RawSendable;

use super::{args::ArgMemory, types::Type};

pub struct Trampoline {
    addr: RawSendable<()>,
    arg_mem: ArgMemory,
}

impl Trampoline {
    pub fn new(address: usize, args: &[Type], vm: &VirtualMachine) -> PyResult<Self> {
        let arg_mem = ArgMemory::new(args)
            .ok_or_else(|| vm.new_runtime_error("failed to create ArgMemory".to_owned()))?;

        let ptr = NonNull::new(address as *mut ())
            .ok_or_else(|| vm.new_runtime_error("address is unexpectedly null".to_owned()))?;

        let slf = Self {
            addr: RawSendable(ptr),
            arg_mem,
        };

        Ok(slf)
    }
}
