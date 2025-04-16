#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::module_name_repetitions)]
//! Defined Traits for bondrewd-derive.
//!
//! For Derive Docs see [bondrewd-derive](https://docs.rs/bondrewd-derive/latest/bondrewd_derive/)

mod error;
#[cfg(all(feature = "dyn_fns", feature = "hex_fns"))]
pub use error::BitfieldHexDynError;
#[cfg(feature = "hex_fns")]
pub use error::BitfieldHexError;
#[cfg(feature = "dyn_fns")]
pub use error::BitfieldLengthError;

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

#[cfg(feature = "dyn_fns")]
pub trait BitfieldsDyn<const SIZE: usize>: Bitfields<SIZE>
where
    Self: Sized,
{
    /// If `Ok(bitfield_struct)` is returned, the required bytes to create the object will be removed from
    /// `input_byte_buffer`.
    ///
    /// # Errors
    /// If there is not enough bytes to create the object from `input_byte_buffer`.
    #[cfg(feature = "std")]
    fn from_vec(input_byte_buffer: &mut Vec<u8>) -> Result<Self, BitfieldLengthError>;
    /// If `Ok(bitfield_struct)` is returned, the required bytes to create the object will be copied from
    /// `input_byte_buffer`.
    ///
    /// # Errors
    /// If there is not enough bytes to create the object from `input_byte_buffer`.
    fn from_slice(input_byte_buffer: &[u8]) -> Result<Self, BitfieldLengthError>;
}

#[cfg(feature = "dyn_fns")]
pub trait BitfieldsSlice<const SIZE: usize>: Bitfields<SIZE> + Sized {
    /// Returns a "checked" slice type for the type. This typically should be a structure that stores a slice of bytes,
    /// that are confirmed to contain enough bytes for all fields. this allows the user to read specific fields
    /// from the byte slice rather than getting all fields with `from_bytes`.
    fn check_slice(slice: &[u8]) -> Result<Self, BitfieldLengthError>;
    /// Returns a mutable "checked" slice type for the type. This typically should be a structure that stores a slice of bytes,
    /// that are confirmed to contain enough bytes for all fields. this allows the user to read/write specific fields
    /// from/to the byte slice rather getting all fields with `from_bytes` and or outputting the new bytes with `into_bytes`.
    fn check_slice_mut(slice: &mut [u8]) -> Result<Self, BitfieldLengthError>;
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
#[cfg(feature = "hex_fns")]
mod hex;
#[cfg(feature = "hex_fns")]
pub use hex::BitfieldHex;
#[cfg(all(feature = "hex_fns", feature = "dyn_fns"))]
pub use hex::BitfieldHexDyn;

// re-export the derive stuff
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use bondrewd_derive::*;
