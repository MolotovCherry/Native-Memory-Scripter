#[pyfunction]
fn hook(from: usize, to: usize, vm: &VirtualMachine) -> Option<PyObjectRef> {
    let trampoline = unsafe { libmem::hook_code(from, to) };
    trampoline.map(|t| PyTrampoline(t.address, t.size).into_pyobject(vm))
}

#[pyfunction]
fn unhook_code(from: usize, trampoline: PyRef<PyTrampoline>) -> bool {
    let t = &**trampoline;
    unsafe { libmem::unhook_code(from, t.into()).is_some() }
}

#[pyattr]
#[pyclass(name = "Trampoline")]
#[derive(Debug, Clone, PyPayload)]
struct PyTrampoline(mem::hook::Trampoline);

#[pyclass]
impl PyTrampoline {
    #[pygetset]
    fn address(&self) -> usize {
        self.0.address as _
    }

    #[pygetset]
    fn size(&self) -> usize {
        self.0.size
    }

    #[pymethod(magic)]
    fn repr(&self) -> String {
        format!("{:?}", self.0)
    }

    #[pymethod(magic)]
    fn str(&self) -> String {
        self.repr()
    }
}

impl From<&PyTrampoline> for mem::hook::Trampoline {
    fn from(t: &PyTrampoline) -> Self {
        t.clone().0
    }
}
