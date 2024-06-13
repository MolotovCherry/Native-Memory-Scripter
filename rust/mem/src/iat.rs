//! This module allows one to search through, demangle, and hook a module's import address table functions

use std::{
    ffi::{CStr, FromBytesWithNulError},
    fmt, mem, ptr,
    sync::{Arc, Mutex},
};

use pelite::{
    pe::{Pe, PeView},
    pe64::imports::Import,
};
use windows::Win32::System::WindowsProgramming::IMAGE_THUNK_DATA64;

use crate::{
    memory::{self, MemError},
    modules::Module,
    symbols, Prot,
};

/// An error for the [Symbol] type
#[derive(Clone, Debug, thiserror::Error)]
pub enum IATSymbolError {
    /// an error from pelite
    #[error(transparent)]
    Pelite(#[from] pelite::Error),
    /// Cstr had a conversion error
    #[error(transparent)]
    CStr(#[from] FromBytesWithNulError),
    /// an error occurred during mem access
    #[error(transparent)]
    Mem(#[from] MemError),
}

/// Identifier for import symbol
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolIdent {
    /// The symbol's name
    Name(String),
    /// The symbol's ordinal
    Ordinal(u16),
}

/// A symbol in a [Module](crate::module::Module)
#[derive(Clone)]
pub struct IATSymbol {
    /// the symbol name or ordinal
    pub ident: SymbolIdent,
    /// the dll the symbol belongs to
    pub dll: String,
    /// the address to the original function stored at the iat entry
    pub orig_fn: *const (),
    /// the address in the iat table where the actual pointer to the function is stored
    /// note: you cannot write to this without first making it writable
    pub iat_entry: *const u64,
    /// to prevent data races
    lock: Arc<Mutex<()>>,
    // these are used as backup addresses since the others are public and can be modified
    orig_fn_backup: *const (),
    iat_backup: *mut u64,
}

impl fmt::Debug for IATSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let IATSymbol {
            ident,
            dll,
            orig_fn,
            iat_entry,
            ..
        } = self;

        f.debug_struct("IATSymbol")
            .field("ident", &ident)
            .field("dll", &dll)
            .field("orig_fn", &orig_fn)
            .field("iat_entry", &iat_entry)
            .finish()
    }
}

impl IATSymbol {
    /// Get the function address the iat symbol is pointing to
    pub fn fn_addr(&self) -> *const () {
        let _guard = self.lock.lock().unwrap();
        unsafe { ptr::read(self.iat_backup) as _ }
    }

    /// Set the function address the iat symbol is pointing to
    ///
    /// # Safety
    /// This can cause unforseen side effects. All fn calls are now belong to us.
    /// You are on your own.
    pub unsafe fn hook(&self, address: *const ()) -> Result<(), IATSymbolError> {
        let _guard = self.lock.lock().unwrap();

        // first we need to make this region writable
        let old =
            unsafe { memory::prot(self.iat_backup.cast(), mem::size_of::<u64>(), Prot::XRW)? };

        unsafe {
            memory::write(self.iat_backup, address as _);
        }

        // set it back to original now
        unsafe {
            memory::prot(self.iat_backup.cast(), mem::size_of::<u64>(), old)?;
        }

        Ok(())
    }

