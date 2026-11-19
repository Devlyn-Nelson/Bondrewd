#![no_std]
//! Bit-level field packing.
//!
//! Structures and enums deriving [`Bitfields`] get `into_bytes`/`from_bytes` plus
//! per-field `read_<field>`/`write_<field>` associated functions. Deriving
//! [`BitfieldsSlice`] adds fallible `read_slice_<field>`/`write_slice_<field>` functions
//! and checked slice wrapper types.
//!
//! # Bit layout model
//!
//! Every type has `BIT_SIZE` *logical* bits. Fields are laid out in declaration order
//! (unless placed explicitly with `bits = "a..b"`), so field `n` starts where field
//! `n - 1` ended. How logical bits land in the bytes depends on the type's endianness:
//!
//! * `be` / `le`: logical bit `0` is the most significant bit of byte `0` (bits are
//!   read left to right). `be` stores each value most-significant-bit first. `le`
//!   stores the value's little-endian bytes in order, each byte MSB first; when the
//!   bit length isn't a multiple of 8, the final (most significant) byte keeps only its
//!   low `bit_length % 8` bits.
//! * `ale` (aligned little endian): the whole buffer is one little-endian integer and
//!   logical bit `0` is its bit `BYTE_SIZE * 8 - BIT_SIZE` (the "right" side). Values are
//!   stored least-significant-bit first, so a field's value bits stay in order inside the
//!   integer. Unused padding bits sit at the least significant end.
//!
//! Struct-level modifiers: `reverse` reverses the output byte order and
//! `bit_traversal = "back"` reverses the field order inside the `BIT_SIZE` bits.
//!
//! Nested `Bitfields` types are embedded by copying their `BIT_SIZE` logical bits in
//! logical order, so a nested type keeps its own internal layout no matter what the
//! parent uses.

#[cfg(feature = "alloc")]
extern crate alloc;

extern crate self as bondrewd;

mod error;
pub use error::BitfieldLengthError;

#[cfg(feature = "derive")]
pub use bondrewd_derive::{Bitfields, BitfieldsSlice};

/// A type that can be packed into, and unpacked from, a fixed size byte array.
///
/// `SIZE` is the number of bytes in the packed representation.
pub trait Bitfields<const SIZE: usize>: Sized {
    /// Total amount of bytes the bitfields occupy.
    const BYTE_SIZE: usize = SIZE;
    /// Total amount of bits the bitfields occupy.
    const BIT_SIZE: usize;

    /// Packs `self` into a byte array.
    fn into_bytes(self) -> [u8; SIZE];

    /// Unpacks a value from a byte array.
    fn from_bytes(bytes: [u8; SIZE]) -> Self;

    /// Packs `self` so that logical bit `i` is at bit `7 - i % 8` of byte `i / 8`.
    ///
    /// This canonical form is what parents use to embed a nested type. It equals
    /// `into_bytes` for `be`/`le` types without `reverse` or `bit_traversal = "back"`,
    /// which is what the default implementation assumes.
    #[doc(hidden)]
    fn __into_logical(self) -> [u8; SIZE] {
        self.into_bytes()
    }

    /// Inverse of [`Bitfields::__into_logical`].
    #[doc(hidden)]
    fn __from_logical(bytes: [u8; SIZE]) -> Self {
        Self::from_bytes(bytes)
    }
}

/// A type that can be read from (and checked against) byte slices.
///
/// Deriving this also generates `read_slice_<field>` / `write_slice_<field>` associated
/// functions, which only require the bytes that field touches.
pub trait BitfieldsSlice: Sized {
    /// Total amount of bytes the bitfields occupy.
    const BYTE_SIZE: usize;
    /// Total amount of bits the bitfields occupy.
    const BIT_SIZE: usize;

    /// Read-only view over a slice that is known to be long enough.
    type Checked<'a>;
    /// Read/write view over a slice that is known to be long enough.
    type CheckedMut<'a>;

    /// Unpacks a value from the first `BYTE_SIZE` bytes of `bytes`.
    ///
    /// # Errors
    /// If `bytes` is shorter than `BYTE_SIZE`.
    fn from_slice(bytes: &[u8]) -> Result<Self, BitfieldLengthError>;

    /// Checks `bytes` holds at least `BYTE_SIZE` bytes and returns a view with infallible
    /// `read_<field>` functions.
    ///
    /// # Errors
    /// If `bytes` is shorter than `BYTE_SIZE`.
    fn check_slice(bytes: &[u8]) -> Result<Self::Checked<'_>, BitfieldLengthError>;

    /// Checks `bytes` holds at least `BYTE_SIZE` bytes and returns a view with infallible
    /// `read_<field>` and `write_<field>` functions.
    ///
    /// # Errors
    /// If `bytes` is shorter than `BYTE_SIZE`.
    fn check_slice_mut(bytes: &mut [u8]) -> Result<Self::CheckedMut<'_>, BitfieldLengthError>;

    /// Unpacks a value from the front of `bytes`, removing the bytes it used.
    ///
    /// # Errors
    /// If `bytes` is shorter than `BYTE_SIZE`; `bytes` is left untouched.
    #[cfg(feature = "alloc")]
    fn take_from_vec(bytes: &mut alloc::vec::Vec<u8>) -> Result<Self, BitfieldLengthError> {
        let value = Self::from_slice(bytes)?;
        bytes.drain(..Self::BYTE_SIZE);
        Ok(value)
    }
}
