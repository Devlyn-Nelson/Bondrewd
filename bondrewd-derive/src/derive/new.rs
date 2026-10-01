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
    pub fn is_little_endian(&self) -> bool {
        self.little_endian
    }
}

pub enum NameOrIndex {
    /// The field is from a struct and is of course named.
    NameSelf(String),
    /// The field is from an enum and is named.
    Name(String),
    /// The field is from an enum and is not named therefor.
    Index(usize),
}

impl NameOrIndex {
    pub fn field_name(&self) -> String {
        match self {
            NameOrIndex::Name(name) => name.clone(),
            NameOrIndex::Index(index) => format!("field_{index}"),
            NameOrIndex::NameSelf(name) => name.clone(),
        }
    }
    pub fn field_access(&self) -> Ident {
        match self {
            NameOrIndex::Name(name) => format_ident!("self.{name}"),
            NameOrIndex::Index(index) => format_ident!("field_{index}"),
            NameOrIndex::NameSelf(name) => format_ident!("{name}"),
        }
    }
}

pub struct BitOperation {
    struct_field_bit_start: usize,
    struct_field_bit_length: usize,
    bit_field_bit_start: usize,
}

pub struct FieldBits {
    /// A list of all bit operations for the field.
    ranges: Vec<BitOperation>,
}

impl FieldBits {
    pub fn count(&self) -> usize {
        let mut c = 0;
        for r in &self.ranges {
            c += r.struct_field_bit_length;
        }
        c
    }
}

/// `access` is a [`TokenStream`] for accessing the field from the field.
pub fn make_read_code(f: &FieldAttributes, access: &TokenStream) -> TokenStream {
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

pub struct FieldWriteQuote {
    /// code for clearing the field from an existing byte array.
    clear: TokenStream,
    /// code for writing field to byte array.
    write: TokenStream,
}

impl FieldWriteQuote {
    /// `access` is a [`TokenStream`] for accessing the field from the field.
    pub fn new(f: &FieldAttributes) -> FieldWriteQuote {
        let field_name = f.name.field_name();
        let field_name_bytes = format_ident!("{field_name}_bytes");
        let field_name_access = f.name.field_access();
        let mut clear = quote! {
            #field_name_bytes = #field_name_access.to_be_bytes();
        };
        let mut write = match &f.name {
            NameOrIndex::NameSelf(name) => quote! {},
            NameOrIndex::Name(_) => quote! {},
            NameOrIndex::Index(_) => quote! {},
        };
        let field_bits = f.bits.count();
        for (i, r) in f.bits.ranges.iter().rev().enumerate() {
            let output_byte_index = r.bit_field_bit_start / 8;
            let output_start = r.bit_field_bit_start % 8;
            let output_end = (r.bit_field_bit_start + r.struct_field_bit_length) % 8;
            let (mask, left_shift) = MaskAndShift::from_start_end(output_start, output_end).split();
            // neg mask to clear bits before applying the new bits
            let neg_mask = !mask;
            clear = quote! {
                #clear
                output_byte_buffer[#output_byte_index] &= #neg_mask;
            };
            // TODO rotation of bits in field needs to happen. but one operation may
            // try to rotate the same bytes if not careful.
            // only 1 operation to write the field fragment to the output byte array
            let input_byte_index = output_end.div_ceil(8);
            write = quote! {
                #write
                output_byte_buffer[#output_byte_index] |= #field_name_bytes [ #input_byte_index ] & #mask;
            };

            if f.little_endian {
                write = quote! {};
            } else {
            }
        }
        todo!(
            "make logic to place bits in proper place. it can be 1 or 2 operations depending on how the input bits \
            and output bits are aligned."
        );
        FieldWriteQuote { clear, write }
    }
}
