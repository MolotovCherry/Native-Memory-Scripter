use std::{ffi::CStr, fmt};

use pelite::{pe::Pe, pe64::PeView};

use crate::{module::Module, Address};

#[derive(Debug, thiserror::Error)]
pub enum SymbolError {
    #[error("symbol not found")]
    SymbolNotFound,
    #[error(transparent)]
    Pelite(#[from] pelite::Error),
}

#[derive(Debug, Clone)]
pub struct Symbol {
    name: String,
    address: Address,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Symbol {{ name: {}, address: {:#x} }}",
            self.name, self.address
        )
    }
}

fn enum_symbols_cb(
    module: &Module,
    mut cb: impl FnMut(Address, &str) -> bool,
) -> Result<(), SymbolError> {
    // this base address is crate private, so it is guaranteed
    let base = module.handle.base;

    // SAFETY: module field is crate private, it cannot be changed
    //         and we only support 64-bit. Additionally, each module is backed by
    //         an increased refcount, which keeps them valid for the duration of Module
    let view = unsafe { PeView::module(base as _) };

    let exports = view.exports()?;

    for (&func, &name) in exports.functions()?.iter().zip(exports.names()?.iter()) {
        // storing as address, but external mem, provenance OK
        let addr = base + func as usize;
        // external mem, provenance OK
        let name = sptr::from_exposed_addr(base + name as usize);

        let name = unsafe { CStr::from_ptr(name as _) };
        let name = name.to_string_lossy();

        if cb(addr, &name) {
            break;
        }
    }

    Ok(())
}

pub fn enum_symbols(module: &Module) -> Result<Vec<Symbol>, SymbolError> {
    let mut symbols = Vec::new();

    enum_symbols_cb(module, |addr, name| {
        let sym = Symbol {
            name: name.to_string(),
            address: addr,
        };

        symbols.push(sym);

        false
    })?;

    Ok(symbols)
}

pub fn enum_symbols_demangled(module: &Module) -> Result<Vec<Symbol>, SymbolError> {
    let mut symbols = Vec::new();

    enum_symbols_cb(module, |addr, name| {
        let name = demangle_symbol(name);

        let sym = Symbol {
            name,
            address: addr,
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
                address: addr,
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
                address: addr,
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
/// C++ (GCC-style compilers and MSVC) (features = ["cpp", "msvc"])
/// Rust (both legacy and v0) (features = ["rust"])
/// Swift (up to Swift 5.3) (features = ["swift"])
/// ObjC (only symbol detection)
pub fn demangle_symbol(symbol: &str) -> String {
    use symbolic_common::{Language, Name, NameMangling};
    use symbolic_demangle::{Demangle, DemangleOptions};

    let name = Name::new(symbol, NameMangling::Mangled, Language::Unknown);

    name.demangle(DemangleOptions::name_only())
        .unwrap_or_else(|| symbol.to_owned())
}
