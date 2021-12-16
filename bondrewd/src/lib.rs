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

mod error;
#[cfg(feature = "peek_slice")]
pub use error::BitfieldPeekError;

// re-export the derive stuff
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use bondrewd_derive::*;

#[cfg(all(not(feature = "derive"), feature = "peek_slice"))]
compile_error!("the peek_slice attribute depends on the derive attribute");
