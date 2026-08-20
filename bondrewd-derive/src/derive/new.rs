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
        let mut clear = quote! {};
        let mut write = quote! {};
        let mut field_bits = f.bits.count();
        for (i, r) in f.bits.ranges.iter().rev().enumerate() {
            let output_byte_index = r.start / 8;
            let output_start = r.start % 8;
            let output_end = r.last % 8;
            let (mask, left_shift) = MaskAndShift::from_start_end(output_start, output_end).split();
            // neg mask to clear bits before applying the new bits
            let neg_mask = !mask;
            clear = quote! {
                #clear
                output_byte_buffer[#output_byte_index] &= #neg_mask;
            };
            // TODO we need to make the code that puts the bits into the output.
            // also note that we shouldn't need to use `to_be_bytes` because
            // it doesn't actually matter what the bondrewd buffer looks like
            // as long as the endianess in the output is correct. and since it
            // is more likely that little endian is used, we use that as the
            // default
            //
            // 1111 1111 0000 0011 field bytes (little endian)
            // 0011 1111 1111 0000 be/ale
            // 1111 1100 0000 1111 le

            // the amount of bits the first operation will pull from the input.
            // this is determined by the amount of bits available in the input
            // in a single byte. for example, if the output wants 4 bits for
            // the current output byte but the input would cross a bytes boundry
            // then the system need to make this 2 operations (one for each input byte
            // going into the 4 bits of the output byte)
            let total_output_bits = (output_end - output_start) + 1;
            let first_op_bits = field_bits % 8;
            let first_op_bits =
                if first_op_bits == 0 { 8 } else { first_op_bits }.min(total_output_bits);

            if first_op_bits == total_output_bits {
                // only 1 operation to write the field fragment to the output byte array
            } else {
                // 2 operations to write the field fragment to the output byte array
            }

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
