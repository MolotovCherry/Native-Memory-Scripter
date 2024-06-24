use rustpython_vm::pymodule;

#[pymodule]
pub mod asm {
    use mem::asm::Inst;
    use rustpython_vm::{
        builtins::PyByteArray, convert::ToPyObject, function::FuncArgs, pyclass, PyObjectRef,
        PyPayload, PyRef, PyResult, VirtualMachine,
    };

    use crate::modules::Address;

    /// Assemble instructions
    ///
    /// Calling modes:
    /// String (code) -> Inst
    /// Assemble a single instruction to machine code
    ///
    /// String (code), Address (runtime address) -> [Inst]
    /// Assembles multiple instructions to machine code
    ///
    #[pyfunction]
    fn assemble(args: FuncArgs, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        // String
        // String, Address

        let code = args
            .args
            .first()
            .map(|s| s.try_to_value::<String>(vm))
            .transpose()?
            .ok_or_else(|| vm.new_runtime_error("code argument not found".to_owned()))?;

        let obj = match args.args.len() {
            1 => mem::asm::assemble(&code)
                .map(PyInst)
                .map_err(|e| vm.new_runtime_error(format!("{e}")))?
                .to_pyobject(vm),

            2 => {
                let addr = args
                    .args
                    .get(1)
                    .map(|v| v.try_to_value::<Address>(vm))
                    .transpose()?
                    .unwrap();

                mem::asm::assemble_ex(&code, addr)
                    .map(|v| v.into_iter().map(|i| PyInst(i).to_pyobject(vm)))
                    .map_err(|e| vm.new_runtime_error(format!("{e}")))?
                    .collect::<Vec<_>>()
                    .to_pyobject(vm)
            }

            _ => {
                return Err(vm
                    .new_runtime_error(format!("expected 1 or 2 args, found {}", args.args.len())))
            }
        };

        Ok(obj)
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
    fn disassemble(args: FuncArgs, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
        // disassemble bytes:
        // Bytes
        // Bytes, runtime_addr(Address)
        // Bytes, runtime_addr(Address), count(usize)

        // disassemble address:
        // Address
        // Address, size (usize), runtime_addr(Address)
        // Address, size (usize), runtime_addr(Address), count(usize)

        let address = args
            .args
            .first()
            .map(|s| s.try_to_value::<Address>(vm))
            .transpose()?
            .ok_or_else(|| vm.new_runtime_error("address argument not found".to_owned()));

        if let Ok(address) = address {
            let obj = match args.args.len() {
                1 => {
                    let res = unsafe { mem::asm::disassemble(address as *const _) };
                    let res = res.map_err(|e| vm.new_runtime_error(format!("{e}")))?;

                    PyInst(res).to_pyobject(vm)
                }

                3 => {
                    let size = args
                        .args
                        .get(1)
                        .cloned()
                        .map(|i| i.try_into_value::<usize>(vm))
                        .transpose()?
                        .unwrap();

                    let runtime_addr = args
                        .args
                        .get(2)
                        .cloned()
                        .map(|i| i.try_into_value::<Address>(vm))
                        .transpose()?
                        .unwrap();

                    let res = unsafe {
                        mem::asm::disassemble_ex(address as *const _, size, runtime_addr)
                    };

                    let res = res.map_err(|e| vm.new_runtime_error(format!("{e}")))?;
                    let insts = res
                        .into_iter()
                        .map(|d| PyInst(d).to_pyobject(vm))
                        .collect::<Vec<_>>();

                    insts.to_pyobject(vm)
                }

                4 => {
                    let size = args
                        .args
                        .get(1)
                        .cloned()
                        .map(|i| i.try_into_value::<usize>(vm))
                        .transpose()?
                        .unwrap();

                    let runtime_addr = args
                        .args
                        .get(2)
                        .cloned()
                        .map(|i| i.try_into_value::<Address>(vm))
                        .transpose()?
                        .unwrap();

                    let inst_count = args
                        .args
                        .get(3)
                        .cloned()
                        .map(|i| i.try_into_value::<usize>(vm))
                        .transpose()?
                        .unwrap();

                    let res = unsafe {
                        mem::asm::disassemble_ex_count(
                            address as *const _,
                            size,
                            runtime_addr,
                            inst_count,
                        )
                    };

                    let res = res.map_err(|e| vm.new_runtime_error(format!("{e}")))?;
                    let insts = res
                        .into_iter()
                        .map(|d| PyInst(d).to_pyobject(vm))
                        .collect::<Vec<_>>();

                    insts.to_pyobject(vm)
                }

                _ => {
                    return Err(vm.new_runtime_error(format!(
                        "expected 1, 3, or 4 args, found {}",
                        args.args.len()
                    )))
                }
            };

            Ok(obj)
        } else {
            let bytes = args
                .args
                .first()
                .cloned()
                .map(|s| s.try_into_value::<Vec<u8>>(vm))
                .transpose()?
                .ok_or_else(|| vm.new_runtime_error("address argument not found".to_owned()))?;

            let obj = match args.args.len() {
                1 => {
                    let res = mem::asm::disassemble_bytes(&bytes);
                    let res = res.map_err(|e| vm.new_runtime_error(format!("{e}")))?;
                    let res = res
                        .into_iter()
                        .map(|i| PyInst(i).to_pyobject(vm))
                        .collect::<Vec<_>>();

                    res.to_pyobject(vm)
                }

                2 => {
                    let runtime_addr = args
                        .args
                        .get(1)
                        .cloned()
                        .map(|i| i.try_into_value::<usize>(vm))
                        .transpose()?
                        .unwrap();

                    let res = mem::asm::disassemble_bytes_ex(&bytes, runtime_addr);
                    let res = res.map_err(|e| vm.new_runtime_error(format!("{e}")))?;
                    let res = res
                        .into_iter()
                        .map(|i| PyInst(i).to_pyobject(vm))
                        .collect::<Vec<_>>();

                    res.to_pyobject(vm)
                }

                3 => {
                    let runtime_addr = args
                        .args
                        .get(1)
                        .cloned()
                        .map(|i| i.try_into_value::<Address>(vm))
                        .transpose()?
                        .unwrap();

                    let inst_count = args
                        .args
                        .get(2)
                        .cloned()
                        .map(|i| i.try_into_value::<usize>(vm))
                        .transpose()?
                        .unwrap();

                    let res =
                        mem::asm::disassemble_bytes_ex_count(&bytes, runtime_addr, inst_count);

                    let res = res.map_err(|e| vm.new_runtime_error(format!("{e}")))?;
                    let insts = res
                        .into_iter()
                        .map(|d| PyInst(d).to_pyobject(vm))
                        .collect::<Vec<_>>();

                    insts.to_pyobject(vm)
                }

                _ => {
                    return Err(vm.new_runtime_error(format!(
                        "expected 1, 2 or 3 args, found {}",
                        args.args.len()
                    )))
                }
            };

            Ok(obj)
        }
    }

    #[pyattr]
    #[pyclass(name = "Inst")]
    #[derive(Debug, Clone, PyPayload)]
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
