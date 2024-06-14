//! This module allows one to interact with Virtual Method Tables (VMTs) from OOP objects.

use std::{mem, sync::Mutex};

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

/// VTable
#[derive(Debug)]
pub struct VTable {
    /// Pointer to the base vtable address
    base: *mut u64,
    /// Altered vtable entries
    entries: Mutex<Vec<VTableEntry>>,
}

unsafe impl Send for VTable {}
unsafe impl Sync for VTable {}

#[derive(Debug)]
struct VTableEntry {
    /// The original address of the vtable entry
    orig_fn: *const (),
    /// The index of the vtable entry
    index: usize,
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
        let mut lock = self.entries.lock().unwrap();

        let Some(item_idx) = lock.iter().position(|i| i.index == index) else {
            return Ok(());
        };

        let item = lock.remove(item_idx);
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

    /// Get the original vtable pointer for index
    pub fn get_original(&self, index: usize) -> Option<*const ()> {
        let lock = self.entries.lock().unwrap();
        lock.iter().find(|e| e.index == index).map(|e| e.orig_fn)
    }

    /// Reset all altered vtable entries
    ///
    /// # Safety
    /// Overwrites all vtable fn pointers that were altered. Take great care
    pub unsafe fn reset(&self) -> Result<(), VTableError> {
        let mut lock = self.entries.lock().unwrap();
        lock.drain(..).try_for_each(|e| {
            let index_ptr = unsafe { self.base.add(e.index) };

            let old = unsafe { memory::prot(index_ptr.cast(), mem::size_of::<u64>(), Prot::XRW)? };

            unsafe {
                memory::write(index_ptr, e.orig_fn as u64);
            }

            unsafe {
                memory::prot(index_ptr.cast(), mem::size_of::<u64>(), old)?;
            }

            Ok::<_, VTableError>(())
        })?;

        Ok(())
    }
}
