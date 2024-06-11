//! This module allows one to scan memory for specific data

mod aligned_bytes;
mod backends;
mod pattern;
use std::fmt::{self, Display};

use self::pattern::PatternError;

/// Scanning errors
#[derive(Debug, thiserror::Error)]
enum ScannerError {
    /// An error happened during pattern parsing
    #[error(transparent)]
    Pattern(#[from] PatternError),
}

/// The result of a scan
#[derive(Debug)]
pub struct Scan {
    /// the address of a found match
    pub addr: *const u8,
}

impl Display for Scan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scan {{ addr: {:?} }}", self.addr)
    }
}

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
/// let scanner = Scanner::new("48 89 5c 24 ?? 48 89 6c");
/// let result = unsafe { scanner.find(binary.as_ptr(), binary.len()) };
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
/// let binary = [0xab, 0xec, 0x48, 0x89, 0x5c, 0x24, 0xee, 0x48, 0x89, 0x6c];
///
/// let scanner = Scanner::new("48 89 5c 24 ?? 48 89 6c");
/// let result = unsafe { scanner.find(binary.as_ptr(), binary.len()) };
///
/// println!("{:?}", result);
/// ```
pub unsafe fn data_scan(data: &[u8], addr: *const u8, size: usize) -> Option<Scan> {
    let pattern = data.into();
    unsafe { backends::find(&pattern, addr, size) }
}
