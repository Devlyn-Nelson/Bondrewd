//! TODO `START_HERE`
//! create the masked structures. this step is takes a resolver and gives
//! the final form that bondrewd-builder is made to create a `Resolved` type
//! which is meant to be a fully solved version of a struct or enum containing
//! all of the necessary information to build derive functions or access bits in
//! at runtime. it would also be assumed that the structure version shouldn't be
//! needed at runtime because it should only need to be come this far as fields
//! as they are needed.

use crate::solved::field::ResolverType;

/// # Definitions
/// - bitfield: the slice or array of bytes that contain the reduced or "bitfield" form. The bitfield would the the bytes returned by `Bitfield::into_bytes` or given as a argument in `Bitfield::from_bytes`.
/// - field-buffer: the that is use to transition between bitfield and rust form. When using `into_bytes` functions this would be the array returned by the rust type. for `from_bytes` into the array that will be given to the rust type.
struct Extractor {
    /// The index in the field-buffer for this set of bits to go into.
    field_buffer_byte_index: usize,
    /// The mask to use when applying extracted bits to field-buffer.
    field_buffer_bit_mask: Option<u8>,
    /// The index in the bitfield for this set of bits to go into.
    bitfield_byte_index: usize,
    /// The mask to use when applying extracted bits to bitfield.
    bitfield_bit_mask: Option<u8>,
}

pub struct MaskedField {
    rust_byte_size: u32,
    /// Contains an [`Extraction`]` for each grouping of bits within the bitfield.
    extractors: Vec<Extractor>,
    /// Determines the method to use for extraction.
    resolver: ResolverType,
}

impl MaskedField {
    // fn make_into_bytes(&self) -> TokenStream {}
}

pub fn make_into_bytes_standard_single() {}
pub fn make_into_bytes_standard_multi() {}
pub fn make_into_bytes_alternative_single() {}
pub fn make_into_bytes_alternative_multi() {}
pub fn make_into_bytes_nested_single() {}
pub fn make_into_bytes_nested_multi() {}
