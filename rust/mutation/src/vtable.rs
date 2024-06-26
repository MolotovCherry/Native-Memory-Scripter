//! This module allows one to interact with Virtual Method Tables (VMTs) from OOP objects.

use std::{fmt, mem, sync::Mutex};

use crate::{
    memory::{self, MemError},
    Prot,
};

/// VTable errors
#[derive(Debug, thiserror::Error)]
pub enum VTableError {
    /// A mem error happened
    #[error(transparent)]
    Mem(#[from] MemError),
}

#[derive(Debug)]
struct VTableEntry {
    /// The original address of the vtable entry
    orig_fn: *const (),
    /// The index of the vtable entry
    index: usize,
}

/// VTable
///
/// When dropped, will auto unhook everything
pub struct VTable {
    /// Pointer to the base vtable address
    base: *mut u64,
    /// Altered vtable entries
    entries: Mutex<Vec<VTableEntry>>,
}

unsafe impl Send for VTable {}
unsafe impl Sync for VTable {}

impl fmt::Debug for VTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let VTable { base, .. } = self;

        f.debug_tuple("VTable").field(&base).finish()
    }
}

impl Drop for VTable {
    fn drop(&mut self) {
        _ = unsafe { self.reset() };
    }
}

impl VTable {
    /// Create a new vtable hooker
    pub fn new(vtable: *mut u64) -> Self {
        Self {
            base: vtable,
            entries: Mutex::new(Vec::new()),
        }
    }

    /// Hook the vtables index with a new function
    ///
    /// # Safety
    /// - Dst must be valid
    /// - Dst must be to a function with the same signature as original
    /// - Index must be valid
    pub unsafe fn hook(&self, index: usize, dst: *const ()) -> Result<(), VTableError> {
        let mut lock = self.entries.lock().unwrap();

        let index_ptr = unsafe { self.base.add(index) };

        if !lock.iter().any(|e| e.index == index) {
            let orig_fn = unsafe { memory::read(index_ptr) };
            let entry = VTableEntry {
                orig_fn: orig_fn as *const (),
                index,
            };

            lock.push(entry);
        }

        let old = unsafe { memory::prot(index_ptr.cast(), mem::size_of::<u64>(), Prot::XRW)? };

        unsafe {
            memory::write(index_ptr, dst as _);
        }

        unsafe {
            memory::prot(index_ptr.cast(), mem::size_of::<u64>(), old)?;
        }

        Ok(())
    }

    /// Unhook a hooked index. If index wasn't hooked, does nothing.
    ///
    /// # Safety
    /// Overwrites vtable fn pointer if it was altered. Take great care
    pub unsafe fn unhook(&self, index: usize) -> Result<(), VTableError> {
        let lock = self.entries.lock().unwrap();

        let Some(item) = lock.iter().find(|i| i.index == index) else {
            return Ok(());
        };

        let index_ptr = unsafe { self.base.add(item.index) };

        let old = unsafe { memory::prot(index_ptr.cast(), mem::size_of::<u64>(), Prot::XRW)? };

        unsafe {
            memory::write(index_ptr, item.orig_fn as u64);
        }

        unsafe {
            memory::prot(index_ptr.cast(), mem::size_of::<u64>(), old)?;
        }

        Ok(())
    }

    /// Get the original vtable fn pointer for index
    pub fn get_original(&self, index: usize) -> Option<*const ()> {
        let lock = self.entries.lock().unwrap();
        lock.iter().find(|e| e.index == index).map(|e| e.orig_fn)
    }

    /// Reset all altered vtable entries
    ///
    /// # Safety
    /// Overwrites all vtable fn pointers that were altered. Take great care
    pub unsafe fn reset(&self) -> Result<(), VTableError> {
        let lock = self.entries.lock().unwrap();

        for item in &*lock {
            let index_ptr = unsafe { self.base.add(item.index) };

            let old = unsafe { memory::prot(index_ptr.cast(), mem::size_of::<u64>(), Prot::XRW)? };

            unsafe {
                memory::write(index_ptr, item.orig_fn as u64);
            }

            unsafe {
                memory::prot(index_ptr.cast(), mem::size_of::<u64>(), old)?;
            }
        }

        Ok(())
    }
}
