use rustpython_vm::pymodule;

use libmem::{lm_address_t, lm_byte_t, lm_size_t};

// TODO: Remove and replace with libmem-sys once it comes out
#[link(name = "libmem", kind = "static")]
extern "C" {
    fn LM_ReadMemory(src: lm_address_t, dst: *mut lm_byte_t, size: lm_size_t) -> lm_size_t;

    fn LM_WriteMemory(dst: lm_address_t, src: *const lm_byte_t, size: lm_size_t) -> lm_size_t;
}

#[allow(clippy::module_inception)]
#[pymodule]
pub mod mem {
    use std::ptr::NonNull;

    use libmem::{
        lm_address_t, lm_byte_t, lm_inst_t, lm_module_t, lm_page_t, lm_pid_t, lm_process_t,
        lm_prot_t, lm_size_t, lm_symbol_t, lm_thread_t, lm_tid_t, lm_vmt_t, LM_AllocMemory,
        LM_Assemble, LM_CodeLength, LM_DataScan, LM_DemangleSymbol, LM_Disassemble, LM_EnumModules,
        LM_EnumPages, LM_EnumProcesses, LM_EnumSymbols, LM_EnumSymbolsDemangled, LM_EnumThreads,
        LM_FindModule, LM_FindProcess, LM_FindSymbolAddress, LM_FindSymbolAddressDemangled,
        LM_FreeMemory, LM_GetPage, LM_GetProcess, LM_GetSystemBits, LM_GetThread,
        LM_GetThreadProcess, LM_HookCode, LM_IsProcessAlive, LM_LoadModule, LM_PatternScan,
        LM_ProtMemory, LM_SetMemory, LM_SigScan, LM_UnhookCode, LM_UnloadModule,
    };
    use rustpython_vm::{
        builtins::{PyByteArray, PyTypeRef},
        function::FuncArgs,
        prelude::{VirtualMachine, *},
        pyclass,
        types::Constructor,
        PyPayload, TryFromBorrowedObject,
    };

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_AllocMemory.md
    #[pyfunction]
    fn alloc_memory(size: lm_size_t, prot: py_lm_prot_t) -> Option<lm_address_t> {
        LM_AllocMemory(size, prot.0)
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_Assemble.md
    #[pyfunction]
    fn assemble(code: String) -> Option<py_lm_inst_t> {
        LM_Assemble(&code).map(|inst| py_lm_inst_t(Opaque::new(inst)))
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_CodeLength.md
    #[pyfunction]
    fn code_length(code: lm_address_t, minlength: lm_size_t) -> Option<lm_size_t> {
        unsafe { LM_CodeLength(code, minlength) }
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_DataScan.md
    #[pyfunction]
    fn data_scan(data: Vec<u8>, addr: lm_address_t, scansize: lm_size_t) -> Option<lm_address_t> {
        unsafe { LM_DataScan(&data, addr, scansize) }
    }

    // TODO: Implement when new version comes out
    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_DeepPointer.md
    // #[pyfunction]
    // fn deep_pointer(base: lm_address_t, offsets: Vec<lm_address_t>) -> Option<lm_address_t> {
    //     unsafe { LM_DeepPointer::<()>(base, offsets) }
    // }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_DemangleSymbol.md
    #[pyfunction]
    fn demangle_symbol(symbol: String) -> Option<String> {
        LM_DemangleSymbol(&symbol)
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_Disassemble.md
    #[pyfunction]
    fn disassemble(code: lm_address_t) -> Option<py_lm_inst_t> {
        unsafe { LM_Disassemble(code).map(|inst| py_lm_inst_t(Opaque::new(inst))) }
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_EnumModules.md
    #[pyfunction]
    fn enum_modules(vm: &VirtualMachine) -> Option<Vec<PyObjectRef>> {
        let modules = LM_EnumModules();

        modules.map(|modules| {
            modules
                .into_iter()
                .map(|module| py_lm_module_t(Opaque::new(module)).into_ref(&vm.ctx).into())
                .collect()
        })
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_EnumPages.md
    #[pyfunction]
    fn enum_pages(vm: &VirtualMachine) -> Option<Vec<PyObjectRef>> {
        let pages = LM_EnumPages();

        pages.map(|pages| {
            pages
                .into_iter()
                .map(|page| py_lm_page_t(Opaque::new(page)).into_ref(&vm.ctx).into())
                .collect()
        })
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_EnumProcesses.md
    #[pyfunction]
    fn enum_processes(vm: &VirtualMachine) -> Option<Vec<PyObjectRef>> {
        let processes = LM_EnumProcesses();

        processes.map(|pages| {
            pages
                .into_iter()
                .map(|process| {
                    py_lm_process_t(Opaque::new(process))
                        .into_ref(&vm.ctx)
                        .into()
                })
                .collect()
        })
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_EnumSymbols.md
    #[pyfunction]
    fn enum_symbols(pmod: &py_lm_module_t, vm: &VirtualMachine) -> Option<Vec<PyObjectRef>> {
        let module: &lm_module_t = unsafe { pmod.0.as_ref() };
        let symbols = LM_EnumSymbols(module);

        symbols.map(|symbols| {
            symbols
                .into_iter()
                .map(|symbol| py_lm_symbol_t(Opaque::new(symbol)).into_ref(&vm.ctx).into())
                .collect()
        })
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_EnumSymbolsDemangled.md
    #[pyfunction]
    fn enum_symbols_demangled(
        pmod: &py_lm_module_t,
        vm: &VirtualMachine,
    ) -> Option<Vec<PyObjectRef>> {
        let module: &lm_module_t = unsafe { pmod.0.as_ref() };
        let symbols = LM_EnumSymbolsDemangled(module);

        symbols.map(|symbols| {
            symbols
                .into_iter()
                .map(|symbol| py_lm_symbol_t(Opaque::new(symbol)).into_ref(&vm.ctx).into())
                .collect()
        })
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_EnumThreads.md
    #[pyfunction]
    fn enum_threads(vm: &VirtualMachine) -> Option<Vec<PyObjectRef>> {
        let threads = LM_EnumThreads();

        threads.map(|threads| {
            threads
                .into_iter()
                .map(|thread| py_lm_thread_t(Opaque::new(thread)).into_ref(&vm.ctx).into())
                .collect()
        })
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_FindModule.md
    #[pyfunction]
    fn find_module(name: String) -> Option<py_lm_module_t> {
        let module = LM_FindModule(&name);
        module.map(|module| py_lm_module_t(Opaque::new(module)))
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_FindProcess.md
    #[pyfunction]
    fn find_process(procstr: String) -> Option<py_lm_process_t> {
        let process = LM_FindProcess(&procstr);
        process.map(|process| py_lm_process_t(Opaque::new(process)))
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_FindSymbolAddress.md
    #[pyfunction]
    fn find_symbol_address(pmod: &py_lm_module_t, name: String) -> Option<lm_address_t> {
        let module: &lm_module_t = unsafe { pmod.0.as_ref() };
        LM_FindSymbolAddress(module, &name)
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_FindSymbolAddressDemangled.md
    #[pyfunction]
    fn find_symbol_address_demangled(pmod: &py_lm_module_t, name: String) -> Option<lm_address_t> {
        let module: &lm_module_t = unsafe { pmod.0.as_ref() };
        LM_FindSymbolAddressDemangled(module, &name)
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_FreeMemory.md
    #[pyfunction]
    fn free_memory(alloc: lm_address_t, size: lm_size_t) -> bool {
        unsafe { LM_FreeMemory(alloc, size).is_some() }
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_GetPage.md
    #[pyfunction]
    fn get_page(addr: lm_address_t) -> Option<py_lm_page_t> {
        LM_GetPage(addr).map(|page| py_lm_page_t(Opaque::new(page)))
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_GetProcess.md
    #[pyfunction]
    fn get_process() -> Option<py_lm_process_t> {
        LM_GetProcess().map(|process| py_lm_process_t(Opaque::new(process)))
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_GetSystemBits.md
    #[pyfunction]
    fn get_system_bits() -> lm_size_t {
        LM_GetSystemBits()
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_GetThread.md
    #[pyfunction]
    fn get_thread() -> Option<py_lm_thread_t> {
        LM_GetThread().map(|thread| py_lm_thread_t(Opaque::new(thread)))
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_GetThreadProcess.md
    #[pyfunction]
    fn get_thread_process(pthr: &py_lm_thread_t) -> Option<py_lm_process_t> {
        let thread: &lm_thread_t = unsafe { pthr.0.as_ref() };
        LM_GetThreadProcess(thread).map(|process| py_lm_process_t(Opaque::new(process)))
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_HookCode.md
    #[pyfunction]
    fn hook_code(from: lm_address_t, to: lm_address_t) -> Option<(lm_address_t, lm_size_t)> {
        unsafe { LM_HookCode(from, to) }
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_IsProcessAlive.md
    #[pyfunction]
    fn is_process_alive(pproc: &py_lm_process_t) -> bool {
        let process: &lm_process_t = unsafe { pproc.0.as_ref() };
        LM_IsProcessAlive(process)
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_LoadModule.md
    #[pyfunction]
    fn load_module(modpath: String) -> Option<py_lm_module_t> {
        LM_LoadModule(&modpath).map(|module| py_lm_module_t(Opaque::new(module)))
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_PatternScan.md
    #[pyfunction]
    fn pattern_scan(
        pattern: Vec<u8>,
        mask: String,
        addr: lm_address_t,
        scansize: lm_size_t,
    ) -> Option<lm_address_t> {
        unsafe { LM_PatternScan(&pattern, &mask, addr, scansize) }
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_ProtMemory.md
    #[pyfunction]
    fn prot_memory(
        addr: lm_address_t,
        size: lm_size_t,
        prot: py_lm_prot_t,
        vm: &VirtualMachine,
    ) -> PyResult<Option<PyObjectRef>> {
        let prot = unsafe { LM_ProtMemory(addr, size, prot.0) };

        prot.map(|prot| py_lm_prot_t::to_pyobject(prot, vm))
            .transpose()
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_ReadMemory.md
    #[pyfunction]
    fn read_memory(
        src: lm_address_t,
        size: lm_size_t,
        vm: &VirtualMachine,
    ) -> Option<PyRef<PyByteArray>> {
        let mut data: Vec<u8> = Vec::with_capacity(size);

        let dst = data.as_mut_ptr() as *mut lm_byte_t;

        if unsafe { super::LM_ReadMemory(src, dst, size) } == size {
            unsafe {
                data.set_len(size);
            }

            let bytes = PyByteArray::new_ref(data, &vm.ctx);
            Some(bytes)
        } else {
            None
        }
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_SetMemory.md
    #[pyfunction]
    fn set_memory(dst: lm_address_t, byte: lm_byte_t, size: lm_size_t) -> bool {
        unsafe { LM_SetMemory(dst, byte, size).is_some() }
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_SigScan.md
    #[pyfunction]
    fn sig_scan(sig: String, addr: lm_address_t, scansize: lm_size_t) -> Option<lm_address_t> {
        unsafe { LM_SigScan(&sig, addr, scansize) }
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_UnhookCode.md
    #[pyfunction]
    fn unhook_code(from: lm_address_t, trampoline: (lm_address_t, lm_size_t)) -> bool {
        unsafe { LM_UnhookCode(from, trampoline).is_some() }
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_UnhookCode.md
    #[pyfunction]
    fn unload_module(pmod: &py_lm_module_t) -> bool {
        let module = unsafe { pmod.0.as_ref() };
        LM_UnloadModule(module).is_some()
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/LM_WriteMemory.md
    #[pyfunction]
    fn write_memory(dst: lm_address_t, src: Vec<u8>) -> bool {
        let size = src.len();
        let written = unsafe { super::LM_WriteMemory(dst, src.as_ptr(), size) };
        written == size
    }

    /// https://github.com/rdbo/libmem/blob/master/docs/rust/VMT.md
    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(name = "Vmt")]
    #[derive(Debug, PyPayload)]
    struct Vmt(Opaque);

    impl Constructor for Vmt {
        type Args = lm_address_t;

        fn py_new(_cls: PyTypeRef, args: Self::Args, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let ptr = args as *mut lm_address_t;
            let vmt = lm_vmt_t::new(ptr);
            let slf = Self(Opaque::new(vmt)).into_ref(&vm.ctx).into();
            Ok(slf)
        }
    }

    #[pyclass(with(Constructor))]
    impl Vmt {
        #[pymethod]
        fn hook(&self, index: lm_size_t, dst: lm_address_t) {
            let this: &mut lm_vmt_t = unsafe { self.0.as_mut() };
            unsafe {
                this.hook(index, dst);
            }
        }

        #[pymethod]
        fn unhook(&self, index: lm_size_t) {
            let this: &mut lm_vmt_t = unsafe { self.0.as_mut() };
            unsafe {
                this.unhook(index);
            }
        }

        #[pymethod]
        fn get_original(&self, index: lm_size_t) -> Option<lm_address_t> {
            let this: &lm_vmt_t = unsafe { self.0.as_ref() };
            unsafe { this.get_original(index) }
        }

        #[pymethod]
        fn reset(&self) {
            let this: &mut lm_vmt_t = unsafe { self.0.as_mut() };
            unsafe {
                this.reset();
            }
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            let data: &lm_vmt_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            let data: &lm_vmt_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }
    }

    impl Drop for Vmt {
        fn drop(&mut self) {
            unsafe { self.0.drop::<lm_vmt_t>() }
        }
    }

    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(name = "inst")]
    #[derive(Debug, PyPayload)]
    struct py_lm_inst_t(Opaque);

    #[pyclass]
    impl py_lm_inst_t {
        #[pygetset]
        fn bytes(&self, vm: &VirtualMachine) -> PyRef<PyByteArray> {
            let data: &lm_inst_t = unsafe { self.0.as_ref() };
            let bytes = data.get_bytes();

            PyByteArray::new_ref(bytes.to_owned(), &vm.ctx)
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            let data: &lm_inst_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            let data: &lm_inst_t = unsafe { self.0.as_ref() };
            format!("{data}")
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
    #[pyattr]
    #[pyclass(name = "module")]
    #[derive(Debug, PyPayload)]
    struct py_lm_module_t(Opaque);

    #[pyclass]
    impl py_lm_module_t {
        #[pygetset]
        fn base(&self) -> lm_address_t {
            let data: &lm_module_t = unsafe { self.0.as_ref() };
            data.get_base()
        }

        #[pygetset]
        fn end(&self) -> lm_address_t {
            let data: &lm_module_t = unsafe { self.0.as_ref() };
            data.get_end()
        }

        #[pygetset]
        fn size(&self) -> lm_address_t {
            let data: &lm_module_t = unsafe { self.0.as_ref() };
            data.get_size()
        }

        #[pygetset]
        fn path(&self) -> String {
            let data: &lm_module_t = unsafe { self.0.as_ref() };
            data.get_path()
        }

        #[pygetset]
        fn name(&self) -> String {
            let data: &lm_module_t = unsafe { self.0.as_ref() };
            data.get_name()
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            let data: &lm_module_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            let data: &lm_module_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }
    }

    impl Drop for py_lm_module_t {
        fn drop(&mut self) {
            unsafe {
                self.0.drop::<lm_module_t>();
            }
        }
    }

    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(name = "page")]
    #[derive(Debug, PyPayload)]
    struct py_lm_page_t(Opaque);

    #[pyclass]
    impl py_lm_page_t {
        #[pygetset]
        fn base(&self) -> lm_address_t {
            let data: &lm_page_t = unsafe { self.0.as_ref() };
            data.get_base()
        }

        #[pygetset]
        fn end(&self) -> lm_address_t {
            let data: &lm_page_t = unsafe { self.0.as_ref() };
            data.get_end()
        }

        #[pygetset]
        fn size(&self) -> lm_size_t {
            let data: &lm_page_t = unsafe { self.0.as_ref() };
            data.get_size()
        }

        #[pygetset]
        fn prot(&self, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let data: &lm_page_t = unsafe { self.0.as_ref() };
            let prot = data.get_prot();

            py_lm_prot_t::to_pyobject(prot, vm)
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            let data: &lm_page_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            let data: &lm_page_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }
    }

    impl Drop for py_lm_page_t {
        fn drop(&mut self) {
            unsafe {
                self.0.drop::<lm_page_t>();
            }
        }
    }

    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(name = "process")]
    #[derive(Debug, PyPayload)]
    struct py_lm_process_t(Opaque);

    #[pyclass]
    impl py_lm_process_t {
        #[pygetset]
        fn pid(&self) -> lm_pid_t {
            let data: &lm_process_t = unsafe { self.0.as_ref() };
            data.get_pid()
        }

        #[pygetset]
        fn ppid(&self) -> lm_pid_t {
            let data: &lm_process_t = unsafe { self.0.as_ref() };
            data.get_ppid()
        }

        #[pygetset]
        fn bits(&self) -> lm_size_t {
            let data: &lm_process_t = unsafe { self.0.as_ref() };
            data.get_bits()
        }

        // lm_time_t is inexplicably private right now
        #[pygetset]
        fn start_time(&self) -> u64 {
            let data: &lm_process_t = unsafe { self.0.as_ref() };
            data.get_start_time()
        }

        #[pygetset]
        fn path(&self) -> String {
            let data: &lm_process_t = unsafe { self.0.as_ref() };
            data.get_path()
        }

        #[pygetset]
        fn name(&self) -> String {
            let data: &lm_process_t = unsafe { self.0.as_ref() };
            data.get_name()
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            let data: &lm_process_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            let data: &lm_process_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }
    }

    impl Drop for py_lm_process_t {
        fn drop(&mut self) {
            unsafe {
                self.0.drop::<lm_process_t>();
            }
        }
    }

    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(name = "symbol")]
    #[derive(Debug, PyPayload)]
    struct py_lm_symbol_t(Opaque);

    #[pyclass]
    impl py_lm_symbol_t {
        #[pygetset]
        fn name(&self) -> String {
            let data: &lm_symbol_t = unsafe { self.0.as_ref() };
            data.get_name().to_owned()
        }

        #[pygetset]
        fn address(&self) -> lm_address_t {
            let data: &lm_symbol_t = unsafe { self.0.as_ref() };
            data.get_address()
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            let data: &lm_symbol_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            let data: &lm_symbol_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }
    }

    impl Drop for py_lm_symbol_t {
        fn drop(&mut self) {
            unsafe {
                self.0.drop::<lm_symbol_t>();
            }
        }
    }

    #[allow(non_camel_case_types)]
    #[pyattr]
    #[pyclass(name = "thread")]
    #[derive(Debug, PyPayload)]
    struct py_lm_thread_t(Opaque);

    #[pyclass]
    impl py_lm_thread_t {
        #[pygetset]
        fn tid(&self) -> lm_tid_t {
            let data: &lm_thread_t = unsafe { self.0.as_ref() };
            data.get_tid()
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            let data: &lm_thread_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            let data: &lm_thread_t = unsafe { self.0.as_ref() };
            format!("{data}")
        }
    }

    impl Drop for py_lm_thread_t {
        fn drop(&mut self) {
            unsafe {
                self.0.drop::<lm_thread_t>();
            }
        }
    }

    #[allow(non_camel_case_types)]
    struct py_lm_prot_t(lm_prot_t);

    impl py_lm_prot_t {
        fn to_pyobject(prot: lm_prot_t, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let mem = vm.import("mem", 0)?;
            let prot_cls = mem.get_attr("Prot", vm)?;
            let mut args = FuncArgs::default();
            args.prepend_arg(vm.ctx.new_int(prot as u8).into());

            let prot = prot_cls.call_with_args(args, vm)?;

            Ok(prot)
        }
    }

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

                _ => return Err(vm.new_value_error(format!("{value} is not a valid `prot`"))),
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
        unsafe fn as_mut<'a, T>(&self) -> &'a mut T {
            let ptr: *mut T = self.0.as_ptr().cast();
            unsafe { &mut *ptr }
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
