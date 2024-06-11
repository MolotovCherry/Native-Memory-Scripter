use rustpython_vm::pymodule;

#[allow(clippy::module_inception)]
#[pymodule]
pub mod asm {
    use mem::asm::Inst;
    use rustpython_vm::{
        builtins::PyByteArray, pyclass, PyObjectRef, PyPayload, PyRef, PyResult, VirtualMachine,
    };

    use crate::modules::Address;

    /// Assemble a single instruction
    #[pyfunction]
    fn assemble(code: String, vm: &VirtualMachine) -> PyResult<PyInst> {
        mem::asm::assemble(&code)
            .map(PyInst)
            .map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    /// Assemble all instructions with a runtime address
    #[pyfunction]
    fn assemble_ex(
        code: String,
        address: Address,
        vm: &VirtualMachine,
    ) -> PyResult<Vec<PyObjectRef>> {
        mem::asm::assemble_ex(&code, address)
            .map(|ins| {
                ins.into_iter()
                    .map(|ins| PyInst(ins).into_pyobject(vm))
                    .collect()
            })
            .map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    /// Get the code length of an instruction(s) starting at address, with a minimum length
    ///
    /// unsafe fn
    #[pyfunction]
    fn code_len(code: Address, min_length: usize, vm: &VirtualMachine) -> PyResult<usize> {
        let res = unsafe { mem::asm::code_len(code as _, min_length) };
        res.map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    /// Disassemble a single instruction at target address
    /// Address must be valid for a 16 byte read
    ///
    /// unsafe fn
    #[pyfunction]
    fn disassemble(addr: Address, vm: &VirtualMachine) -> PyResult<PyInst> {
        let res = unsafe { mem::asm::disassemble(addr as *const _).map(PyInst) };
        res.map_err(|e| vm.new_runtime_error(format!("{e}")))
    }

    #[pyattr]
    #[pyclass(name = "Inst")]
    #[derive(Debug, PyPayload)]
    struct PyInst(Inst);

    #[pyclass]
    impl PyInst {
        #[pygetset]
        fn bytes(&self, vm: &VirtualMachine) -> PyRef<PyByteArray> {
            PyByteArray::new_ref(self.0.bytes.clone(), &vm.ctx)
        }

        #[pygetset]
        fn address(&self) -> usize {
            self.0.address as _
        }

        #[pygetset]
        fn mnemonic(&self) -> Option<String> {
            self.0.mnemonic.clone()
        }

        #[pygetset]
        fn op_str(&self) -> Option<String> {
            self.0.op_str.clone()
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            self.0.to_string()
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            self.0.to_string()
        }
    }
}
