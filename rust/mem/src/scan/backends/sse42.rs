//! SSE4.2 pattern scanning backend

use std::arch::x86_64::{
    _mm_blendv_epi8, _mm_cmpeq_epi8, _mm_load_si128, _mm_loadu_si128, _mm_movemask_epi8,
    _mm_set1_epi8,
};

use crate::scan::{pattern::Pattern, Scan};

/// Find the first occurrence of a pattern in the binary
/// using SSE4.2 instructions
///
/// # Safety
///
/// * `ptr` - is a valid pointer
///
/// * `size` - corresponds to a valid size of `binary`
///
/// * Currently running CPU supports SSE4.2
#[target_feature(enable = "sse4.2")]
pub unsafe fn find(pattern_data: &Pattern, ptr: *const u8, size: usize) -> Option<Scan> {
    const UNIT_SIZE: usize = 16;

    let mut processed_size = 0;

    // SAFETY: this function is only called if the CPU supports SSE4.2

    let mut pattern = unsafe { _mm_load_si128(pattern_data.data.as_ptr() as *const _) };
    let mut mask = unsafe { _mm_load_si128(pattern_data.mask.as_ptr() as *const _) };
    let all_zeros = unsafe { _mm_set1_epi8(0x00) };

    let mut chunk = 0;

    while chunk < size {
        let chunk_data = unsafe { _mm_loadu_si128(ptr.add(chunk) as *const _) };
        let blend = unsafe { _mm_blendv_epi8(all_zeros, chunk_data, mask) };
        let eq = unsafe { _mm_cmpeq_epi8(pattern, blend) };

        if unsafe { _mm_movemask_epi8(eq) == 0xffff } {
            processed_size += UNIT_SIZE;

            if processed_size < pattern_data.unpadded_size {
                chunk += UNIT_SIZE - 1;

                pattern = unsafe {
                    _mm_load_si128(pattern_data.data.as_ptr().add(processed_size) as *const _)
                };

                mask = unsafe {
                    _mm_load_si128(pattern_data.mask.as_ptr().add(processed_size) as *const _)
                };
            } else {
                let addr = unsafe { ptr.add(chunk).sub(processed_size).add(UNIT_SIZE) };

                let scan = Scan { addr };

                return Some(scan);
            }
        } else {
            pattern = unsafe { _mm_load_si128(pattern_data.data.as_ptr() as *const _) };
            mask = unsafe { _mm_load_si128(pattern_data.mask.as_ptr() as *const _) };
            processed_size = 0;
        }

        chunk += 1;
    }

    None
}
