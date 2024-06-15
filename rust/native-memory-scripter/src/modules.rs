#![allow(clippy::module_inception)]

pub mod asm;
pub mod cffi;
pub mod hook;
pub mod iat;
pub mod info;
pub mod log;
pub mod mem;
pub mod modules;
pub mod scan;
pub mod segments;
pub mod symbols;
pub mod vmt;

pub type Address = usize;
