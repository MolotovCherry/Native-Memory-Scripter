use std::fmt::{self, Formatter};

use rustpython_vm::vm::thread::ThreadedVirtualMachine;

/// Wrapper to get debug since ThreadedVirtualMachine didn't impl it
pub struct PyThreadedVirtualMachine(pub ThreadedVirtualMachine);

impl fmt::Debug for PyThreadedVirtualMachine {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ThreadedVirtualMachine")
    }
}
