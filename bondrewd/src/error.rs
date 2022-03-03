/// Currently there is only 1 error type which is not enough bytes provided to slice at field.
/// (amount of bytes provided , amount of bytes required)
#[derive(Debug)]
pub struct BitfieldSliceError(pub usize, pub usize);

impl core::fmt::Display for BitfieldSliceError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            fmt,
            "expected {} bytes, {} bytes were provided.",
            self.1, self.0
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BitfieldSliceError {}

#[derive(Debug)]
pub struct BitfieldHexError(pub char, pub usize);

impl core::fmt::Display for BitfieldHexError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            fmt,
            "found Invalid character {} @ index {}.",
            self.0, self.1
        )
    }
}
