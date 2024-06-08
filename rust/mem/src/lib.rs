#![allow(clippy::missing_safety_doc)]
#![warn(unsafe_op_in_unsafe_fn)]

pub mod asm;
pub mod hook;
pub mod memory;
pub mod module;
mod utils;

use windows::Win32::System::Memory::{
    PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY,
    PAGE_PROTECTION_FLAGS, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY,
};

pub type Address = usize;

trait AddressUtils {
    fn is_null(&self) -> bool;
    fn as_ptr<T>(&self) -> *const T;
    fn as_mut<T>(&self) -> *mut T;
}

impl AddressUtils for Address {
    fn is_null(&self) -> bool {
        *self == 0
    }

    fn as_ptr<T>(&self) -> *const T {
        *self as _
    }

    fn as_mut<T>(&self) -> *mut T {
        *self as _
    }
}

#[derive(Debug, strum::Display)]
pub enum Prot {
    None,
    R,
    W,
    X,
    XR,
    XW,
    RW,
    XRW,
}

impl From<Prot> for PAGE_PROTECTION_FLAGS {
    fn from(value: Prot) -> Self {
        match value {
            Prot::None => todo!(),
            Prot::R => PAGE_READONLY,
            Prot::W => PAGE_WRITECOPY,
            Prot::X => PAGE_EXECUTE,
            Prot::XR => PAGE_EXECUTE_READ,
            Prot::XW => PAGE_EXECUTE_WRITECOPY,
            Prot::RW => PAGE_READWRITE,
            Prot::XRW => PAGE_EXECUTE_READWRITE,
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
            _ => Prot::None,
        }
    }
}
