//! This module allows one to read and write underlying system memory

use std::{ffi::c_void, mem, ptr};

use windows::Win32::{
    Foundation::{GetLastError, WIN32_ERROR},
    System::{
        Memory::{
            VirtualAlloc, VirtualFree, VirtualProtect, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
            PAGE_PROTECTION_FLAGS,
        },
        SystemInformation::{GetSystemInfo, SYSTEM_INFO},
    },
};

use crate::Prot;

/// An error for the [memory](crate::memory) module
#[derive(Debug, Clone, thiserror::Error)]
pub enum MemError {
    /// address is invalid
    #[error("bad address")]
    BadAddress,
    /// a windows error
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
    /// a windows error
    #[error("{0:?}: {}", .0.to_hresult().message())]
    Win32(windows::Win32::Foundation::WIN32_ERROR),
}

impl From<WIN32_ERROR> for MemError {
    fn from(value: WIN32_ERROR) -> Self {
        Self::Win32(value)
    }
}

/// A Windows allocation which will be freed when this type is dropped
#[derive(Debug)]
#[repr(transparent)]
pub struct Alloc(*mut c_void);

unsafe impl Send for Alloc {}
unsafe impl Sync for Alloc {}

impl Alloc {
    /// Get the address of the allocation. This ptr is valid up to the size of the allocation
    pub fn addr(&self) -> *mut u8 {
        self.0.cast()
    }
}

impl Drop for Alloc {
    fn drop(&mut self) {
        _ = unsafe { VirtualFree(self.0, 0, MEM_RELEASE) };
    }
}

/// Read a T from memory address
///
/// # Safety
/// - Addr must be valid for tp to T bytes
/// - Memory at location must be initialized
/// - Memory at location must contain a valid bitpattern for T
/// - Beware of any drop impl on T
pub unsafe fn read<T>(addr: *const T) -> T {
    if cfg!(debug_assertions) {
        let align = mem::align_of::<T>();
        assert!(addr as usize % align == 0, "addr is not aligned to T");
    }

    debug_assert!(!addr.is_null(), "ptr must not be null");

    unsafe { ptr::read_volatile(addr) }
}

/// Read bytes from address
///
/// # Safety
/// - Memory at location must be initialized
/// - Ptr must be non-null
/// - Address must be valid for reads up to addr+count bytes
pub unsafe fn read_bytes(src: *const u8, count: usize) -> Vec<u8> {
    debug_assert!(!src.is_null(), "src must not be null");

    let mut buffer = Vec::with_capacity(count);

    unsafe {
        ptr::copy_nonoverlapping(src, buffer.as_mut_ptr(), count);
    }

    unsafe {
        buffer.set_len(count);
    }

    buffer
}

/// Write T to dst
///
/// # Safety
/// - dst must be valid for up to T bytes
/// - dst must be valid for writes up to T bytes
pub unsafe fn write<T>(dst: *mut T, src: T) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    if cfg!(debug_assertions) {
        let align = mem::align_of::<T>();

        assert!(dst as usize % align == 0, "dst is not aligned to T");
    }

    unsafe {
        ptr::write_volatile(dst, src);
    }
}

/// Write bytes to dst
///
/// # Safety
/// - dst must be valid for up to src.len() bytes
/// - addresses must not overlap
pub unsafe fn write_bytes(src: &[u8], dst: *mut u8) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    unsafe { write_raw(src.as_ptr(), dst, src.len()) }
}

/// Write ptr to dst
///
/// # Safety
/// - src must be valid for reads up to count bytes
/// - dst must be valid for writes up to count bytes
/// - addresses must not overlap
pub unsafe fn write_raw(src: *const u8, dst: *mut u8, count: usize) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    unsafe {
        ptr::copy_nonoverlapping(src, dst, count);
    }
}

/// Set dst to val for N bytes
///
/// # Safety
/// - dst must be valid for writes up to count bytes
pub unsafe fn set(dst: *mut u8, val: u8, count: usize) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    unsafe {
        ptr::write_bytes(dst, val, count);
    }
}

/// Change memory protection of a certain region of memory
///
/// # Safety
/// - address must be valid fpr up to `size` bytes
/// - any safety requirements of VirtualProtect
pub unsafe fn prot(addr: *const (), mut size: usize, prot: Prot) -> Result<Prot, MemError> {
    if addr.is_null() {
        return Err(MemError::BadAddress);
    }

    if size == 0 {
        size = get_page_size() as usize;
    }

    let mut old_prot = PAGE_PROTECTION_FLAGS::default();

    unsafe {
        VirtualProtect(addr.cast(), size, prot.into(), &mut old_prot)?;
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
        let error = unsafe { GetLastError() };
        return Err(error.into());
    }

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
