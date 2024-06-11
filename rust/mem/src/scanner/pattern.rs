use std::fmt;

use super::aligned_bytes::AlignedBytes;

#[derive(Debug, thiserror::Error)]
pub enum PatternError {
    #[error("pattern is invalid. pattern must be a-f, A-F, 0-9, or ?? or ? for wildcards")]
    Invalid,
}

/// An IDA-style binary pattern
pub struct Pattern {
    pub(crate) data: AlignedBytes,
    pub(crate) mask: AlignedBytes,
    pub(crate) unpadded_size: usize,
}

impl fmt::Debug for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Pattern { data, .. } = self;

        let data = &**data;
        write!(f, r#"Pattern({data:?})"#)
    }
}

impl Pattern {
    const ALIGN: usize = 32;

    /// Create a new IDA-style [`Pattern`] instance
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Pattern::new("48 89 ?? 24 ?? 48 89 6c");
    /// Pattern::new("48 89 ? 24 ? 48 89 6c");
    /// ```
    pub fn new(pattern: &str) -> Result<Self, PatternError> {
        let char_to_byte = |c| match c {
            c if matches!(c, 'a'..='f') => c as u8 - b'a' + 0xA,
            c if matches!(c, 'A'..='F') => c as u8 - b'A' + 0xA,
            c if c.is_ascii_digit() => c as u8 - b'0',
            _ => unreachable!(),
        };

        let mut data = Vec::new();
        let mut mask = Vec::new();

        let mut pattern = pattern.chars().peekable();

        while let Some(sym) = pattern.next() {
            let next_sym = pattern.peek().copied();

            match sym {
                ' ' => (),

                '?' => {
                    data.push(0x00);
                    mask.push(0x00);

                    pattern.next_if_eq(&'?');
                }

                _ => {
                    // check if iterator got out of sync, which indicates a partial match
                    let Some(next_sym) = next_sym else {
                        return Err(PatternError::Invalid);
                    };

                    // only hex digits are allowed; a-f A-F 0-9
                    if !sym.is_ascii_hexdigit() || !next_sym.is_ascii_hexdigit() {
                        return Err(PatternError::Invalid);
                    }

                    let byte = char_to_byte(sym) << 4 | char_to_byte(next_sym);

                    data.push(byte);
                    mask.push(0xFF);

                    pattern.next();
                }
            }
        }

        let unpadded_size = data.len();

        let count = f32::ceil(unpadded_size as f32 / Self::ALIGN as f32) as usize;
        let padding_size = count * Self::ALIGN - unpadded_size;

        data.resize(unpadded_size + padding_size, 0);
        mask.resize(unpadded_size + padding_size, 0);

        // SAFETY: our align is a power of 2 above
        let slf = Self {
            data: unsafe { AlignedBytes::new(&data, Self::ALIGN).unwrap_unchecked() },
            mask: unsafe { AlignedBytes::new(&mask, Self::ALIGN).unwrap_unchecked() },
            unpadded_size,
        };

        Ok(slf)
    }
}

impl TryFrom<&str> for Pattern {
    type Error = PatternError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
