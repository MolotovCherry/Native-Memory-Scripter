//! AVX2 pattern scanning backend

use std::arch::x86_64::{
    _mm256_blendv_epi8, _mm256_cmpeq_epi8, _mm256_load_si256, _mm256_loadu_si256,
    _mm256_movemask_epi8, _mm256_set1_epi8,
};

use crate::scan::{pattern::Pattern, Scan};

/// Find the first occurrence of a pattern in the binary
/// using AVX2 instructions
///
/// # Safety
///
/// * `ptr` - is a valid pointer
///
/// * `size` - corresponds to a valid size of `binary`
///
/// * Currently running CPU supports AVX2
#[target_feature(enable = "avx2")]
pub unsafe fn find(pattern_data: &Pattern, ptr: *const u8, size: usize) -> Option<Scan> {
    const UNIT_SIZE: usize = 32;

    let mut processed_size = 0;

    let data_base = pattern_data.data.as_ptr();
    let mask_base = pattern_data.mask.as_ptr();

    // SAFETY: this function is only called if the CPU supports AVX2

    let mut pattern = unsafe { _mm256_load_si256(data_base.cast()) };
    let mut mask = unsafe { _mm256_load_si256(mask_base.cast()) };
    let all_zeros = unsafe { _mm256_set1_epi8(0x00) };

    let mut chunk = 0;
    while chunk < size {
        let chunk_data = unsafe { _mm256_loadu_si256(ptr.add(chunk).cast()) };

        let blend = unsafe { _mm256_blendv_epi8(all_zeros, chunk_data, mask) };
        let eq = unsafe { _mm256_cmpeq_epi8(pattern, blend) };

        if unsafe { _mm256_movemask_epi8(eq) as u32 == 0xffffffff } {
            processed_size += UNIT_SIZE;

            if processed_size < pattern_data.unpadded_size {
                chunk += UNIT_SIZE - 1;

                pattern = unsafe { _mm256_load_si256(data_base.add(processed_size).cast()) };
                mask = unsafe { _mm256_load_si256(mask_base.add(processed_size).cast()) };
            } else {
                let addr = unsafe { ptr.add(chunk).sub(processed_size).add(UNIT_SIZE) };

                let scan = Scan { addr };

                return Some(scan);
            }
        } else {
            pattern = unsafe { _mm256_load_si256(data_base.cast()) };
            mask = unsafe { _mm256_load_si256(mask_base.cast()) };
            processed_size = 0;
        }

        chunk += 1;
    }

    None
}
