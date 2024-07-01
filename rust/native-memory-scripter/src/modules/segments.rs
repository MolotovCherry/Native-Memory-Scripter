use rustpython_vm::pymodule;

#[pymodule]
pub mod segments {
    use mutation::segments::{self, Segment};
    use rustpython_vm::{
        convert::ToPyObject as _, pyclass, PyObjectRef, PyPayload, VirtualMachine,
    };

    use crate::modules::{mem::mem::PyProt, Address};

    #[pyfunction(name = "enum")]
    fn enum_(vm: &VirtualMachine) -> Vec<PyObjectRef> {
        segments::enum_segments()
            .into_iter()
            .map(|segment| PySegment(segment).into_pyobject(vm))
            .collect()
    }

    #[pyfunction]
    fn find(address: Address, vm: &VirtualMachine) -> Option<PyObjectRef> {
        segments::find_segment(address as _).map(|segment| PySegment(segment).to_pyobject(vm))
    }

    #[pyattr]
    #[pyclass(name = "Segment")]
    #[derive(Debug, PyPayload)]
    struct PySegment(Segment);

    #[pyclass]
    impl PySegment {
        #[pygetset]
        fn base(&self) -> Address {
            self.0.base as _
        }

        #[pygetset]
        fn end(&self) -> Address {
            self.0.end as _
        }

        #[pygetset]
        fn size(&self) -> usize {
            self.0.size
        }

        #[pygetset]
        fn prot(&self) -> PyProt {
            self.0.prot.into()
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
