use core::fmt;

/// Not enough bytes were provided in a slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitfieldLengthError(
    /// Amount of provided bytes.
    pub usize,
    /// Amount of expected bytes.
    pub usize,
);

impl BitfieldLengthError {
    /// Amount of bytes that were provided.
    #[must_use]
    pub fn provided(&self) -> usize {
        self.0
    }
    /// Amount of bytes that were required.
    #[must_use]
    pub fn expected(&self) -> usize {
        self.1
    }
}

impl fmt::Display for BitfieldLengthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected {} bytes, {} bytes were provided", self.1, self.0)
    }
}

impl core::error::Error for BitfieldLengthError {}
