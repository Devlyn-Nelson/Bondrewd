use crate::new::{FieldInfo, MaskAndShift};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub struct FieldWriteQuote {
    /// code for clearing the field from an existing byte array.
    clear: TokenStream,
    /// code for writing field to byte array.
    write: TokenStream,
}

impl FieldWriteQuote {
    /// `access` is a [`TokenStream`] for accessing the field from the field.
    pub fn new(f: &FieldInfo) -> FieldWriteQuote {
        let field_name = f.name.field_name();
        let field_name_bytes = format_ident!("{field_name}_bytes");
        let field_name_access = f.name.field_access();
        let mut clear = quote! {
        };
        let mut write = quote! {
            #field_name_bytes = #field_name_access.to_be_bytes();
        };
        let field_bits = f.bits.count();
        for (i, r) in f.bits.ranges.iter().rev().enumerate() {
            let output_byte_index = r.bit_field_index;
            let output_start = r.bit_field_start;
            let output_end = r.bit_field_start + r.bit_length - 1;
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
            let input_byte_index = r.field_index;
            write = quote! {
                #write
                output_byte_buffer[#output_byte_index] |= #field_name_bytes [ #input_byte_index ] & #mask;
            };

            if f.little_endian {
                write = quote! {};
            }
        }
        todo!(
            "make logic to place bits in proper place. it can be 1 or 2 operations depending on how the input bits \
            and output bits are aligned."
        );
        FieldWriteQuote { clear, write }
    }
}
