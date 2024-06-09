use std::{
    fmt, iter, mem,
    os::windows::prelude::OsStrExt,
    path::{Path, PathBuf},
    string::FromUtf16Error,
};

use sptr::Strict;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{FreeLibrary, ERROR_NO_MORE_FILES, HANDLE, HMODULE},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, MODULEENTRY32W,
                TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32,
            },
            LibraryLoader::{GetModuleFileNameW, GetModuleHandleW, LoadLibraryW},
            ProcessStatus::{GetModuleInformation, MODULEINFO},
            Threading::{GetCurrentProcess, GetCurrentProcessId},
        },
    },
};

use crate::{utils::LazyLock, Address};

#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    #[error("filename does not exist")]
    BadPath,
    #[error("failed to convert to utf8")]
    OsStrConversion,
    #[error(transparent)]
    Utf16Conversion(#[from] FromUtf16Error),
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
    #[error("no modules available")]
    NoModules(windows::core::Error),
}

type Pid = u32;

static PROCESS: LazyLock<(HANDLE, Pid)> =
    LazyLock::new(|| unsafe { (GetCurrentProcess(), GetCurrentProcessId()) });

/// A handle based type which keeps the library loaded, which ensures the
/// base address is always correct as long as the handle exists
#[derive(Debug)]
pub(crate) struct ModuleHandle {
    path: Vec<u16>,
    pub(crate) base: Address, // equivalent to HMODULE
}

impl ModuleHandle {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, ModuleError> {
        let path = path
            .as_ref()
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0))
            .collect::<Vec<_>>();

        // increase library refcount
        let module = unsafe { LoadLibraryW(PCWSTR(path.as_ptr()))? };

        let slf = Self {
            path,
            // external ptr, but provenance OK
            base: module.0 as _,
        };

        Ok(slf)
    }
}

impl Clone for ModuleHandle {
    fn clone(&self) -> Self {
        // increase refcount
        unsafe { LoadLibraryW(PCWSTR(self.path.as_ptr())).expect("load library failed") };

        Self {
            path: self.path.clone(),
            base: self.base,
        }
    }
}

impl Drop for ModuleHandle {
    fn drop(&mut self) {
        _ = unsafe { FreeLibrary(HMODULE(self.base as _)) };
    }
}

/// Represents a module. The dll refcount is increased 1 for this, so it will not
/// be unloaded until all modules go out of scope
#[derive(Clone)]
pub struct Module {
    // our own unalterable copy of the base
    pub(crate) handle: ModuleHandle,

    pub base: Address,
    pub end: Address,
    pub size: u32,
    pub path: PathBuf,
    pub name: String,
}

unsafe impl Send for Module {}
unsafe impl Sync for Module {}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Module")
            .field("base", &self.base)
            .field("end", &self.end)
            .field("size", &self.size)
            .field("path", &self.path)
            .field("name", &self.name)
            .finish()
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Module {{ base: {:#x?}, end: {:#x?}, size: {}, path: {}, name: {} }}",
            self.base,
            self.end,
            self.size,
            self.path.display(),
            self.name
        )
    }
}

impl TryFrom<HMODULE> for Module {
    type Error = ModuleError;

    fn try_from(module: HMODULE) -> Result<Self, Self::Error> {
        let mut module_info = MODULEINFO::default();

        unsafe {
            GetModuleInformation(
                PROCESS.0,
                module,
                &mut module_info,
                mem::size_of::<MODULEINFO>() as u32,
            )?;
        }

        let mut buffer = vec![0; 1024];
        let n = unsafe { GetModuleFileNameW(module, &mut buffer) };

        let path: PathBuf = String::from_utf16(&buffer[..n as usize])?.into();
        let name = path
            .file_name()
            .ok_or(ModuleError::BadPath)?
            .to_str()
            .ok_or(ModuleError::OsStrConversion)?
            .to_owned();

        let handle = ModuleHandle::new(&path)?;

        let module = Module {
            handle,
            // provenance will always be OK since external mem
            base: module_info.lpBaseOfDll.expose_addr(),
            // provenance will always be OK since external mem
            end: module_info.lpBaseOfDll.expose_addr() + module_info.SizeOfImage as usize,
            size: module_info.SizeOfImage,
            path,
            name,
        };

        Ok(module)
    }
}

impl Module {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ModuleError> {
        let path = path
            .as_ref()
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0))
            .collect::<Vec<_>>();

        let module = unsafe { LoadLibraryW(PCWSTR(path.as_ptr()))? };

        module.try_into()
    }

    pub fn unload(self) -> Result<(), ModuleError> {
        unsafe {
            FreeLibrary(HMODULE(self.handle.base as _))?;
        }

        Ok(())
    }

    pub fn unload_path<P: AsRef<Path>>(path: P) -> Result<(), ModuleError> {
        let path = path
            .as_ref()
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0))
            .collect::<Vec<_>>();

        let module = unsafe { GetModuleHandleW(PCWSTR(path.as_ptr()))? };

        unsafe {
            FreeLibrary(module)?;
        }

        Ok(())
    }

    pub fn handle(&self) -> HMODULE {
        HMODULE(self.handle.base as _)
    }
}

pub fn enum_modules() -> Result<Vec<Module>, ModuleError> {
    let process = *PROCESS;

    let hsnap =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, process.1)? };

    let mut entry = MODULEENTRY32W {
        dwSize: mem::size_of::<MODULEENTRY32W>() as u32,
        ..Default::default()
    };

    if let Err(err) = unsafe { Module32FirstW(hsnap, &mut entry) } {
        return Err(ModuleError::NoModules(err));
    };

    let mut modules = Vec::new();

    loop {
        let len = entry.szModule.iter().position(|n| *n == 0).unwrap_or(255);
        let name = String::from_utf16(&entry.szModule[..len])?;

        let len = entry.szExePath.iter().position(|n| *n == 0).unwrap_or(259);
        let path = String::from_utf16(&entry.szExePath[..len])?;

        let handle = ModuleHandle::new(&path)?;

        let module = Module {
            handle,
            // external mem, provenance OK
            base: entry.modBaseAddr.expose_addr(),
            // external mem, provenance OK
            end: entry.modBaseAddr.expose_addr() + entry.dwSize as usize,
            size: entry.modBaseSize,
            path: PathBuf::from(path),
            name,
        };

        modules.push(module);

        if let Err(err) = unsafe { Module32NextW(hsnap, &mut entry) } {
            if err.code() == ERROR_NO_MORE_FILES.to_hresult() {
                break;
            } else {
                Err(err)?;
            }
        }
    }

    Ok(modules)
}
