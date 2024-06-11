//! This module allows one to scan pages

use std::mem;

use windows::Win32::System::Memory::{VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_FREE};

use crate::Prot;

/// A segment (page)
#[derive(Debug, Copy, Clone)]
pub struct Segment {
    /// the base address of the page
    pub base: *const (),
    /// the end address of the page
    pub end: *const (),
    /// the page's size
    pub size: usize,
    /// the page's protection flag
    pub prot: Prot,
}

unsafe impl Send for Segment {}
unsafe impl Sync for Segment {}

fn enum_segments_cb(mut cb: impl FnMut(Segment) -> bool) {
    let mut address = 0;
    let mut mem_info = MEMORY_BASIC_INFORMATION::default();

    let mut written = 0;
    while written > 0 {
        // conditions at top
        address += mem_info.RegionSize;

        written = unsafe {
            VirtualQuery(
                Some(address as _),
                &mut mem_info,
                mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };

        if mem_info.State == MEM_FREE {
            continue;
        }

        let segment = Segment {
            base: mem_info.BaseAddress.cast(),
            end: unsafe { mem_info.BaseAddress.add(mem_info.RegionSize).cast() },
            size: mem_info.RegionSize,
            prot: mem_info.AllocationProtect.into(),
        };

        if cb(segment) {
            break;
        }
    }
}

/// Enumerates all the segments in the calling process, returning them on a vector.
pub fn enum_segments() -> Vec<Segment> {
    let mut segments = Vec::new();

    enum_segments_cb(|segment| {
        segments.push(segment);
        false
    });

    segments
}

/// Finds a segment in the calling process from a virtual address.
pub fn find_segment(address: *const ()) -> Option<Segment> {
    let mut segment = None;

    enum_segments_cb(|segment_in| {
        if address >= segment_in.base && address < segment_in.end {
            segment = Some(segment_in);
            return true;
        }

        false
    });

    segment
}
