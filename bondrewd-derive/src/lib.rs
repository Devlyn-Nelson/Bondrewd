extern crate proc_macro;

mod enums;
use enums::parse::EnumInfo;
mod structs;
use structs::common::StructInfo;
use structs::from_bytes::create_from_bytes_field_quotes;
use structs::into_bytes::create_into_bytes_field_quotes;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

/// # Attributes: 
/// * `bit_length = {BITS}` - define the total amount of bits to use when packed.
/// 
/// # Tasks ((- = in-progress), (T = needs testing), (X = Done), (* = Partial))
/// Structs
/// [T] - read_direction ( the bit order is reversed with no runtime cost)
/// [T] - flip (flip the entire byte order with no runtime cost)
/// [T] - Little Endian primitives
///     [T] - Impl into_bytes.
///     [T] - Impl peek_{field} and peek_slice_{field} functions.
///     [T] - Impl from_bytes.
/// [T] - Big Endian primitives
///     [T] - Impl into_bytes.
///     [T] - Impl peek_{field} and peek_slice_{field} functions.
///     [T] - Impl from_bytes.
/// [T] - Struct
///     [T] - Impl into_bytes.
///     [T] - Impl peek_{field} and peek_slice_{field} functions.
///     [T] - Impl from_bytes.
/// [T] - Enum
///     [T] - Impl into_bytes.
///     [T] - Impl peek_{field} and peek_slice_{field} functions.
///     [T] - Impl from_bytes.
/// [T] - Element Arrays
///     [T] - Impl into_bytes.
///     [T] - Impl peek_{field} and peek_slice_{field} functions.
///     [T] - Impl from_bytes.
/// [T] - Block Arrays
///     [T] - Impl into_bytes.
///     [T] - Impl peek_{field} and peek_slice_{field} functions.
///     [T] - Impl from_bytes.
/// [T] - bit size enforcement as an option to ensure proper struct sizing
///     [T] - full bytes attribute (BIT_SIZE % 8 == 0)
///     [T] - total bit/bytes length enforcement by a specified amount of
///             bits or bytes.
/// * primitives should exclude usize and isize due to ambiguous sizing
#[proc_macro_derive(
    Bitfields,
    attributes(
        bondrewd,
    )
)]
pub fn derive_smart_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // parse the input into a StructInfo which contains all the information we
    // along with some helpful structures to generate our Bitfield code.
    let struct_info = match StructInfo::parse(&input) {
        Ok(parsed_struct) => parsed_struct,
        Err(err) => {
            return TokenStream::from(err.to_compile_error());
        }
    };
    // get the struct size and name so we can use them in a quote.
    let struct_size = struct_info.total_bytes();
    let struct_name = format_ident!("{}", struct_info.name);

    
    // get a list of all fields from_bytes logic which gets there bytes from an array called
    // input_byte_buffer.
    let slice_fns: bool;
    #[cfg(not(feature = "slice_fns"))]
    {
        slice_fns = false;
    }
    #[cfg(feature = "slice_fns")]
    {
        slice_fns = true;
    }
    // get a list of all fields into_bytes logic which puts there bytes into an array called
    // output_byte_buffer.
    let fields_into_bytes = match create_into_bytes_field_quotes(&struct_info, slice_fns) {
        Ok(ftb) => ftb,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };
    let fields_from_bytes = match create_from_bytes_field_quotes(&struct_info, slice_fns) {
        Ok(ffb) => ffb,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };
    // combine all of the into_bytes quotes separated by newlines
    let into_bytes_quote = fields_into_bytes.into_bytes_fn;
    let mut set_quotes = fields_into_bytes.set_field_fns;

    if let Some(set_slice_quote) = fields_into_bytes.set_slice_field_fns {
        set_quotes = quote! {
            #set_quotes
            #set_slice_quote
        }
    }

    let from_bytes_quote = fields_from_bytes.from_bytes_fn;
    let mut peek_quotes = fields_from_bytes.peek_field_fns;

    if let Some(peek_slice_quote) = fields_from_bytes.peek_slice_field_fns {
        peek_quotes = quote! {
            #peek_quotes
            #peek_slice_quote
        }
    }

    let getter_setters_quotes = quote! {
        impl #struct_name {
            #peek_quotes
            #set_quotes
        }
    };

    // get the bit size of the entire set of fields to fill in trait requirement.
    let bit_size = struct_info.total_bits();

    // put it all together.
    // to_bytes_quote will put all of the fields in self into a array called output_byte_buffer.
    // so for into_bytes all we need is the fn declaration, the output_byte_buffer, and to return
    // that buffer.
    // from_bytes is essentially the same minus a variable because input_byte_buffer is the input.
    // slap peek quotes inside a impl block at the end and we good to go
    let to_bytes_quote = quote! {
        impl Bitfields<#struct_size> for #struct_name {
            const BIT_SIZE: usize = #bit_size;
            #into_bytes_quote
            #from_bytes_quote
        }
        #getter_setters_quotes
    };

    TokenStream::from(to_bytes_quote)
}

/// # Tasks ((- = in-progress), (T = needs testing), (X = Done), (* = Partial))
/// Enum
/// [X] - from_primitive.
/// [X] - into_primitive.
/// [X] - Invalid flag (Invalid values will be dropped an a generic no field
///                         variant will be used).
/// [X] - Invalid catch (stores the actual primitive in a 1 field Variant).
/// [ ] - types other than u8.
#[proc_macro_derive(BitfieldEnum, attributes(bondrewd_enum))]
pub fn derive_bondrewd_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_info = match EnumInfo::parse(&input) {
        Ok(parsed_enum) => parsed_enum,
        Err(err) => {
            return TokenStream::from(err.to_compile_error());
        }
    };
    let into = enums::into_bytes::generate_into_bytes(&enum_info);
    let from = enums::from_bytes::generate_from_bytes(&enum_info);
    let enum_name = enum_info.name;
    let primitive = enum_info.primitive;
    TokenStream::from(quote! {
        impl bondrewd::BitfieldEnum for #enum_name {
            type Primitive = #primitive;
            #into
            #from
        }
    })
}
