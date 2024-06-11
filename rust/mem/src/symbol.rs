//! This module allows one to search through and demangle a module's external symbols

use std::ffi::CStr;

use pelite::{pe::Pe, pe64::PeView};

use crate::module::Module;

/// An error for the [Symbol] type
#[derive(Debug, thiserror::Error)]
pub enum SymbolError {
    /// symbol was not found
    #[error("symbol not found")]
    SymbolNotFound,
    /// an error from pelite
    #[error(transparent)]
    Pelite(#[from] pelite::Error),
}

/// A symbol in a [Module](crate::module::Module)
#[derive(Debug, Clone)]
pub struct Symbol {
    /// the symbol name
    pub name: String,
    /// the symbol's starting address
    pub address: *const (),
}

unsafe impl Send for Symbol {}

fn enum_symbols_cb(
    module: &Module,
    mut cb: impl FnMut(*mut u8, &str) -> bool,
) -> Result<(), SymbolError> {
    // this base address is crate private, so it is guaranteed
    let base = module.handle.base;

    // SAFETY: module field is crate private, it cannot be changed
    //         and we only support 64-bit. Additionally, each module is backed by
    //         an increased refcount, which keeps them valid for the duration of Module
    let view = unsafe { PeView::module(base as _) };

    let exports = view.exports()?;

    for (&func, &name) in exports.functions()?.iter().zip(exports.names()?.iter()) {
        let addr = unsafe { base.add(func as usize) };

        let name = unsafe { CStr::from_ptr(name as _) };
        let name = name.to_string_lossy();

        if cb(addr, &name) {
            break;
        }
    }

    Ok(())
}

/// Return all symbols in their raw form
pub fn enum_symbols(module: &Module) -> Result<Vec<Symbol>, SymbolError> {
    let mut symbols = Vec::new();

    enum_symbols_cb(module, |addr, name| {
        let sym = Symbol {
            name: name.to_string(),
            address: addr.cast(),
        };

        symbols.push(sym);

        false
    })?;

    Ok(symbols)
}

/// Return all symbols in their demangled form
pub fn enum_symbols_demangled(module: &Module) -> Result<Vec<Symbol>, SymbolError> {
    let mut symbols = Vec::new();

    enum_symbols_cb(module, |addr, name| {
        let name = demangle_symbol(name);

        let sym = Symbol {
            name,
            address: addr.cast(),
        };

        symbols.push(sym);

        false
    })?;

    Ok(symbols)
}

/// Find the address of an exported symbol in the module
/// Note that the name IS case-sensitive and requires an exact match!
pub fn find_symbol_address(module: &Module, symbol: &str) -> Result<Symbol, SymbolError> {
    let mut symbol_out = None;

    enum_symbols_cb(module, |addr, name| {
        if symbol == name {
            let sym = Symbol {
                name: name.to_string(),
                address: addr.cast(),
            };

            symbol_out = Some(sym);

            true
        } else {
            false
        }
    })?;

    if let Some(symbol) = symbol_out {
        Ok(symbol)
    } else {
        Err(SymbolError::SymbolNotFound)
    }
}

/// Find the address of an exported symbol in the module
/// Note that the name IS case-sensitive but only requires a partial match!
pub fn find_symbol_address_demangled(module: &Module, symbol: &str) -> Result<Symbol, SymbolError> {
    let mut symbol_out = None;

    enum_symbols_cb(module, |addr, name| {
        let name = demangle_symbol(name);

        if name.contains(symbol) {
            let sym = Symbol {
                name,
                address: addr.cast(),
            };

            symbol_out = Some(sym);

            true
        } else {
            false
        }
    })?;

    if let Some(symbol) = symbol_out {
        Ok(symbol)
    } else {
        Err(SymbolError::SymbolNotFound)
    }
}

/// Demangle a symbol. If language can not be detected, returns original mangled symbol, otherwise
/// will return demangled symbol
///
/// Supports:
/// C++ (GCC-style compilers and MSVC)
/// Rust (both legacy and v0)
/// Swift (up to Swift 5.3)
/// ObjC (only symbol detection)
pub fn demangle_symbol(symbol: &str) -> String {
    use symbolic_common::{Language, Name, NameMangling};
    use symbolic_demangle::{Demangle, DemangleOptions};

    let name = Name::new(symbol, NameMangling::Mangled, Language::Unknown);

    name.demangle(DemangleOptions::name_only())
        .unwrap_or_else(|| symbol.to_owned())
}
