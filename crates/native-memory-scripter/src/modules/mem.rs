use rustpython_vm::pymodule;

#[allow(clippy::module_inception)]
#[pymodule]
pub mod mem {
    use std::{
        fmt::{Debug, Display},
        sync::{Arc, Mutex},
    };

    use libmem::{
        Address, Arch, Inst, Module, Pid, Process, Prot, Segment, Symbol, Thread, Tid, Time, Vmt,
    };
    use libmem_sys::{lm_byte_t, LM_DataScan, LM_ReadMemory, LM_WriteMemory, LM_ADDRESS_BAD};
    use rustpython_vm::{
        builtins::{PyByteArray, PyTypeRef},
        prelude::{VirtualMachine, *},
        pyclass, pymodule,
        types::Constructor,
        PyPayload,
    };

    use crate::{
        modules::cffi::{cffi::Callable, trampoline::Trampoline},
        utils::Sendable,
    };

    #[pyfunction]
    fn alloc_memory(size: usize, prot: PyRef<PyProt>) -> Option<Address> {
        libmem::alloc_memory(size, prot.0)
    }

    #[pyfunction]
    fn assemble(code: String) -> Option<PyInst> {
        libmem::assemble(&code).map(PyInst)
    }

    #[pyfunction]
    fn code_length(code: Address, min_length: usize) -> Option<usize> {
        unsafe { libmem::code_length(code, min_length) }
    }

    #[pyfunction]
    fn data_scan(data: Vec<u8>, address: Address, scan_size: usize) -> Option<Address> {
        let scan = unsafe { LM_DataScan(data.as_ptr(), data.len(), address, scan_size) };

        (scan != LM_ADDRESS_BAD).then_some(scan)
    }

    #[pyfunction]
    fn deep_pointer(base: Address, offsets: Vec<Address>) -> Address {
        unsafe { libmem::deep_pointer::<()>(base, &offsets) as Address }
    }

    #[pyfunction]
    fn demangle_symbol(symbol_name: String) -> Option<String> {
        libmem::demangle_symbol(&symbol_name)
    }

    #[pyfunction]
    fn disassemble(code: Address) -> Option<PyInst> {
        unsafe { libmem::disassemble(code).map(PyInst) }
    }

    #[pyfunction]
    fn enum_modules(vm: &VirtualMachine) -> Option<Vec<PyObjectRef>> {
        libmem::enum_modules().map(|modules| {
            modules
                .into_iter()
                .map(|module| PyModule(module).into_ref(&vm.ctx).into())
                .collect()
        })
    }

    #[pyfunction]
    fn enum_segments(vm: &VirtualMachine) -> Option<Vec<PyObjectRef>> {
        libmem::enum_segments().map(|segments| {
            segments
                .into_iter()
                .map(|segment| PySegment(segment).into_ref(&vm.ctx).into())
                .collect()
        })
    }

    #[pyfunction]
    fn enum_processes(vm: &VirtualMachine) -> Option<Vec<PyObjectRef>> {
        libmem::enum_processes().map(|processes| {
            processes
                .into_iter()
                .map(|process| PyProcess(process).into_ref(&vm.ctx).into())
                .collect()
        })
    }

    #[pyfunction]
    fn enum_symbols(module: PyRef<PyModule>, vm: &VirtualMachine) -> Option<Vec<PyObjectRef>> {
        libmem::enum_symbols(&module.0).map(|symbols| {
            symbols
                .into_iter()
                .map(|symbol| PySymbol(symbol).into_ref(&vm.ctx).into())
                .collect()
        })
    }

    #[pyfunction]
    fn enum_symbols_demangled(
        module: PyRef<PyModule>,
        vm: &VirtualMachine,
    ) -> Option<Vec<PyObjectRef>> {
        libmem::enum_symbols_demangled(&module.0).map(|symbols| {
            symbols
                .into_iter()
                .map(|symbol| PySymbol(symbol).into_ref(&vm.ctx).into())
                .collect()
        })
    }

    #[pyfunction]
    fn enum_threads(vm: &VirtualMachine) -> Option<Vec<PyObjectRef>> {
        libmem::enum_threads().map(|threads| {
            threads
                .into_iter()
                .map(|thread| PyThread(thread).into_ref(&vm.ctx).into())
                .collect()
        })
    }

    #[pyfunction]
    fn find_module(name: String) -> Option<PyModule> {
        libmem::find_module(&name).map(PyModule)
    }

    #[pyfunction]
    fn find_process(name: String) -> Option<PyProcess> {
        libmem::find_process(&name).map(PyProcess)
    }

    #[pyfunction]
    fn find_symbol_address(module: PyRef<PyModule>, symbol_name: String) -> Option<Address> {
        libmem::find_symbol_address(&module.0, &symbol_name)
    }

    #[pyfunction]
    fn find_symbol_address_demangled(
        module: PyRef<PyModule>,
        demangled_symbol_name: String,
    ) -> Option<Address> {
        libmem::find_symbol_address_demangled(&module.0, &demangled_symbol_name)
    }

