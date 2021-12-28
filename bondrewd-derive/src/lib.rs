//! Fast and easy bitfield proc macro
//!
//! Provides a proc macro for compressing a data structure with data which can be expressed with bit
//! lengths that are not a power of Two.
//! # Derive Generated Functions:
//! - Conversion between a sized u8 array and the rust structure you define.
//! - Peek and Set functions that allow the field to be accessed or overwritten within a sized u8 array.
//!
//! For example we can define a data structure with 5 total bytes as:
//! - a field named one will be the first 3 bits.
//! - a field named two will be the next 19 bits.
//! - a field named six will be the next 14 bits.
//! - a field named four will be the next 4 bits.
//!
//!
//! ```
//! // Users code
//! use bondrewd::*;
//! #[derive(Bitfields)]
//! #[bondrewd(default_endianness = "be")]
//! struct SimpleExample {
//!     #[bondrewd(bit_length = 3)]
//!     one: u8,
//!     #[bondrewd(bit_length = 19)]
//!     two: u32,
//!     #[bondrewd(bit_length = 14)]
//!     six: u16,
//!     #[bondrewd(bit_length = 4)]
//!     four: u8,
//! }
//! ```
//! ```compile_fail
//! // Generated Code
//! impl Bitfields<5usize> for SimpleExample {
//!     const BIT_SIZE: usize = 40usize;
//!     fn into_bytes(self) -> [u8; 5usize] { .. }
//!     fn from_bytes([u8; 5usize]) -> Self { .. }
//! }
//! impl SimpleExample {
//!     pub fn peek_one(&[u8; 5usize]) -> u8 { .. }
//!     pub fn peek_two(&[u8; 5usize]) -> u32 { .. }
//!     pub fn peek_six(&[u8; 5usize]) -> u16 { .. }
//!     pub fn peek_four(&[u8; 5usize]) -> u8 { .. }
//!     pub fn set_one(&mut [u8; 5usize], u8) { .. }
//!     pub fn set_two(&mut [u8; 5usize], u32) { .. }
//!     pub fn set_six(&mut [u8; 5usize], u16) { .. }
//!     pub fn set_four(&mut [u8; 5usize], u8) { .. }
//! }
//! ```
//! # Supported Field Types
//! * All primitives other than usize and isize (i believe ambiguous sizing is bad for this type of work).
//! * Enums which implement the BitfieldEnum trait in bondrewd.
//! * Structs which implement the Bitfield trait in bondrewd.
//!
//! # Struct Attributes
//! * `default_endianness = {"le" or "be"}` describes a default endianness for primitive fields.
//! * `read_from = {"msb0" or "lsb0"}` defines bit positioning. which end of the byte array to start at.
//! * `enforce_bytes = {BYTES}` defines a required resulting BIT_SIZE divided by 8 of the structure in condensed form.
//! * `enforce_bits = {BYTES}` defines a required resulting BIT_SIZE of the structure in condensed form.
//! * `enforce_full_bytes` defines that the resulting BIT_SIZE is required to be a multiple of 8.
//! * `reverse` defines that the entire byte array should be reversed before reading. no runtime cost.
//!
//! # Field Attributes
//! * `bit_length = {BITS}` define the total amount of bits to use when condensed.
//! * `byte_length = {BYTES}` define the total amount of bytes to use when condensed.
//! * `endianness = {"le" or "be"}` define per field endianess.
//! * `block_bit_length = {BITS}` describes a bit length for the entire array dropping lower indexes first. (default array type)
//! * `block_byte_length = {BYTES}` describes a byte length for the entire array dropping lower indexes first. (default array type)
//! * `element_bit_length = {BITS}` describes a bit length for each element of an array.
//! * `element_byte_length = {BYTES}` describes a byte length for each element of an array.
//! * `enum_primitive = "u8"` defines the size of the enum. the BitfieldEnum currently only supports u8.
//! * `struct_size = {SIZE}` defines the field as a struct which implements the Bitfield trait and the BYTE_SIZE const defined in said trait.
//! * `reserve` defines that this field should be ignored in from and into bytes functions.
//! * /!Untested!\ `bits = "RANGE"` - define the bit indexes yourself rather than let the proc macro figure
//! it out. using a rust range in quotes.
//!
//! # Crate Features:
//! * `slice_fns` generates slice functions:
//!     * `fn peek_slice_{field}(&[u8]) -> Result<{field_type}, BondrewdSliceError> {}`
//!     * `fn set_slice_{field}(&mut [u8], {field_type}) -> Result<(), BondrewdSliceError> {}`
//! * `hex_fns` provided from/into hex functions like from/into bytes. the hex inputs/outputs are \[u8;N\]
//! where N is double the calculated bondrewd STRUCT_SIZE. hex encoding and decoding is based off the
//! [hex](https://crates.io/crates/hex) crate's from/into slice functions but with statically sized
//! arrays so we could eliminate sizing errors.
//!
//! ### Full Example Generated code
//! ```
//! use bondrewd::*;
//! struct SimpleFull {
//!     one: u8,
//!     two: u32,
//!     six: u16,
//!     four: u8,
//! }
//! impl Bitfields<5usize> for SimpleFull {
//!     const BIT_SIZE: usize = 40usize;
//!     fn into_bytes(self) -> [u8; 5usize] {
//!         let mut output_byte_buffer: [u8; 5usize] = [0u8; 5usize];
//!         output_byte_buffer[0usize] |= ((self.one as u8) << 5usize) & 224u8;
//!         let two_bytes = (self.two.rotate_left(2u32)).to_be_bytes();
//!         output_byte_buffer[0usize] |= two_bytes[1usize] & 31u8;
//!         output_byte_buffer[1usize] |= two_bytes[2usize];
//!         output_byte_buffer[2usize] |= two_bytes[3usize] & 252u8;
//!         let six_bytes = (self.six.rotate_right(4u32)).to_be_bytes();
//!         output_byte_buffer[2usize] |= six_bytes[0usize] & 3u8;
//!         output_byte_buffer[3usize] |= six_bytes[1usize];
//!         output_byte_buffer[4usize] |= six_bytes[0] & 240u8;
//!         output_byte_buffer[4usize] |= ((self.four as u8) << 0usize) & 15u8;
//!         output_byte_buffer
//!     }
//!     fn from_bytes(mut input_byte_buffer: [u8; 5usize]) -> Self {
//!         let one = Self::peek_one(&input_byte_buffer);
//!         let two = Self::peek_two(&input_byte_buffer);
//!         let six = Self::peek_six(&input_byte_buffer);
//!         let four = Self::peek_four(&input_byte_buffer);
//!         Self {
//!             one,
//!             two,
//!             six,
//!             four,
//!         }
//!     }
//! }
//! impl SimpleFull {
//!     #[inline]
//!     pub fn peek_one(input_byte_buffer: &[u8; 5usize]) -> u8 {
//!         ((input_byte_buffer[0usize] & 224u8) >> 5usize) as u8
//!     }
//!     #[inline]
//!     pub fn peek_two(input_byte_buffer: &[u8; 5usize]) -> u32 {
//!         u32::from_be_bytes({
//!             let mut two_bytes: [u8; 4usize] = [0u8; 4usize];
//!             two_bytes[1usize] = input_byte_buffer[0usize] & 31u8;
//!             two_bytes[2usize] |= input_byte_buffer[1usize];
//!             two_bytes[3usize] |= input_byte_buffer[2usize] & 252u8;
//!             two_bytes
//!         })
//!         .rotate_right(2u32)
//!     }
//!     #[inline]
//!     pub fn peek_six(input_byte_buffer: &[u8; 5usize]) -> u16 {
//!         u16::from_be_bytes({
//!             let mut six_bytes: [u8; 2usize] = [0u8; 2usize];
//!             six_bytes[0usize] = input_byte_buffer[2usize] & 3u8;
//!             six_bytes[1usize] |= input_byte_buffer[3usize];
//!             six_bytes[0] |= input_byte_buffer[4usize] & 240u8;
//!             six_bytes
//!         })
//!         .rotate_left(4u32)
//!     }
//!     #[inline]
//!     pub fn peek_four(input_byte_buffer: &[u8; 5usize]) -> u8 {
//!         ((input_byte_buffer[4usize] & 15u8) >> 0usize) as u8
//!     }
//!     #[inline]
//!     pub fn set_one(output_byte_buffer: &mut [u8; 5usize], one: u8) {
//!         output_byte_buffer[0usize] &= 31u8;
//!         output_byte_buffer[0usize] |= ((one as u8) << 5usize) & 224u8;
//!     }
//!     #[inline]
//!     pub fn set_two(output_byte_buffer: &mut [u8; 5usize], two: u32) {
//!         output_byte_buffer[0usize] &= 224u8;
//!         output_byte_buffer[2usize] &= 3u8;
//!         let two_bytes = (two.rotate_left(2u32)).to_be_bytes();
//!         output_byte_buffer[0usize] |= two_bytes[1usize] & 31u8;
//!         output_byte_buffer[1usize] |= two_bytes[2usize];
//!         output_byte_buffer[2usize] |= two_bytes[3usize] & 252u8;
//!     }
//!     #[inline]
//!     pub fn set_six(output_byte_buffer: &mut [u8; 5usize], six: u16) {
//!         output_byte_buffer[2usize] &= 252u8;
//!         output_byte_buffer[3usize] = 0u8;
//!         output_byte_buffer[4usize] &= 15u8;
//!         let six_bytes = (six.rotate_right(4u32)).to_be_bytes();
//!         output_byte_buffer[2usize] |= six_bytes[0usize] & 3u8;
//!         output_byte_buffer[3usize] |= six_bytes[1usize];
//!         output_byte_buffer[4usize] |= six_bytes[0] & 240u8;
//!     }
//!     #[inline]
//!     pub fn set_four(output_byte_buffer: &mut [u8; 5usize], four: u8) {
//!         output_byte_buffer[4usize] &= 240u8;
//!         output_byte_buffer[4usize] |= ((four as u8) << 0usize) & 15u8;
//!     }
//! }
//! ```
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

