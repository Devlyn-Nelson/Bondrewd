use std::range::RangeInclusive;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::build::Endianness;

pub struct StructAttributes {
    endianess: Endianness,
}

pub struct FieldAttributes {
    name: NameOrIndex,
    bits: FieldBits,
    little_endian: bool,
}

impl FieldAttributes {
    pub fn get_into_bytes_function(&self) -> TokenStream {
        if self.little_endian {
            quote! { .to_le_bytes() }
        } else {
            quote! { .to_be_bytes() }
        }
    }
    pub fn is_little_endian(&self) -> bool {
        self.little_endian
    }
}

pub enum NameOrIndex {
    Name(String),
    Index(usize),
}

impl NameOrIndex {
    pub fn field_name(&self) -> String {
        match self {
            NameOrIndex::Name(name) => name.clone(),
            NameOrIndex::Index(index) => format!("field_{index}"),
        }
    }
    pub fn field_access(&self) -> Ident {
        match self {
            NameOrIndex::Name(name) => format_ident!("self.{name}"),
            NameOrIndex::Index(index) => format_ident!("field_{index}"),
        }
    }
}

pub struct FieldBits {
    /// A list of each bit range for the field. each bit range should pertain to a single byte. the first
    /// element SHALL be the lowest byte index, and the last element SHALL be the highest byte index.
    ranges: Vec<RangeInclusive<usize>>,
}

impl FieldBits {
    pub fn count(&self) -> usize {
        let mut c = 0;
        for r in &self.ranges {
            c += r.last - r.start + 1;
        }
        c
    }
}

/// `access` is a [`TokenStream`] for accessing the field from the field.
pub fn make_write_code(f: &FieldAttributes, access: &TokenStream) -> TokenStream {
    let field_name = f.name.field_name();
    let field_name_bytes = format_ident!("{field_name}_bytes");
    let bytes_func = f.get_into_bytes_function();
    let mut output = quote! {
        let #field_name_bytes = #access #bytes_func;
    };
    let effected_bytes = f.bits.ranges.len();
    for (i, r) in f.bits.ranges.iter().rev().enumerate() {
        let output_byte_index = r.start / 8;
        let start = r.start % 8;
        let end = r.last % 8;
        let (mask, left_shift) = MaskAndShift::from_start_end(start, end).split();
        // neg mask to clear bits before applying the new bits
        let neg_mask = !mask;
        // 1111 1111 0000 0011 field bytes
        // 0011 1111 1111 0000 be/ale
        // 1111 1100 0000 1111 le
        output = quote! {
            #output
            output_byte_buffer[#output_byte_index] &= #neg_mask;
        };
        todo!(
            "make logic to bit bits in proper place. it can be 1 or 2 operations depending on how the input bits \
            and output bits are aligned."
        );
        output = quote! {
            output_byte_buffer[#output_byte_index] |= #field_name_bytes [ #i ] & #mask;
        };
    }
    todo!("output tokenstream for writing field to bytes");
}

/// `access` is a [`TokenStream`] for accessing the field from the field.
pub fn make_read_code(f: &FieldAttributes, access: &TokenStream) -> TokenStream {
    todo!("output tokenstream for reading field from bytes");
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
