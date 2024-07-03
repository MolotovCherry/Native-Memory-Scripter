//! Rust based memory hacking library
//!
//! # Note about Provenance
//! You cannot use Rust-based fn pointers with this, because you have to obey provenance.
//! They are defined with an alloc of 0. You must only use pointers to external memory, or pointers with
//! defined provenance (making sure you never write/read outside of the alloc)

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
#![warn(missing_copy_implementations, missing_debug_implementations)]

#[cfg(not(any(target_arch = "x86_64", target_os = "windows")))]
compile_error!("only x86_64 windows is supported");

pub mod asm;
pub mod hook;
pub mod iat;
pub mod memory;
pub mod modules;
pub mod scan;
pub mod segments;
pub mod symbols;
mod utils;
pub mod vtable;

use windows::Win32::System::Memory::{
    PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_NOACCESS,
    PAGE_PROTECTION_FLAGS, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY,
};

/// The protection status of some memory
#[derive(Debug, Copy, Clone, strum::Display)]
pub enum Prot {
    /// none
    None,
    /// read
    R,
    /// write
    W,
    /// execute
    X,
    /// execute + read
    XR,
    /// execute + write
    XW,
    /// read + write
    RW,
    /// execute + read + right
    XRW,
    /// Other not listed
    Other,
}

impl From<Prot> for PAGE_PROTECTION_FLAGS {
    fn from(value: Prot) -> Self {
        match value {
            Prot::None => PAGE_NOACCESS,
            Prot::R => PAGE_READONLY,
            Prot::W => PAGE_WRITECOPY,
            Prot::X => PAGE_EXECUTE,
            Prot::XR => PAGE_EXECUTE_READ,
            Prot::XW => PAGE_EXECUTE_WRITECOPY,
            Prot::RW => PAGE_READWRITE,
            Prot::XRW => PAGE_EXECUTE_READWRITE,
            _ => unimplemented!("this flag is not valid"),
        }
    }
}

impl From<PAGE_PROTECTION_FLAGS> for Prot {
    fn from(value: PAGE_PROTECTION_FLAGS) -> Self {
        match value {
            PAGE_READONLY => Self::R,
            PAGE_WRITECOPY => Self::W,
            PAGE_EXECUTE => Self::X,
            PAGE_EXECUTE_READ => Self::XR,
            PAGE_EXECUTE_WRITECOPY => Self::XW,
            PAGE_READWRITE => Self::RW,
            PAGE_EXECUTE_READWRITE => Self::XRW,
            PAGE_NOACCESS => Self::None,
            _ => Self::Other,
        }
    }
}
