#[pyfunction]
fn enum_segments(vm: &VirtualMachine) -> Vec<PyObjectRef> {
    mem::segment::enum_segments()
        .into_iter()
        .map(|segment| PySegment(segment).into_pyobject(vm))
        .collect()
}

#[pyfunction]
fn find_segment(address: usize, vm: &VirtualMachine) -> Option<PyObjectRef> {
    mem::segment::find_segment(address as _).map(|segment| PySegment(segment).to_pyobject(vm))
}

#[pyattr]
#[pyclass(name = "Segment")]
#[derive(Debug, PyPayload)]
struct PySegment(Segment);

#[pyclass]
impl PySegment {
    #[pygetset]
    fn base(&self) -> usize {
        self.0.base as _
    }

    #[pygetset]
    fn end(&self) -> usize {
        self.0.end as _
    }

    #[pygetset]
    fn size(&self) -> usize {
        self.0.size
    }

    #[pygetset]
    fn prot(&self) -> PyProt {
        PyProt(self.0.prot)
    }

    #[pymethod(magic)]
    fn repr(&self) -> String {
        format!("{self:?}")
    }

    #[pymethod(magic)]
    fn str(&self) -> String {
        format!("{self:?}")
    }
}