    #[pyfunction]
    fn find_segment(address: Address, vm: &VirtualMachine) -> Option<PyObjectRef> {
        libmem::find_segment(address).map(|segment| PySegment(segment).into_ref(&vm.ctx).into())
    }

    #[pyfunction]
    fn free_memory(alloc: Address, size: usize) {
        unsafe { libmem::free_memory(alloc, size) }
    }

    #[pyfunction]
    fn get_architecture() -> String {
        ArchDisplay(libmem::get_architecture()).to_string()
    }

    #[pyfunction]
    fn get_process() -> Option<PyProcess> {
        libmem::get_process().map(PyProcess)
    }

    #[pyfunction]
    fn get_bits() -> usize {
        libmem::get_bits().into()
    }

    #[pyfunction]
    fn get_system_bits() -> usize {
        libmem::get_system_bits().into()
    }

    #[pyfunction]
    fn get_thread() -> Option<PyThread> {
        libmem::get_thread().map(PyThread)
    }

    #[pyfunction]
    fn get_thread_process(thread: PyRef<PyThread>) -> Option<PyProcess> {
        libmem::get_thread_process(&thread.0).map(PyProcess)
    }

    #[pyfunction]
    fn hook_code(from: Address, to: Address, vm: &VirtualMachine) -> Option<PyObjectRef> {
        let trampoline = unsafe { libmem::hook_code(from, to) };
        trampoline.map(|t| PyTrampoline(t.address, t.size).into_pyobject(vm))
    }

    #[pyfunction]
    fn is_process_alive(process: PyRef<PyProcess>) -> bool {
        libmem::is_process_alive(&process.0)
    }

    #[pyfunction]
    fn load_module(path: String) -> Option<PyModule> {
        libmem::load_module(&path).map(PyModule)
    }

    #[pyfunction]
    fn pattern_scan(
        pattern: Vec<u8>,
        mask: String,
        address: Address,
        scan_size: usize,
    ) -> Option<Address> {
        unsafe { libmem::pattern_scan(&pattern, &mask, address, scan_size) }
    }

    #[pyfunction]
    fn prot_memory(address: Address, size: usize, prot: PyRef<PyProt>) -> Option<PyProt> {
        let prot = unsafe { libmem::prot_memory(address, size, prot.0) };

        prot.map(PyProt)
    }

    #[pyfunction]
    fn read_memory(src: Address, size: usize, vm: &VirtualMachine) -> Option<PyRef<PyByteArray>> {
        let mut data: Vec<u8> = Vec::with_capacity(size);

        let dst = data.as_mut_ptr() as *mut lm_byte_t;

        if unsafe { LM_ReadMemory(src, dst, size) } == size {
            unsafe {
                data.set_len(size);
            }

            let bytes = PyByteArray::new_ref(data, &vm.ctx);
            Some(bytes)
        } else {
            None
        }
    }

    #[pyfunction]
    fn set_memory(dst: Address, byte: u8, size: usize) {
        unsafe { libmem::set_memory(dst, byte, size) }
    }

    #[pyfunction]
    fn sig_scan(sig: String, addr: Address, scansize: usize) -> Option<Address> {
        unsafe { libmem::sig_scan(&sig, addr, scansize) }
    }

    #[pyfunction]
    fn unhook_code(from: Address, trampoline: PyRef<PyTrampoline>) -> bool {
        let t = &**trampoline;
        unsafe { libmem::unhook_code(from, t.into()).is_some() }
    }

    #[pyfunction]
    fn unload_module(module: PyRef<PyModule>) -> bool {
        libmem::unload_module(&module.0).is_some()
    }

    #[pyfunction]
    fn write_memory(dst: Address, src: Vec<u8>) -> bool {
        let size = src.len();
        let written = unsafe { LM_WriteMemory(dst, src.as_ptr(), size) };
        written == size
    }

    #[pyattr]
    #[pyclass(name = "Vmt")]
    #[derive(PyPayload)]
    struct PyVmt(Address, Sendable<Mutex<Vmt>>);

