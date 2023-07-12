#![cfg_attr(not(feature = "std"), no_std)]

//! Defined Traits for bondrewd-derive.
//! For Derive Docs see [bondrewd-derive](https://docs.rs/bondrewd-derive/latest/bondrewd_derive/)
pub trait Bitfields<const SIZE: usize> {
    /// Total amount of Bytes the Bitfields within this structure take to contain in a fixed size array.
    const BYTE_SIZE: usize = SIZE;
    /// Total amount of Bits the Bitfields within this structure take to contain in a fixed size array.
    const BIT_SIZE: usize;
    /// Inserts the values of the Bitfields in this structure into a fixed size array, consuming the structure.
    ///
    /// Returns a fixed sized byte array containing the Bitfields of the provided structure.
    fn into_bytes(self) -> [u8; SIZE];
    /// Extracts the values of the Bitfields in this structure from a fixed size array while consuming it.
    ///
    /// Returns Self with the fields containing the extracted values from provided fixed size array of bytes.
    fn from_bytes(input_byte_buffer: [u8; SIZE]) -> Self;
}

#[deprecated(
    since = "0.1.15",
    note = "please use `Bitfields` instead of `BitfieldEnum`"
)]
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
    /// Extracts the values of the Bitfields in this structure from a hex encoded fixed size byte array
    /// while consuming it.
    ///
    /// Returns Self with the fields containing the extracted values from provided hex encoded fixed size
    /// array of bytes.
    fn from_hex(hex: [u8; SIZE]) -> Result<Self, BitfieldHexError>;
    /// Inserts the values of the Bitfields in this structure into a fixed size array with upper case hex
    /// encoding, consuming the structure.
    ///
    /// Returns a hex encoded fixed sized byte array containing the Bitfields of the provided structure.
    fn into_hex_upper(self) -> [u8; SIZE];
    /// Inserts the values of the Bitfields in this structure into a fixed size array with lower case hex
    /// encoding, consuming the structure.
    ///
    /// Returns a hex encoded fixed sized byte array containing the Bitfields of the provided structure.
    fn into_hex_lower(self) -> [u8; SIZE];
}

// re-export the derive stuff
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use bondrewd_derive::*;

#[cfg(all(not(feature = "derive"), feature = "slice_fns"))]
compile_error!("the slice_fns attribute depends on the derive attribute");
