use std::range::Range;

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
    /// A list of each bit range for the field. each bit range should pertain to a single byte.
    ranges: Vec<Range<usize>>,
}

/// `access` is a [`TokenStream`] for accessing the field from the field.
pub fn make_write_code(f: &FieldAttributes, access: &TokenStream) -> TokenStream {
    let field_name = f.name.field_name();
    let bytes_func = f.get_into_bytes_function();
    let mut output = quote! {
        let #field_name = #access #bytes_func;
    };
    for r in &f.bits.ranges {
        let byte_index = r.start / 8;
        let start = r.start % 8;
        let end = r.end % 8;
        let mask = make_mask(start, end);
        let neg_mask = !mask;
        // TODO add actual writing from field to byte buffer, currently only clearing old
        // bits is done.
        output = quote! {
            #output
            output_byte_buffer &= #neg_mask;
        }
        // TODO use makes an bit info to write data. still need to figure out what
        // byte in the fields output array to write from.
    }
    todo!("output tokenstream for writing field to bytes");
}

/// `access` is a [`TokenStream`] for accessing the field from the field.
pub fn make_read_code(f: &FieldAttributes, access: &TokenStream) -> TokenStream {
    todo!("output tokenstream for reading field from bytes");
}

/// `start` should never be greater than `end`. `end` should be less than or equal to 7.
fn make_mask(start: usize, end: usize) -> u8 {
    debug_assert!(start < 8, "make_mask param `start` must be less than 8");
    debug_assert!(end < 8, "make_mask param `end` must be less than 8");
    // TODO this might need the `+ 1` removed if we use exclusive ranges.
    let bits = (end - start) + 1;
    let mut mask: u8 = 0;
    for _ in 0..bits {
        mask <<= 1;
        mask |= 1;
    }
    let shift = (8 - bits) - start;
    mask.wrapping_shl(shift as u32)
}
