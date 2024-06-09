//! This module allows one to read and write underlying system memory

use std::{mem, ptr};

use sptr::Strict;
use windows::Win32::System::{
    Memory::{
        VirtualAlloc, VirtualFree, VirtualProtect, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
        PAGE_PROTECTION_FLAGS,
    },
    SystemInformation::{GetSystemInfo, SYSTEM_INFO},
};

use crate::{Address, AddressUtils, Prot};

/// An error for the [memory](crate::memory) module
#[derive(Debug, thiserror::Error)]
pub enum MemError {
    /// address is invalid
    #[error("bad address")]
    BadAddress,
    /// a windows error
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
}

/// A Windows allocation which will be freed when this type is dropped
#[derive(Debug)]
#[repr(transparent)]
pub struct Alloc(Address);

impl Alloc {
    /// Get the address of the allocation. This ptr is valid up to the size of the allocation
    pub fn addr(&self) -> Address {
        self.0
    }
}

impl Drop for Alloc {
    fn drop(&mut self) {
        // provenance valid cause it's external mem, and address obtained from VirtualAlloc
        let ptr = sptr::from_exposed_addr_mut(self.0);
        _ = unsafe { VirtualFree(ptr, 0, MEM_RELEASE) };
    }
}

/// Read a T from memory address
///
/// # Safety
/// - Ptr must be valid, aligned, point to valid init data, have provenance for T bytes
/// - Memory at location must be initialized
/// - Memory at location must contain a valid bitpattern for T
/// - Beware of any drop impl on T
pub unsafe fn read<T>(addr: Address) -> T {
    if cfg!(debug_assertions) {
        let align = mem::align_of::<T>();
        assert!(addr % align == 0, "addr is not aligned to T");
    }

    debug_assert!(!addr.is_null(), "ptr must not be null");

    // provenance valid cause caller asserts it was previously exposed
    let addr = sptr::from_exposed_addr(addr);
    unsafe { ptr::read(addr) }
}

/// Read bytes from address
///
/// # Safety
/// - Memory at location must be initialized
/// - Ptr must be non-null
/// - Address must be valid for reads up to addr+count bytes
pub unsafe fn read_bytes(src: Address, count: usize) -> Vec<u8> {
    debug_assert!(!src.is_null(), "src must not be null");

    let mut buffer = Vec::with_capacity(count);

    // provenance valid cause caller asserts it was previously exposed
    let ptr = sptr::from_exposed_addr(src);
    unsafe {
        ptr::copy_nonoverlapping(ptr, buffer.as_mut_ptr(), count);
    }

    unsafe {
        buffer.set_len(count);
    }

    buffer
}

/// Write T to dst
///
/// # Safety
/// - dst must be aligned, contain provenance for T bytes
/// - dst must be valid for writes up to T bytes
pub unsafe fn write<T>(dst: Address, src: T) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    if cfg!(debug_assertions) {
        let align = mem::align_of::<T>();

        assert!(dst % align == 0, "dst is not aligned to T");
    }

    // provenance valid cause caller asserts it was previously exposed
    let dst = sptr::from_exposed_addr_mut(dst);

    unsafe {
        ptr::write(dst, src);
    }
}

/// Write bytes to dst
///
/// # Safety
/// - dst must be aligned, contain provenance for src.len() bytes
/// - addresses must not overlap
pub unsafe fn write_bytes(src: &[u8], dst: Address) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    unsafe { write_raw(src.as_ptr().expose_addr(), dst, src.len()) }
}

/// Write ptr to dst
///
/// # Safety
/// - src must be valid for reads up to count bytes
/// - dst must be valid for writes up to count bytes
/// - addresses must not overlap
pub unsafe fn write_raw(src: Address, dst: Address, count: usize) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    // provenances are valid cause caller asserts they were previously exposed
    let src = sptr::from_exposed_addr::<u8>(src);
    let dst = sptr::from_exposed_addr_mut(dst);
    unsafe {
        ptr::copy_nonoverlapping(src, dst, count);
    }
}

/// Set dst to val for N bytes
///
/// # Safety
/// - dst must be valid for writes up to count bytes
pub unsafe fn set(dst: Address, val: u8, count: usize) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    // provenance valid cause caller asserts it was previously exposed
    let dst = sptr::from_exposed_addr_mut::<u8>(dst);
    unsafe {
        ptr::write_bytes(dst, val, count);
    }
}

/// Change memory protection of a certain region of memory
///
/// # Safety
/// - address must contain exposed provenance, and for `size` bytes
/// - any safety requirements of VirtualProtect
pub unsafe fn prot(addr: Address, mut size: usize, prot: Prot) -> Result<Prot, MemError> {
    if addr.is_null() {
        return Err(MemError::BadAddress);
    }

    if size == 0 {
        size = get_page_size() as usize;
    }

    let mut old_prot = PAGE_PROTECTION_FLAGS::default();

    // provenance valid cause caller asserts it was previously exposed
    let addr = sptr::from_exposed_addr(addr);
    unsafe {
        VirtualProtect(addr, size, prot.into(), &mut old_prot)?;
    }

    Ok(old_prot.into())
}

/// Allocate memory of size `size` with protection `prot`
pub fn alloc(mut size: usize, prot: Prot) -> Result<Alloc, MemError> {
    if size == 0 {
        size = get_page_size() as usize;
    }

    let alloc = unsafe { VirtualAlloc(None, size, MEM_COMMIT | MEM_RESERVE, prot.into()) };
    if alloc.is_null() {
        return Err(MemError::BadAddress);
    }

    // provenance valid cause it was given from external mem
    let alloc = alloc.expose_addr();
    let alloc = Alloc(alloc);

    Ok(alloc)
}

// TODO: DeepPointer

fn get_page_size() -> u32 {
    let mut sysinfo = SYSTEM_INFO::default();
    unsafe {
        GetSystemInfo(&mut sysinfo);
    }

    sysinfo.dwPageSize
}
