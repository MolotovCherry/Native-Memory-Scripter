// #[pyattr]
// #[pyclass(name = "Vmt")]
// #[derive(PyPayload)]
// struct PyVmt(usize, Sendable<Mutex<Vmt>>);

// impl Debug for PyVmt {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Vmt")
//     }
// }

// impl Constructor for PyVmt {
//     type Args = usize;

//     fn py_new(_cls: PyTypeRef, args: Self::Args, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
//         let vmt = Vmt::new(args);
//         let slf = Self(args, Sendable(Mutex::new(vmt))).into_pyobject(vm);

//         Ok(slf)
//     }
// }

// #[pyclass(with(Constructor))]
// impl PyVmt {
//     #[pymethod]
//     fn hook(&self, index: usize, dst: usize) {
//         let mut lock = self.1.lock().unwrap();

//         unsafe {
//             lock.hook(index, dst);
//         }
//     }

//     #[pymethod]
//     fn unhook(&self, index: usize) {
//         let mut lock = self.1.lock().unwrap();

//         unsafe {
//             lock.unhook(index);
//         }
//     }

//     #[pymethod]
//     fn get_original(&self, index: usize) -> Option<usize> {
//         let lock = self.1.lock().unwrap();

//         unsafe { lock.get_original(index) }
//     }

//     #[pymethod]
//     fn reset(&self) {
//         let mut lock = self.1.lock().unwrap();

//         unsafe {
//             lock.reset();
//         }
//     }

//     #[pymethod(magic)]
//     fn repr(&self) -> String {
//         format!("Vmt {{ address: {} }}", self.0)
//     }

//     #[pymethod(magic)]
//     fn str(&self) -> String {
//         self.repr()
//     }
// }
