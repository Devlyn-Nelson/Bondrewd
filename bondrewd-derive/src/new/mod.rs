mod proc;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::build::Endianness;

pub struct StructAttributes {
    endianess: Endianness,
}

pub struct FieldInfo {
    name: FieldAccessInfo,
    bits: BitFieldOperations,
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

pub enum Shift {
    /// Shift the structure bits left when outputting to bitfield.
    Left(u32),
    /// Shift the structure bits right when outputting to bitfield.
    Right(u32),
}

pub struct BitOperation {
    shift: Shift,
    bit_mask: u8,
    struct_index: usize,
    bitfield_index: usize,
}

pub struct BitFieldOperations {
    /// A list of all bit operations for the field.
    ranges: Vec<BitOperation>,
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
