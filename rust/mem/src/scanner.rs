//! This module allows one to scan memory for specific data

mod aligned_bytes;
mod backends;
mod pattern;
use std::fmt::{self, Display};

use self::pattern::PatternError;
pub use pattern::Pattern;

/// Scan result errors
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    /// Pattern not found
    #[error("pattern not found")]
    NotFound,
}

/// Scanning errors
#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
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
        write!(f, "Scan {{ addr: 0x{:X?} }}", self.addr)
    }
}

/// Single result IDA-style pattern scanner
///
/// A pattern scanner that searches for an IDA-style pattern
/// and returns the pointer to the first occurrence in the binary.

pub struct Scanner(Pattern);

impl Scanner {
    /// Create a new [`Scanner`] instance
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let scanner = Scanner::new("48 89 5c 24 ?? 48 89 6c");
    /// ```
    pub fn new(pattern: &str) -> Result<Self, ScannerError> {
        let pat = Pattern::new(pattern)?;
        Ok(Self(pat))
    }

    /// Find the first occurence of the pattern in the binary
    ///
    /// # Params
    ///
    /// * `ptr` - pointer to the first element of the binary to search the pattern in
    ///
    /// * `size` - binary size
    ///
    /// # Safety
    ///
    /// * `ptr` - is a valid pointer
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
    pub unsafe fn find(&self, ptr: *const u8, size: usize) -> Result<Scan, ScanError> {
        // SAFETY: safe to call as long as the safety conditions were met for this function
        unsafe { backends::find(&self.0, ptr, size) }
    }
}