    impl Debug for PyVmt {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Vmt")
        }
    }

    impl Constructor for PyVmt {
        type Args = Address;

        fn py_new(_cls: PyTypeRef, args: Self::Args, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let vmt = Vmt::new(args);
            let slf = Self(args, Sendable(Mutex::new(vmt)))
                .into_ref(&vm.ctx)
                .into();

            Ok(slf)
        }
    }

    #[pyclass(with(Constructor))]
    impl PyVmt {
        #[pymethod]
        fn hook(&self, index: usize, dst: Address) {
            let mut lock = self.1.lock().unwrap();

            unsafe {
                lock.hook(index, dst);
            }
        }

        #[pymethod]
        fn unhook(&self, index: usize) {
            let mut lock = self.1.lock().unwrap();

            unsafe {
                lock.unhook(index);
            }
        }

        #[pymethod]
        fn get_original(&self, index: usize) -> Option<Address> {
            let lock = self.1.lock().unwrap();

            unsafe { lock.get_original(index) }
        }

        #[pymethod]
        fn reset(&self) {
            let mut lock = self.1.lock().unwrap();

            unsafe {
                lock.reset();
            }
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            format!("Vmt {{ address: {} }}", self.0)
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            self.repr()
        }
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
        fn address(&self) -> Address {
            self.0.address
        }

        #[pygetset]
        fn mnemonic(&self) -> String {
            self.0.mnemonic.clone()
        }

        #[pygetset]
        fn op_str(&self) -> String {
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

    #[pyattr]
    #[pyclass(name = "Module")]
    #[derive(Debug, PyPayload)]
    struct PyModule(Module);

    #[pyclass]
    impl PyModule {
        #[pygetset]
        fn base(&self) -> Address {
            self.0.base
        }

        #[pygetset]
        fn end(&self) -> Address {
            self.0.end
        }

        #[pygetset]
        fn size(&self) -> usize {
            self.0.size
        }

        #[pygetset]
        fn path(&self) -> String {
            self.0.path.clone()
        }

        #[pygetset]
        fn name(&self) -> String {
            self.0.name.clone()
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

    #[pyattr]
    #[pyclass(name = "Segment")]
    #[derive(Debug, PyPayload)]
    struct PySegment(Segment);

    #[pyclass]
    impl PySegment {
        #[pygetset]
        fn base(&self) -> Address {
            self.0.base
        }

        #[pygetset]
        fn end(&self) -> Address {
            self.0.end
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
            self.0.to_string()
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            self.0.to_string()
        }
    }

    #[pyattr]
    #[pyclass(name = "Process")]
    #[derive(Debug, PyPayload)]
    struct PyProcess(Process);

    #[pyclass]
    impl PyProcess {
        #[pygetset]
        fn pid(&self) -> Pid {
            self.0.pid
        }

        #[pygetset]
        fn ppid(&self) -> Pid {
            self.0.ppid
        }

        #[pygetset]
        fn arch(&self) -> String {
            ArchDisplay(self.0.arch).to_string()
        }

        #[pygetset]
        fn bits(&self) -> usize {
            self.0.bits.into()
        }

        #[pygetset]
        fn start_time(&self) -> Time {
            self.0.start_time
        }

        #[pygetset]
        fn path(&self) -> String {
            self.0.path.clone()
        }

        #[pygetset]
        fn name(&self) -> String {
            self.0.name.clone()
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

    #[pyattr]
    #[pyclass(name = "Symbol")]
    #[derive(PyPayload)]
    struct PySymbol(Symbol);

    impl Debug for PySymbol {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[pyclass]
    impl PySymbol {
        #[pygetset]
        fn name(&self) -> String {
            self.0.name.clone()
        }

        #[pygetset]
        fn address(&self) -> Address {
            self.0.address
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

    #[pyattr]
    #[pyclass(name = "Thread")]
    #[derive(Debug, PyPayload)]
    struct PyThread(Thread);

    #[pyclass]
    impl PyThread {
        #[pygetset]
        fn tid(&self) -> Tid {
            self.0.tid
        }

        #[pygetset]
        fn owner_pid(&self) -> Pid {
            self.0.owner_pid
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

    #[pyattr]
    #[pyclass(name = "Trampoline")]
    #[derive(Debug, Clone, PyPayload)]
    struct PyTrampoline(Address, usize);

    #[pyclass]
    impl PyTrampoline {
        #[pygetset]
        fn address(&self) -> Address {
            self.0
        }

        #[pygetset]
        fn size(&self) -> usize {
            self.1
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            format!("Trampoline {{ address: {}, size: {} }}", self.0, self.1)
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            self.repr()
        }
    }

    impl From<&PyTrampoline> for libmem::Trampoline {
        fn from(t: &PyTrampoline) -> Self {
            Self {
                address: t.0,
                size: t.1,
            }
        }
    }

    #[pyclass(no_attr, name = "Prot")]
    #[derive(Debug, Copy, Clone, PyPayload)]
    struct PyProt(Prot);

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

    struct ArchDisplay(Arch);
    impl Display for ArchDisplay {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let arch = match self.0 {
                Arch::ARMV7 => "ARMV7",
                Arch::ARMV8 => "ARMV8",
                Arch::THUMBV7 => "THUMBV7",
                Arch::THUMBV8 => "THUMBV8",
                Arch::ARMV7EB => "ARMV7EB",
                Arch::THUMBV7EB => "THUMBV7EB",
                Arch::ARMV8EB => "ARMV8EB",
                Arch::THUMBV8EB => "THUMBV8EB",
                Arch::AARCH64 => "AARCH64",
                Arch::MIPS => "MIPS",
                Arch::MIPS64 => "MIPS64",
                Arch::MIPSEL => "MIPSEL",
                Arch::MIPSEL64 => "MIPSEL64",
                Arch::X86_16 => "X86_16",
                Arch::X86 => "X86",
                Arch::X64 => "X64",
                Arch::PPC32 => "PPC32",
                Arch::PPC64 => "PPC64",
                Arch::PPC64LE => "PPC64LE",
                Arch::SPARC => "SPARC",
                Arch::SPARC64 => "SPARC64",
                Arch::SPARCEL => "SPARCEL",
                Arch::SYSZ => "SYSZ",
            };

            write!(f, "{arch}")
        }
    }
}
