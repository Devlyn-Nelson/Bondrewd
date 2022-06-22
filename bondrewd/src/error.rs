
//! Error types for Bondrewd Functionality which can fail. Base bondrewd with no
//! features other than derive will have no errors types.

#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(feature = "std")]
use std::fmt;

/// Error type describing that not enough bytes were provided in a slice.
/// The first value contains the provided amount of bytes.
/// The second value contains the expected amount of bytes.
#[derive(Debug)]
pub struct BitfieldSliceError(pub usize, pub usize);

impl fmt::Display for BitfieldSliceError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "Expected {} bytes, {} bytes were provided.",
            self.1, self.0
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BitfieldSliceError {}

/// Error type describing that a character in provided slice is Invalid.
/// The first value contains the Invalid character.
/// The second value contains the index of the Invalid character.
#[derive(Debug)]
pub struct BitfieldHexError(pub char, pub usize);

impl fmt::Display for BitfieldHexError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "Found Invalid character {} @ index {}.",
            self.0, self.1
        )
    }
}