/// Generates an implementation of the bondrewd::Bitfield trait, as well as peek and set functions for direct
/// sized u8 arrays access.
///
/// # Struct Derive Tasks
/// - [x] Little Endian primitives
///     - [x] Impl peek_{field} and set_{field} functions.
///     - [x] Impl into_bytes.
///     - [x] Impl from_bytes.
///     - [x] Impl peek_slice_{field} and set_slice_{field} functions.
/// - [x] Big Endian primitives
///     - [x] Impl peek_{field} and set_{field} functions.
///     - [x] Impl into_bytes.
///     - [x] Impl from_bytes.
///     - [x] Impl peek_slice_{field} and set_slice_{field} functions.
/// - [x] Struct
///     - [x] Impl peek_{field} and set_{field} functions.
///     - [x] Impl into_bytes.
///     - [x] Impl from_bytes.
///     - [x] Impl peek_slice_{field} and set_slice_{field} functions.
/// - [x] Enum
///     - [x] Impl peek_{field} and set_{field} functions.
///     - [x] Impl into_bytes.
///     - [x] Impl from_bytes.
///     - [x] Impl peek_slice_{field} and set_slice_{field} functions.
/// - [x] Element Arrays
///     - [x] Impl peek_{field} and set_{field} functions.
///     - [x] Impl into_bytes.
///     - [x] Impl from_bytes.
///     - [x] Impl peek_slice_{field} and set_slice_{field} functions.
/// - [x] Block Arrays
///     - [x] Impl peek_{field} and set_{field} functions.
///     - [x] Impl into_bytes.
///     - [x] Impl from_bytes.
///     - [x] Impl peek_slice_{field} and set_slice_{field} functions.
///     - [x] use this array type automatically if no attributes are provided.
/// - [x] bit size enforcement as an option to ensure proper struct sizing
///     - [x] full bytes attribute (BIT_SIZE % 8 == 0)
///     - [x] total bit/bytes length enforcement by a specified amount of
///             bits or bytes.
/// - [x] read_direction ( the bit order is reversed with no runtime cost)
/// - [x] flip (flip the entire byte order with no runtime cost)
/// - [x] reserve fields which don't get read or written in into or from bytes functions.
/// * primitives should exclude usize and isize due to ambiguous sizing
#[proc_macro_derive(Bitfields, attributes(bondrewd,))]
pub fn derive_bitfields(input: TokenStream) -> TokenStream {
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

    let setters: bool;
    #[cfg(not(feature = "setters"))]
    {setters = false;}
    #[cfg(feature = "setters")]
    {setters = true;}
    let setters_quote = if setters {
        match structs::struct_fns::create_into_bytes_field_quotes(&struct_info) {
            Ok(parsed_struct) => parsed_struct,
            Err(err) => {
                return TokenStream::from(err.to_compile_error());
            }
        }
    }else{
        quote!{}
    };

    let getter_setters_quotes = quote! {
        impl #struct_name {
            #peek_quotes
            #set_quotes
            #setters_quote
        }
    };
    let hex;
    #[cfg(feature = "hex_fns")]
    {
        hex = true;
    }
    #[cfg(not(feature = "hex_fns"))]
    {
        hex = false;
    }
    let hex_size = struct_size * 2;
    let hex_fns_quote = if hex {
        quote! {
            impl BitfieldHex<#hex_size> for #struct_name {
                fn from_hex(hex: [u8;#hex_size]) -> Result<Self, BitfieldHexError> {
                    let bytes: [u8; #struct_size] = [0;#struct_size];
                    let mut bytes: [u8; Self::BYTE_SIZE] = [0;Self::BYTE_SIZE];
                    for i in 0usize..#struct_size {
                        let index = i * 2;
                        let index2 = index + 1;
                        let decode_nibble = |c, c_i| match c {
                            b'A'..=b'F' => Ok(c - b'A' + 10u8),
                            b'a'..=b'f' => Ok(c - b'a' + 10u8),
                            b'0'..=b'9' => Ok(c - b'0'),
                            _ => return Err(BitfieldHexError(
                                c as char,
                                c_i,
                            )),
                        };
                        bytes[i] = ((decode_nibble(hex[index], index)? & 0b00001111) << 4) | decode_nibble(hex[index2], index2)?;
                    }
                    Ok(Self::from_bytes(bytes))

                }

                fn into_hex_upper(self) -> [u8;#hex_size] {
                    let bytes = self.into_bytes();
                    let mut output: [u8;#hex_size] = [0; #hex_size];
                    for (i, byte) in (0..#hex_size).step_by(2).zip(bytes) {
                        output[i] = (Self::UPPERS[((byte & 0b11110000) >> 4) as usize]);
                        output[i + 1] = (Self::UPPERS[(byte & 0b00001111) as usize]);
                    }
                    output
                }

                fn into_hex_lower(self) -> [u8;#hex_size] {
                    let bytes = self.into_bytes();
                    let mut output: [u8;#hex_size] = [0; #hex_size];
                    for (i, byte) in (0..#hex_size).step_by(2).zip(bytes) {
                        output[i] = (Self::LOWERS[((byte & 0b11110000) >> 4) as usize]);
                        output[i + 1] = (Self::LOWERS[(byte & 0b00001111) as usize]);
                    }
                    output
                }
            }
        }
    } else {
        quote! {}
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
        #hex_fns_quote
    };

    TokenStream::from(to_bytes_quote)
}

/// Generates an implementation of bondrewd::BitfieldEnum trait.
///
/// # Enum Derive Tasks
/// - [x] from_primitive.
/// - [x] into_primitive.
/// - [x] Invalid flag (Invalid values will be dropped an a generic no field
///                         variant will be used).
/// - [x] Invalid catch (stores the actual primitive in a 1 field Variant).
/// - [ ] types other than u8.
/// - [x] Support `u8` literals for Enum Variants
/// - [x] Support for impl of `std::cmp::PartialEq` for the given primitive (currently only u8)
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
    let partial_eq = enums::partial_eq::generate_partial_eq(&enum_info);
    let enum_name = enum_info.name;
    let primitive = enum_info.primitive;
    TokenStream::from(quote! {
        impl bondrewd::BitfieldEnum for #enum_name {
            type Primitive = #primitive;
            #into
            #from
        }

        #partial_eq
    })
}
