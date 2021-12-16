/// Currently there is only 1 error type which is not enough bytes provided to peek at field.
/// (amount of bytes provided , amount of bytes required)
#[derive(Debug)]
pub struct BitfieldPeekError(pub usize, pub usize);

impl std::fmt::Display for BitfieldPeekError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            fmt,
            "expected {} bytes, {} bytes were provided.",
            self.1, self.0
        )
    }
}

impl std::error::Error for BitfieldPeekError {}