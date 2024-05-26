use rustpython_vm::pymodule;

#[allow(clippy::module_inception)]
#[pymodule]
pub mod mem {
    use std::ptr::NonNull;

    use libmem::{lm_address_t, lm_inst_t, lm_prot_t, lm_size_t, LM_AllocMemory, LM_Assemble};
    use rustpython_vm::{
        builtins::PyByteArray, prelude::*, pyclass, PyPayload, TryFromBorrowedObject,
    };

    #[pyfunction]
    fn alloc_memory(size: lm_size_t, prot: py_lm_prot_t) -> Option<lm_address_t> {
        LM_AllocMemory(size, prot.0)
    }

    #[pyfunction]
    fn assemble(code: String) -> Option<py_lm_inst_t> {
        LM_Assemble(&code).map(|inst| py_lm_inst_t(Opaque::new(inst)))
    }

    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(module = "mem", name = "lm_inst_t")]
    #[derive(Debug, PyPayload)]
    struct py_lm_inst_t(Opaque);

    #[pyclass]
    impl py_lm_inst_t {
        #[pymethod]
        fn get_bytes(&self, vm: &VirtualMachine) -> PyResult<PyRef<PyByteArray>> {
            let data: &lm_inst_t = unsafe { self.0.as_ref() };
            let bytes = data.get_bytes();

            let bytes = PyByteArray::new_ref(bytes.to_owned(), &vm.ctx);

            Ok(bytes)
        }
    }

    impl Drop for py_lm_inst_t {
        fn drop(&mut self) {
            unsafe {
                self.0.drop::<lm_inst_t>();
            }
        }
    }

    #[allow(non_camel_case_types)]
    struct py_lm_prot_t(lm_prot_t);
    impl<'a> TryFromBorrowedObject<'a> for py_lm_prot_t {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &'a PyObject) -> PyResult<Self> {
            let value: u8 = obj.try_int(vm)?.try_to_primitive(vm)?;

            let prot = match value {
                0b000 => lm_prot_t::LM_PROT_NONE,
                0b001 => lm_prot_t::LM_PROT_X,
                0b010 => lm_prot_t::LM_PROT_R,
                0b100 => lm_prot_t::LM_PROT_W,
                0b011 => lm_prot_t::LM_PROT_XR,
                0b101 => lm_prot_t::LM_PROT_XW,
                0b110 => lm_prot_t::LM_PROT_RW,
                0b111 => lm_prot_t::LM_PROT_XRW,

                _ => return Err(vm.new_value_error(format!("{value} is not a valid lm_prot_t"))),
            };

            Ok(Self(prot))
        }
    }

    /// An Opaque pointer which can be casted back to the original data type
    #[pyclass(name, no_attr)]
    #[derive(Debug)]
    struct Opaque(NonNull<()>);
    unsafe impl Send for Opaque {}
    unsafe impl Sync for Opaque {}

    #[pyclass]
    impl Opaque {
        fn new<T>(t: T) -> Self {
            let ptr = Box::into_raw(Box::new(t)).cast();
            Self(NonNull::new(ptr).unwrap())
        }

        /// SAFETY: No other unique refs can exist anywhere when you call this
        unsafe fn as_ref<T>(&self) -> &T {
            let ptr: *mut T = self.0.as_ptr().cast();
            unsafe { &*ptr }
        }

        /// SAFETY: No other unique or shared refs can exist anywhere when you call this
        unsafe fn as_mut<T>(&mut self) -> &mut T {
            let ptr: *mut T = self.0.as_ptr().cast();
            unsafe { &mut *ptr }
        }

        unsafe fn into<T>(self) -> Box<T> {
            unsafe { Box::from_raw(self.0.as_ptr().cast()) }
        }

        /// SAFETY: There must be no calls to any other functions after this
        ///         as the inside pointer is no longer valid
        unsafe fn drop<T>(&mut self) {
            unsafe {
                _ = Box::from_raw(self.0.as_ptr().cast::<T>());
            }
        }
    }
}
