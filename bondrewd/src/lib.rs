pub trait Bitfields<const SIZE: usize> {
    const BYTE_SIZE: usize = SIZE;
    const BIT_SIZE: usize;
    fn into_bytes(self) -> [u8; SIZE];
    fn from_bytes(input_byte_buffer: [u8; SIZE]) -> Self;
}

pub trait BitfieldEnum {
    type Primitive;
    fn from_primitive(prim: Self::Primitive) -> Self;
    fn into_primitive(self) -> Self::Primitive;
}

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
