#![no_std]

//! Defined Traits for bondrewd-derive.
//! For Derive Docs see [bondrewd-derive](https://docs.rs/bondrewd-derive/latest/bondrewd_derive/)
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
#[cfg(feature = "hex_fns")]
pub use error::BitfieldHexError;
#[cfg(feature = "slice_fns")]
pub use error::BitfieldSliceError;
#[cfg(feature = "hex_fns")]
pub trait BitfieldHex<const SIZE: usize>
where
    Self: Sized,
{
    const UPPERS: &'static [u8; 16] = b"0123456789ABCDEF";
    const LOWERS: &'static [u8; 16] = b"0123456789abcdef";
    fn from_hex(hex: [u8; SIZE]) -> Result<Self, BitfieldHexError>;
    fn into_hex_upper(self) -> [u8; SIZE];
    fn into_hex_lower(self) -> [u8; SIZE];
}

// re-export the derive stuff
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use bondrewd_derive::*;

#[cfg(all(not(feature = "derive"), feature = "slice_fns"))]
compile_error!("the slice_fns attribute depends on the derive attribute");