    /// Undoes any hooking to the iat entry
    ///
    /// # Safety
    /// This can cause unforseen side effects. All fn calls are now belong to us.
    /// You are on your own.
    pub unsafe fn unhook(&self) -> Result<(), IATSymbolError> {
        let _guard = self.lock.lock().unwrap();

        // first we need to make this region writable
        let old =
            unsafe { memory::prot(self.iat_backup.cast(), mem::size_of::<u64>(), Prot::XRW)? };

        unsafe {
            memory::write(self.iat_backup, self.orig_fn_backup as _);
        }

        // set it back to original now
        unsafe {
            memory::prot(self.iat_backup.cast(), mem::size_of::<u64>(), old)?;
        }

        Ok(())
    }
}

unsafe impl Send for IATSymbol {}
unsafe impl Sync for IATSymbol {}

fn enum_iat_symbols_cb(
    module: &Module,
    mut cb: impl FnMut(&str, (*mut u64, *const ()), SymbolIdent) -> bool,
) -> Result<(), IATSymbolError> {
    // this base address is crate private, so it is guaranteed
    let base = module.handle.base;

    // SAFETY: module field is crate private, it cannot be changed
    //         and we only support 64-bit. Additionally, each module is backed by
    //         an increased refcount, which keeps them valid for the duration of Module
    let view = unsafe { PeView::module(base.cast()) };

    let imports = view.imports()?;

    for desc in imports {
        // DLL being imported from
        let dll_name = desc.dll_name()?.c_str();
        let dll_name = CStr::from_bytes_with_nul(dll_name)?;
        let dll_name = dll_name.to_string_lossy();

        // Get all RVAs
        let image = desc.image();
        let mut thunk = view.rva_to_va(image.FirstThunk)? as *mut IMAGE_THUNK_DATA64;

        // Import Name Table for this imported DLL
        let int = desc.int()?;

        for import in int {
            let import = import?;

            let original_fn = unsafe { (*thunk).u1.Function };
            let thunk_data = (thunk as *mut u64, original_fn as *const ());

            let ident = match import {
                Import::ByName { name, .. } => {
                    let name = CStr::from_bytes_with_nul(name.c_str())?;
                    let name = name.to_string_lossy();
                    SymbolIdent::Name(name.to_string())
                }

                Import::ByOrdinal { ord } => SymbolIdent::Ordinal(ord),
            };

            if cb(&dll_name, thunk_data, ident) {
                break;
            }

            thunk = unsafe { thunk.add(1) };
        }
    }

    Ok(())
}

/// Return all import symbols in their raw form
pub fn enum_iat_symbols(module: &Module) -> Result<Vec<IATSymbol>, IATSymbolError> {
    let mut imports = Vec::new();

    enum_iat_symbols_cb(module, |dll, (iat_entry, orig_fn), ident| {
        let sym = IATSymbol {
            ident,
            dll: dll.to_string(),
            orig_fn,
            iat_entry,
            lock: Arc::default(),
            orig_fn_backup: orig_fn,
            iat_backup: iat_entry,
        };

        imports.push(sym);

        false
    })?;

    Ok(imports)
}

/// Return all demangled import symbols
pub fn enum_iat_symbols_demangled(module: &Module) -> Result<Vec<IATSymbol>, IATSymbolError> {
    let mut imports = Vec::new();

    enum_iat_symbols_cb(module, |dll, (iat_entry, original_fn), ident| {
        let ident = match ident {
            SymbolIdent::Name(n) => {
                let demangled = symbols::demangle_symbol(&n).unwrap_or(n);
                SymbolIdent::Name(demangled)
            }

            v => v,
        };

        let sym = IATSymbol {
            ident,
            dll: dll.to_string(),
            orig_fn: original_fn,
            iat_entry,
            lock: Arc::default(),
            orig_fn_backup: original_fn,
            iat_backup: iat_entry,
        };

        imports.push(sym);

        false
    })?;

    Ok(imports)
}

/// Find the first specific iat symbol ident or ordinal
pub fn find_iat_symbol(
    module: &Module,
    ident: &SymbolIdent,
) -> Result<Option<IATSymbol>, IATSymbolError> {
    let mut out_sym = None;

    enum_iat_symbols_cb(module, |dll, (iat, orig_fn), import_ident| {
        if ident == &import_ident {
            let sym = IATSymbol {
                ident: import_ident,
                dll: dll.to_string(),
                orig_fn,
                iat_entry: iat,
                lock: Arc::default(),
                orig_fn_backup: orig_fn,
                iat_backup: iat,
            };

            out_sym = Some(sym);

            return true;
        }

        false
    })?;

    Ok(out_sym)
}

/// Find the first specific iat symbol ident or ordinal in a specific dll
///
/// Note that dll name is an exact case sensitive match (with ".dll" extension)
pub fn find_dll_iat_symbol(
    module: &Module,
    dll: &str,
    ident: &SymbolIdent,
) -> Result<Option<IATSymbol>, IATSymbolError> {
    let mut out_sym = None;

    enum_iat_symbols_cb(module, |dll_name, (iat_entry, orig_fn), import_ident| {
        if dll == dll_name && ident == &import_ident {
            let sym = IATSymbol {
                ident: import_ident,
                dll: dll_name.to_string(),
                orig_fn,
                iat_entry,
                lock: Arc::default(),
                orig_fn_backup: orig_fn,
                iat_backup: iat_entry,
            };

            out_sym = Some(sym);

            return true;
        }

        false
    })?;

    Ok(out_sym)
}

/// Find a specific iat symbol demangled ident name. If you want to find an ordinal, use [find_iat_symbol] instead
/// The search is a fuzzy contains search, but is still case sensitive
pub fn find_iat_symbol_demangled(
    module: &Module,
    name: &str,
) -> Result<Option<IATSymbol>, IATSymbolError> {
    let mut out_sym = None;

    enum_iat_symbols_cb(module, |dll, (iat_entry, orig_fn), import_ident| {
        let is_match = match import_ident {
            SymbolIdent::Name(ref n) => {
                let demangled = symbols::demangle_symbol(n);
                let demangled = demangled.as_deref().unwrap_or(n);

                demangled.contains(name)
            }

            SymbolIdent::Ordinal(_) => false,
        };

        if is_match {
            let sym = IATSymbol {
                ident: import_ident,
                dll: dll.to_string(),
                orig_fn,
                iat_entry,
                lock: Arc::default(),
                orig_fn_backup: orig_fn,
                iat_backup: iat_entry,
            };

            out_sym = Some(sym);

            return true;
        }

        false
    })?;

    Ok(out_sym)
}

/// Find a specific iat symbol demangled ident name. If you want to find an ordinal, use [find_iat_symbol] instead
/// The search is a fuzzy contains search, but is still case sensitive
///
/// Note that dll name is an exact case sensitive match (with ".dll" extension)
pub fn find_dll_iat_symbol_demangled(
    module: &Module,
    dll: &str,
    name: &str,
) -> Result<Option<IATSymbol>, IATSymbolError> {
    let mut out_sym = None;

    enum_iat_symbols_cb(module, |dll_name, (iat_entry, orig_fn), import_ident| {
        let is_match = match import_ident {
            SymbolIdent::Name(ref n) => {
                let demangled = symbols::demangle_symbol(n);
                let demangled = demangled.as_deref().unwrap_or(n);

                demangled.contains(name)
            }

            SymbolIdent::Ordinal(_) => false,
        };

        if dll == dll_name && is_match {
            let sym = IATSymbol {
                ident: import_ident,
                dll: dll_name.to_string(),
                orig_fn,
                iat_entry,
                lock: Arc::default(),
                orig_fn_backup: orig_fn,
                iat_backup: iat_entry,
            };

            out_sym = Some(sym);

            return true;
        }

        false
    })?;

    Ok(out_sym)
}
