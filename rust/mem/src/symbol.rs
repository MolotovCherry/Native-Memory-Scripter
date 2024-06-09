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

/// Find the address of an exported symbol in the module
/// Note that the name IS case-sensitive!
pub fn find_symbol_address(module: &Module, symbol: &str) -> Result<Symbol, SymbolError> {
    // this base address is crate private, so it is guaranteed
    let base = module.handle.base;

    // SAFETY: module field is crate private, it cannot be changed
    //         and we only support 64-bit. Additionally, each module is backed by
    //         an increased refcount, which keeps them valid for the duration of Module
    let view = unsafe { PeView::module(base as _) };

    let exports = view.exports()?;

    for (&func, &name) in exports.functions()?.iter().zip(exports.names()?.iter()) {
        let fn_addr = base + func as usize;
        let name = base + name as usize;

        let name = unsafe { CStr::from_ptr(name as _) };
        let name = name.to_string_lossy();
        if symbol == name {
            let symbol = Symbol {
                name: name.to_string(),
                address: fn_addr,
            };

            return Ok(symbol);
        }
    }

    Err(SymbolError::SymbolNotFound)
}
