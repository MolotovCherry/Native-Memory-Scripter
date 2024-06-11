//! This module allows one to scan memory for specific data

mod aligned_bytes;
mod backends;
mod pattern;

use self::pattern::{Pattern, PatternError};

/// Scanning errors
#[derive(Debug, thiserror::Error)]
enum ScannerError {
    /// An error happened during pattern parsing
    #[error(transparent)]
    Pattern(#[from] PatternError),
}

/// The result of a scan
#[derive(Debug, Copy, Clone)]
pub struct Scan {
    /// the address of a found match
    pub addr: *const u8,
}

unsafe impl Send for Scan {}
unsafe impl Sync for Scan {}

/// Single result IDA-style pattern scanner
///
/// A pattern scanner that searches for an IDA-style pattern
/// and returns the pointer to the first occurrence in the binary.
///
/// Find the first occurence of the pattern in the binary
///
/// # Params
///
/// * `addr` - pointer to the first element of the binary to search the pattern in
///
/// * `size` - binary size
///
/// # Safety
///
/// * `addr` - is a valid pointer
///
/// * `size` - corresponds to a valid size of `binary`
///
/// # Example
///
/// ```rust,ignore
/// let binary = [0xab, 0xec, 0x48, 0x89, 0x5c, 0x24, 0xee, 0x48, 0x89, 0x6c];
///
/// let pattern = "48 89 5c 24 ?? 48 89 6c";
/// let result = unsafe { sig_scan(pattern, binary.as_ptr(), binary.len()) };
///
/// println!("{:?}", result);
/// ```
pub unsafe fn sig_scan(pattern: &str, addr: *const u8, size: usize) -> Option<Scan> {
    let pattern = pattern.try_into().ok()?;
    // SAFETY: safe to call as long as the safety conditions were met for this function
    unsafe { backends::find(&pattern, addr, size) }
}

/// Scan address for data.
///
/// Find the first occurence of the pattern in the binary
///
/// # Params
///
/// * `data` - the binary data to search for
///
/// * `addr` - pointer to the first element of the binary to search the pattern in
///
/// * `size` - binary size
///
/// # Safety
///
/// * `addr` - is a valid pointer
///
/// * `size` - corresponds to a valid size of `binary`
///
/// # Example
///
/// ```rust,ignore
/// let search = [0x24, 0xee, 0x48];
/// let binary = [0xab, 0xec, 0x48, 0x89, 0x5c, 0x24, 0xee, 0x48, 0x89, 0x6c];
///
/// let result = unsafe { data_scan(&search, binary.as_ptr(), binary.len()) };
///
/// println!("{:?}", result);
/// ```
pub unsafe fn data_scan(data: &[u8], addr: *const u8, size: usize) -> Option<Scan> {
    let pattern = data.into();
    unsafe { backends::find(&pattern, addr, size) }
}

/// Scan address for data with a mask
///
/// Find the first occurence of the pattern in the binary
///
/// # Params
///
/// * `data` - the binary data to search for
///
/// * `mask` - the mask to apply to the data. use `x` for known byte, `?` for unknown byte
///
/// * `addr` - pointer to the first element of the binary to search the pattern in
///
/// * `size` - binary size
///
/// # Safety
///
/// * `addr` - is a valid pointer
///
/// * `size` - corresponds to a valid size of `binary`
///
/// # Example
///
/// ```rust,ignore
/// let binary = [0xab, 0xec, 0x00, 0x89, 0x5c, 0x00, 0xee, 0x00, 0x89, 0x6c];
///
/// let result = unsafe { pattern_scan(&binary, "xx?xx?x?xx", address, 200) };
///
/// println!("{:?}", result);
/// ```
pub unsafe fn pattern_scan(data: &[u8], mask: &str, addr: *const u8, size: usize) -> Option<Scan> {
    let pattern = Pattern::from_data_with_mask(data, mask).ok()?;
    unsafe { backends::find(&pattern, addr, size) }
}
