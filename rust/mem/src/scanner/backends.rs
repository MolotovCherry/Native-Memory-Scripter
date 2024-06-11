#[cfg(target_arch = "x86_64")]
mod avx2;
mod scalar;
#[cfg(target_arch = "x86_64")]
mod sse42;

use super::{Pattern, Scan, ScanError};

/// # Safety
/// address must be valid and have exposed provenance for up to size reads
pub(crate) unsafe fn find(
    pattern: &Pattern,
    ptr: *const u8,
    size: usize,
) -> Result<Scan, ScanError> {
    #[cfg(target_arch = "x86_64")]
    {
        let avx2 = is_x86_feature_detected!("avx2");
        let sse42 = is_x86_feature_detected!("sse4.2");

        if avx2 {
            // SAFETY: safe to call as long as the safety conditions were met for this function
            return unsafe { avx2::find(pattern, ptr, size) };
        } else if sse42 {
            // SAFETY: safe to call as long as the safety conditions were met for this function
            return unsafe { sse42::find(pattern, ptr, size) };
        }
    }

    // SAFETY: safe to call as long as the safety conditions were met for this function
    unsafe { scalar::find(pattern, ptr, size) }
}
