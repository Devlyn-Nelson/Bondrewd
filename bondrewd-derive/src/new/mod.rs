mod write;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::build::Endianness;

pub struct StructAttributes {
    endianess: Endianness,
}

pub struct FieldInfo {
    name: FieldAccessInfo,
    bits: FieldBits,
    little_endian: bool,
}

impl FieldInfo {
    pub fn is_little_endian(&self) -> bool {
        self.little_endian
    }
}

pub enum FieldAccessObjectKind {
    Struct,
    Enum,
}

impl FieldAccessObjectKind {
    pub fn is_struct(&self) -> bool {
        matches!(self, FieldAccessObjectKind::Struct)
    }
    pub fn is_enum(&self) -> bool {
        matches!(self, FieldAccessObjectKind::Enum)
    }
}

pub enum FieldAccessFieldKind {
    Named(String),
    Indexed(usize),
}

impl FieldAccessFieldKind {
    pub fn is_named(&self) -> bool {
        matches!(self, FieldAccessFieldKind::Named(_))
    }
    pub fn is_indexed(&self) -> bool {
        matches!(self, FieldAccessFieldKind::Indexed(_))
    }
}

pub struct FieldAccessInfo {
    obj_kind: FieldAccessObjectKind,
    field_kind: FieldAccessFieldKind,
}

impl FieldAccessInfo {
    pub fn field_name(&self) -> String {
        match &self.field_kind {
            FieldAccessFieldKind::Named(name) => name.clone(),
            FieldAccessFieldKind::Indexed(index) => format!("field_{index}"),
        }
    }
    pub fn field_access(&self) -> TokenStream {
        let ident = match &self.field_kind {
            FieldAccessFieldKind::Named(name) => {
                let ident = format_ident!("{name}");
                quote! { #ident }
            }
            FieldAccessFieldKind::Indexed(index) => {
                let ident = format_ident!("field_{index}");
                quote! { #ident }
            }
        };
        if self.obj_kind.is_struct() {
            quote! {self.#ident}
        } else {
            quote! {#ident}
        }
    }
}

pub struct BitOperation {
    /// total count of bits in this operation.
    bit_length: usize,
    /// the byte index in the field's byte array, rust side.
    field_index: usize,
    /// the start bit within the byte accessed by `field_index`, rust side.
    field_start: usize,
    /// the index of the byte in the bit field's byte array, bitfield side.
    bit_field_index: usize,
    /// the start bit within the byte accessed by `bit_field_index`, bitfield side.
    bit_field_start: usize,
}

pub struct FieldBits {
    /// A list of all bit operations for the field.
    ranges: Vec<BitOperation>,
}

impl FieldBits {
    pub fn count(&self) -> usize {
        let mut c = 0;
        for r in &self.ranges {
            c += r.bit_length;
        }
        c
    }
}

/// `access` is a [`TokenStream`] for accessing the field from the field.
pub fn make_read_code(f: &FieldInfo, access: &TokenStream) -> TokenStream {
    todo!("output token stream for reading field from bytes");
}

pub struct MaskAndShift {
    /// mask to get only the relevant bits.
    pub mask: u8,
    /// left shift amount.
    pub shift: u32,
}

impl MaskAndShift {
    /// `start` should never be greater than `end`. `end` should be less than or equal to 7.
    pub fn from_start_end(start: usize, end: usize) -> Self {
        debug_assert!(start < 8, "make_mask param `start` must be less than 8");
        debug_assert!(end < 8, "make_mask param `end` must be less than 8");
        // NOTE this might need the `+ 1` removed if we use exclusive ranges.
        let bits = (end - start) + 1;
        let mut mask: u8 = 0;
        for _ in 0..bits {
            mask <<= 1;
            mask |= 1;
        }
        let shift = ((8 - bits) - start) as u32;
        let mask = mask.wrapping_shl(shift);
        Self { mask, shift }
    }
    pub fn split(self) -> (u8, u32) {
        (self.mask, self.shift)
    }
}
