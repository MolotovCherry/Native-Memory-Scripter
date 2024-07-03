use rustpython_vm::pymodule;

#[pymodule]
pub mod mem {
    use std::fmt::Debug;

    use mutation::{memory, memory::Alloc, Prot};
    use rustpython_vm::{
        builtins::PyByteArray, convert::ToPyObject as _, function::FuncArgs, prelude::*, pyclass,
        pymodule, PyPayload, VirtualMachine,
    };
    use tracing::{trace, trace_span};

    use crate::modules::Address;

    /// Calculates a deep pointer address by applying a series of offsets to a base address and dereferencing intermediate pointers.
    ///
    /// unsafe fn
    #[pyfunction]
    fn deep_pointer(base: Address, offsets: Vec<usize>, vm: &VirtualMachine) -> PyResult<Address> {
        let res = unsafe { memory::deep_pointer(base as _, &offsets) };
        let address = res.map_err(|e| vm.new_runtime_error(format!("{e}")))?;

        Ok(address as _)
    }

    /// Allocate memory. Once python object is dropped, memory is automatically deallocated
    #[pyfunction]
    fn alloc(size: usize, prot: PyRef<PyProt>, vm: &VirtualMachine) -> PyResult<PyAlloc> {
        memory::alloc(size, prot.0)
            .map(PyAlloc)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    /// Tries to allocate `size` in a free page somewhere within begin..end address.
    /// begin or end may be NULL, in which case it means "there's no limit".
    /// Once python object is dropped, memory is automatically deallocated
    #[pyfunction]
    fn alloc_in(
        begin: Address,
        end: Address,
        size: usize,
        args: FuncArgs,
        vm: &VirtualMachine,
    ) -> PyResult<PyAlloc> {
        if ![1, 2].contains(&args.args.len()) {
            return Err(vm.new_runtime_error("incorrect number of args".to_owned()));
        }

        let (align, prot) = if args.args.len() == 1 {
            let prot = args
                .args
                .first()
                .unwrap()
                .downcast_ref::<PyProt>()
                .ok_or_else(|| {
                    vm.new_type_error(format!(
                        "expected Prot, found {}",
                        args.args[0].class().__name__(vm)
                    ))
                })?;

            (0, prot)
        } else {
            let align = args.args.first().unwrap().try_to_value::<usize>(vm)?;

            let prot = args
                .args
                .get(1)
                .unwrap()
                .downcast_ref::<PyProt>()
                .ok_or_else(|| {
                    vm.new_type_error(format!(
                        "expected Prot, found {}",
                        args.args[0].class().__name__(vm)
                    ))
                })?;

            (align, prot)
        };

        memory::alloc_in(begin as _, end as _, size, align, prot.0)
            .map(PyAlloc)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    #[pyfunction]
    fn alloc_granularity() -> usize {
        memory::alloc_granularity()
    }

    /// Read size bytes of src into byte array
    ///
    /// unsafe fn
    #[pyfunction]
    fn read(src: Address, size: usize, vm: &VirtualMachine) -> PyObjectRef {
        let bytes = unsafe { memory::read_bytes(src as _, size) };
        let bytes: PyByteArray = bytes.into();

        bytes.to_pyobject(vm)
    }

    /// Set dst address + size to byte
    ///
    /// unsafe fn
    #[pyfunction]
    fn set(dst: Address, byte: u8, size: usize) {
        unsafe {
            memory::set(dst as _, byte, size);
        }
    }

    /// Write bytes to dst address
    ///
    /// unsafe fn
    #[pyfunction]
    fn write(src: Vec<u8>, dst: Address) {
        unsafe {
            memory::write_bytes(&src, dst as _);
        }
    }

    /// Change protection flags on a piece of memory
    ///
    /// unsafe fn
    #[pyfunction]
    fn prot(
        address: Address,
        size: usize,
        prot: PyRef<PyProt>,
        vm: &VirtualMachine,
    ) -> PyResult<PyProt> {
        let prot = unsafe { memory::prot(address as _, size, prot.0) };
        let prot = prot.map_err(|e| vm.new_runtime_error(format!("{e}")))?;
        Ok(PyProt(prot))
    }

    #[pyattr]
    #[pyclass(name = "Alloc")]
    #[derive(Debug, PyPayload)]
    struct PyAlloc(Alloc);

    impl Drop for PyAlloc {
        fn drop(&mut self) {
            let span = trace_span!("drop");
            let _guard = span.enter();
            trace!(address = ?self.0.addr(), "dropping Alloc");
        }
    }

    #[pyclass]
    impl PyAlloc {
        #[pygetset]
        fn address(&self) -> usize {
            self.0.addr() as _
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

    #[pyclass(no_attr, name = "Prot")]
    #[derive(Debug, Copy, Clone, PyPayload)]
    pub struct PyProt(Prot);

    #[pyclass]
    impl PyProt {
        #[pymethod(magic)]
        fn repr(&self) -> String {
            self.0.to_string()
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            self.0.to_string()
        }
    }

    impl From<Prot> for PyProt {
        fn from(prot: Prot) -> Self {
            Self(prot)
        }
    }

    #[pymodule(name = "Prot")]
    pub mod _prot {
        use super::{Prot, PyProt};

        #[pyattr]
        const NONE: PyProt = PyProt(Prot::None);

        #[pyattr]
        const X: PyProt = PyProt(Prot::X);

        #[pyattr]
        const R: PyProt = PyProt(Prot::R);

        #[pyattr]
        const W: PyProt = PyProt(Prot::W);

        #[pyattr]
        const XR: PyProt = PyProt(Prot::XR);

        #[pyattr]
        const XW: PyProt = PyProt(Prot::XW);

        #[pyattr]
        const RW: PyProt = PyProt(Prot::RW);

        #[pyattr]
        const XRW: PyProt = PyProt(Prot::XRW);
    }
}
