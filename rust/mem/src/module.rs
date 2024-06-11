//! This module allows one to get a process's loaded modules

use std::{
    fmt, iter, mem,
    os::windows::prelude::OsStrExt,
    path::{Path, PathBuf},
    string::FromUtf16Error,
};

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

use crate::utils::LazyLock;

/// An error for the [Module] type
#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    /// wrong path or path does not exist
    #[error("filename does not exist")]
    BadPath,
    /// cannot convert osstr to utf8
    #[error("failed to convert to utf8")]
    OsStrConversion,
    /// cannot convert utf16 to utf8
    #[error(transparent)]
    Utf16Conversion(#[from] FromUtf16Error),
    #[error(transparent)]
    /// a windows erorr
    Windows(#[from] windows::core::Error),
    /// no modules were found
    #[error("no modules available")]
    NoModules(windows::core::Error),
    /// module was not found
    #[error("module not found")]
    NotFound,
}

type Pid = u32;

static PROCESS: LazyLock<(HANDLE, Pid)> =
    LazyLock::new(|| unsafe { (GetCurrentProcess(), GetCurrentProcessId()) });

/// A handle based type which keeps the library loaded, which ensures the
/// base address is always correct as long as the handle exists
#[derive(Debug)]
pub(crate) struct ModuleHandle {
    path: Vec<u16>,
    pub(crate) base: *mut u8, // equivalent to HMODULE
}

unsafe impl Send for ModuleHandle {}

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
        // decrease library refcount when done
        _ = unsafe { FreeLibrary(HMODULE(self.base as _)) };
    }
}

/// Represents a module. The dll refcount is increased 1 for this, so it will not
/// be unloaded until all modules go out of scope
#[derive(Clone)]
pub struct Module {
    /// our own unalterable copy of the base
    pub(crate) handle: ModuleHandle,

    /// base address of the module in memory
    pub base: *mut u8,
    /// end address of the module in memory
    pub end: *mut u8,
    /// the size of the module in memory
    pub size: u32,
    /// the filesystem path to the module
    pub path: PathBuf,
    /// the filename of the module
    pub name: String,
}

unsafe impl Send for Module {}

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
            base: module_info.lpBaseOfDll.cast(),
            end: unsafe {
                module_info
                    .lpBaseOfDll
                    .add(module_info.SizeOfImage as usize)
                    .cast()
            },
            size: module_info.SizeOfImage,
            path,
            name,
        };

        Ok(module)
    }
}

impl Module {
    /// load a module into the process
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

    /// Unload a module from the process.
    ///
    /// Note: Windows keeps a refcount of each module. The module is only unloaded when
    ///       this refcount reaches 0. All existing Module's to a specific dll keep a refcount
    ///       open in order to ensure safe operation of apis. Dropping Module will decrease the refcount by 1.
    ///       To ensure a dll you loaded is unloaded, you must Drop or unload all existing Module references to it.
    pub fn unload(self) {
        // this is a no-op. Drop impl releases refcount
    }

    /// Decrease library refcount by 1 and unload it if it reaches 0.
    /// Each call to this will decrease the refcount by 1.
    ///
    /// # Safety
    /// Ensure no other code anywhere in the process is using this module anymore. Otherwise may be UB,
    /// especially if a Module still exists to this library
    pub unsafe fn unload_path<P: AsRef<Path>>(path: P) -> Result<(), ModuleError> {
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
}

fn enum_modules_cb(mut cb: impl FnMut(Module) -> bool) -> Result<(), ModuleError> {
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

    loop {
        let len = entry.szModule.iter().position(|n| *n == 0).unwrap_or(255);
        let name = String::from_utf16(&entry.szModule[..len])?;

        let len = entry.szExePath.iter().position(|n| *n == 0).unwrap_or(259);
        let path = String::from_utf16(&entry.szExePath[..len])?;

        let handle = ModuleHandle::new(&path)?;

        let module = Module {
            handle,
            base: entry.modBaseAddr,
            end: unsafe { entry.modBaseAddr.add(entry.dwSize as usize) },
            size: entry.modBaseSize,
            path: PathBuf::from(path),
            name,
        };

        if cb(module) {
            break;
        }

        if let Err(err) = unsafe { Module32NextW(hsnap, &mut entry) } {
            if err.code() == ERROR_NO_MORE_FILES.to_hresult() {
                break;
            } else {
                Err(err)?;
            }
        }
    }

    Ok(())
}

/// Get a list of all modules loaded into the process
pub fn enum_modules() -> Result<Vec<Module>, ModuleError> {
    let mut modules = Vec::new();

    enum_modules_cb(|module| {
        modules.push(module);
        false
    })?;

    Ok(modules)
}

/// Find a module by name. This is case sensitive
pub fn find_module(name: &str) -> Result<Module, ModuleError> {
    let mut module_ret = None;

    enum_modules_cb(|module| {
        if module.name == name {
            module_ret = Some(module);
            true
        } else {
            false
        }
    })?;

    module_ret.ok_or(ModuleError::NotFound)
}
