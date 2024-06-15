use rustpython_vm::pymodule;

#[allow(clippy::module_inception)]
#[pymodule]
pub mod mem {
    use std::fmt::Debug;

    use mem::{memory::Alloc, Prot};
    use rustpython_vm::{
        builtins::PyByteArray, convert::ToPyObject as _, prelude::*, pyclass, pymodule, PyPayload,
        VirtualMachine,
    };

    use crate::modules::Address;

    // todo: implement this
    // #[pyfunction]
    // fn deep_pointer(base: usize, offsets: Vec<usize>) -> usize {
    //     unsafe { libmem::deep_pointer::<()>(base, &offsets) as usize }
    // }

    /// Allocate memory. Once python object is dropped, memory is automatically deallocated
    #[pyfunction]
    fn alloc(size: usize, prot: PyRef<PyProt>, vm: &VirtualMachine) -> PyResult<PyAlloc> {
        mem::memory::alloc(size, prot.0)
            .map(PyAlloc)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    /// Read size bytes of src into byte array
    ///
    /// unsafe fn
    #[pyfunction]
    fn read(src: Address, size: usize, vm: &VirtualMachine) -> PyObjectRef {
        let bytes = unsafe { mem::memory::read_bytes(src as _, size) };
        let bytes: PyByteArray = bytes.into();

        bytes.to_pyobject(vm)
    }

    /// Set dst address + size to byte
    ///
    /// unsafe fn
    #[pyfunction]
    fn set(dst: Address, byte: u8, size: usize) {
        unsafe {
            mem::memory::set(dst as _, byte, size);
        }
    }

    /// Write bytes to dst address
    ///
    /// unsafe fn
    #[pyfunction]
    fn write(src: Vec<u8>, dst: Address) {
        unsafe {
            mem::memory::write_bytes(&src, dst as _);
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
        let prot = unsafe { mem::memory::prot(address as _, size, prot.0) };
        let prot = prot.map_err(|e| vm.new_runtime_error(format!("{e}")))?;
        Ok(PyProt(prot))
    }

    #[pyattr]
    #[pyclass(name = "Alloc")]
    #[derive(Debug, PyPayload)]
    struct PyAlloc(Alloc);

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
