use crate::new::FieldInfo;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub struct FieldQuotes {
    /// code for clearing the field from an existing byte array.
    clear: TokenStream,
    /// code for writing field to byte array.
    write: TokenStream,
    /// code for reading field from byte array.
    read: TokenStream,
}

impl FieldQuotes {
    /// `access` is a [`TokenStream`] for accessing the field from the field.
    pub fn new(f: &FieldInfo) -> FieldQuotes {
        let field_name = f.name.field_name();
        let field_name_bytes = format_ident!("{field_name}_bytes");
        let field_name_access = f.name.field_access();
        let mut read = quote! {};
        let mut clear = quote! {};
        let mut write = quote! {
            #field_name_bytes = #field_name_access.to_be_bytes();
        };
        for r in f.bits.ranges.iter() {
            let output_byte_index = r.bitfield_index;
            let mask = r.bit_mask;
            // neg mask to clear bits before applying the new bits
            let neg_mask = !mask;
            let input_byte_index = r.struct_index;
            // Make write quotes
            let write_shifted = match r.shift {
                crate::new::Shift::Left(left_shift) => {
                    quote! {#field_name_bytes [ #input_byte_index ] << #left_shift}
                }
                crate::new::Shift::Right(right_shift) => {
                    quote! {#field_name_bytes [ #input_byte_index ] >> #right_shift}
                }
            };
            write = quote! {
                #write
                output_byte_buffer[#output_byte_index] |= #write_shifted & #mask;
            };
            clear = quote! {
                #clear
                output_byte_buffer[#output_byte_index] &= #neg_mask;
            };
            // make read quotes
            let read_masked = quote! { output_byte_buffer[#output_byte_index] & #mask };
            let read_shifted = match r.shift {
                crate::new::Shift::Left(left_shift) => quote! {#read_masked << #left_shift},
                crate::new::Shift::Right(right_shift) => quote! {#read_masked >> #right_shift},
            };
            read = quote! {
                #read
                #field_name_bytes [ #input_byte_index ] |= #read_shifted;
            };
        }
        FieldQuotes { clear, write, read }
    }

    pub fn merge(&mut self, other: FieldQuotes) {
        let o_clear = &other.clear;
        let s_clear = &self.clear;
        self.clear = quote! {
            #s_clear
            #o_clear
        };
        let o_write = &other.write;
        let s_write = &self.write;
        self.write = quote! {
            #s_write
            #o_write
        };
        let o_read = &other.read;
        let s_read = &self.read;
        self.read = quote! {
            #s_read
            #o_read
        };
    }

    pub fn from_list(list: Vec<FieldQuotes>) -> FieldQuotes {
        let mut result = FieldQuotes {
            clear: quote! {},
            write: quote! {},
            read: quote! {},
        };
        for item in list {
            result.merge(item);
        }
        result
    }
}
