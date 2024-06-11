//! Aligned byte storage implementation

use core::slice;
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout, LayoutError},
    marker::PhantomData,
    ops::Deref,
    ptr::{self, NonNull},
};

#[derive(Debug, thiserror::Error)]
pub enum AlignedBytesError {
    #[error(transparent)]
    Layout(#[from] LayoutError),
}

#[derive(Debug)]
#[repr(C)]
pub struct AlignedBytes {
    data: *mut u8,
    len: usize,
    layout: Option<Layout>,
    _phantom: PhantomData<Vec<u8>>,
}

unsafe impl Send for AlignedBytes {}
unsafe impl Sync for AlignedBytes {}

impl AlignedBytes {
    /// Create a new `AlignedBytes` instance from a slice
    pub(crate) fn new(data: &[u8], align: usize) -> Result<AlignedBytes, AlignedBytesError> {
        if data.is_empty() {
            let ptr = NonNull::dangling().as_ptr();
            // SAFETY: len is 0, so we will be dereffing a ZST pointer
            let slf = Self {
                data: ptr,
                len: 0,
                layout: None,
                _phantom: PhantomData,
            };

            Ok(slf)
        } else {
            let layout = Layout::from_size_align(data.len(), align)?;
            // SAFETY: Layout is correct cause we used the safe method to make it
            let ptr = unsafe { alloc(layout) };

            if ptr.is_null() {
                handle_alloc_error(layout);
            }

            // SAFETY: We allocated data.len(), so we can copy data.len() to it
            unsafe {
                ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
            }

            let slf = Self {
                data: ptr,
                len: data.len(),
                layout: Some(layout),
                _phantom: PhantomData,
            };

            Ok(slf)
        }
    }
}

impl Deref for AlignedBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }
}

impl Drop for AlignedBytes {
    fn drop(&mut self) {
        if let Some(layout) = self.layout.take() {
            unsafe { dealloc(self.data, layout) }
        }
    }
}
