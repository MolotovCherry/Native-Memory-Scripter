use std::{
    mem,
    ptr::{self, NonNull},
};

use windows::Win32::System::{
    Memory::{
        VirtualAlloc, VirtualFree, VirtualProtect, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
        PAGE_PROTECTION_FLAGS,
    },
    SystemInformation::{GetSystemInfo, SYSTEM_INFO},
};

use crate::{Address, AddressUtils, Prot};

#[derive(Debug, thiserror::Error)]
pub enum MemError {
    #[error("bad address")]
    BadAddress,
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Alloc(NonNull<()>);

unsafe impl Send for Alloc {}

impl Alloc {
    pub fn addr(&self) -> usize {
        self.0.as_ptr() as _
    }
}

impl Drop for Alloc {
    fn drop(&mut self) {
        _ = unsafe { VirtualFree(self.0.as_ptr().cast(), 0, MEM_RELEASE) };
    }
}

/// Read a T from memory address
///
/// SAFETY:
///     - Memory at location must be initialized
///     - Memory at location must contain a valid bitpattern for T
///     - Ptr must be non-null
///     - Should only use for runtime addresses (otherwise lack of provenance is UB)
///     - Beware of any drop impl on T
///     - Address must be aligned for T
///     - Address must be valid for reads up to size of T
pub unsafe fn read<T>(addr: Address) -> T {
    if cfg!(debug_assertions) {
        let align = mem::align_of::<T>();
        assert!(addr % align == 0, "addr is not aligned to T");
    }

    debug_assert!(!addr.is_null(), "ptr must not be null");

    unsafe { ptr::read(addr.as_ptr()) }
}

/// Read bytes from address
///
/// SAFETY:
///     - Memory at location must be initialized
///     - Ptr must be non-null
///     - Should only use for runtime addresses (otherwise lack of provenance is UB)
///     - Address must be valid for reads
///     - Address must be valid for reads up to addr+count bytes
pub unsafe fn read_bytes(src: Address, count: usize) -> Vec<u8> {
    debug_assert!(!src.is_null(), "src must not be null");

    let mut buffer = Vec::with_capacity(count);

    unsafe {
        ptr::copy_nonoverlapping(src as _, buffer.as_mut_ptr(), count);
    }

    unsafe {
        buffer.set_len(count);
    }

    buffer
}

/// Write T to dst
///
/// SAFETY:
///     - dst must be aligned
///     - dst must be valid for writes
pub unsafe fn write<T>(dst: Address, src: T) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    if cfg!(debug_assertions) {
        let align = mem::align_of::<T>();

        assert!(dst % align == 0, "dst is not aligned to T");
    }

    unsafe {
        ptr::write(dst.as_mut(), src);
    }
}

/// Write bytes to dst
///
/// SAFETY:
///     - dst must be valid for writes up to count bytes
///     - addresses must not overlap
pub unsafe fn write_bytes(src: &[u8], dst: Address) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    unsafe { write_raw(src.as_ptr() as _, dst, src.len()) }
}

/// Write ptr to dst
///
/// SAFETY:
///     - src must be valid for reads up to count bytes
///     - dst must be valid for writes up to count bytes
///     - addresses must not overlap
pub unsafe fn write_raw(src: Address, dst: Address, count: usize) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    unsafe {
        ptr::copy_nonoverlapping(src.as_ptr::<u8>(), dst.as_mut(), count);
    }
}

/// Set dst to val for N bytes
///
/// SAFETY:
///     - dst must be valid for writes up to count bytes
pub unsafe fn set(dst: Address, val: u8, count: usize) {
    debug_assert!(!dst.is_null(), "dst must not be null");

    unsafe {
        ptr::write_bytes(dst.as_mut::<u8>(), val, count);
    }
}

pub unsafe fn prot(addr: Address, mut size: usize, prot: Prot) -> Result<Prot, MemError> {
    if addr.is_null() {
        return Err(MemError::BadAddress);
    }

    if size == 0 {
        size = get_page_size() as usize;
    }

    let mut old_prot = PAGE_PROTECTION_FLAGS::default();
    unsafe {
        VirtualProtect(addr.as_ptr(), size, prot.into(), &mut old_prot)?;
    }

    Ok(old_prot.into())
}

pub fn alloc(mut size: usize, prot: Prot) -> Result<Alloc, MemError> {
    if size == 0 {
        size = get_page_size() as usize;
    }

    let alloc = unsafe { VirtualAlloc(None, size, MEM_COMMIT | MEM_RESERVE, prot.into()) };
    if alloc.is_null() {
        return Err(MemError::BadAddress);
    }

    let alloc = Alloc(unsafe { NonNull::new_unchecked(alloc.cast()) });

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
