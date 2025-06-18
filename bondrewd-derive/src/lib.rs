#![allow(unreachable_code, dead_code, unused_variables)]
//! Fast and easy bitfield proc macro
//!
//! Provides a proc macro used on Structures or Enums which provides functions for:
//! - outputting your native rust structure/enum as a statically sized byte array.
//! - re-creating your structure/enum from a statically sized byte array (no possible errors). `BitfieldsDyn`
//!     a `&[u8]` or `Vec<u8>` (possible length error).
//! - read/writing individual fields within a byte array (or slice with `BitfieldsDyn`) without
//!     re-writing unaffected bytes.
//!
//! # Derive
//! 
//! > `Bitfields` must be derived to derive any other bondrewd trait.
//! 
//! - Implements the [`Bitfields`](https://docs.rs/bondrewd/latest/bondrewd/trait.Bitfields.html) trait
//!     which offers from\into bytes functions that are non-fallible and convert the struct from/into sized
//!     u8 arrays.
//! - `read` and `write` functions that allow fields to be accessed or overwritten within a properly sized
//!     u8 array.
//! - See the [`Bitfields Derive`](Bitfields) page for more information about how to change details for how
//!     each field is handled (bit length, endianness, ..), or structure wide effects
//!     (starting bit position, default field endianness, ..).
//!
//! For example we can define a data structure with 7 total bytes as:
//! - A boolean field named one will be the first bit.
//! - A floating point field named two will be the next 32 bits. floats must be full sized currently.
//! - A signed integer field named three will be the next 14 bits.
//! - An unsigned integer field named four will be the next 6 bits.
//!
//! ```
//! // Users code
//! use bondrewd::*;
//! #[derive(Bitfields)]
//! #[bondrewd(endianness = "be")]
//! struct SimpleExample {
//!     // fields that are as expected do not require attributes.
//!     one: bool,
//!     two: f32,
//!     #[bondrewd(bit_length = 14)]
//!     three: u16,
//!     #[bondrewd(bit_length = 6)]
//!     four: i8,
//! }
//! ```
//! 
//! Generated function code and attributes omitted. If you want to see the Full Generated Source copy the code above
//! to a rust project and add the `dump` attribute on the struct for bondrewd (`#[bondrewd(endianness = "be", dump)]`), the generated
//! code will be output to `target/bondrewd_debug/{you-object-name}_code_gen_{trait}.rs`.
//! 
//! ```compile_fail
//! impl SimpleExample {
//!     /// Reads bit 0 within `input_byte_buffer`,
//!     /// getting the `one` field of a `SimpleExample` in bitfield form.
//!     pub fn read_one(input_byte_buffer: &[u8; 7usize]) -> bool { ... }
//!
//!     /// Reads bits 1 through 32 within `input_byte_buffer`,
//!     /// getting the `two` field of a `SimpleExample` in bitfield form.
//!     pub fn read_two(input_byte_buffer: &[u8; 7usize]) -> f32 { ... }
//!
//!     /// Reads bits 33 through 46 within `input_byte_buffer`,
//!     /// getting the `three` field of a `SimpleExample` in bitfield form.
//!     pub fn read_three(input_byte_buffer: &[u8; 7usize]) -> u16 { ... }
//!
//!     /// Reads bits 47 through 52 within `input_byte_buffer`,
//!     /// getting the `four` field of a `SimpleExample` in bitfield form.
//!     pub fn read_four(input_byte_buffer: &[u8; 7usize]) -> i8 { ... }
//!
//!     /// Writes to bit 0 within `output_byte_buffer`,
//!     /// setting the `one` field of a `SimpleExample` in bitfield form.
//!     pub fn write_one(output_byte_buffer: &mut [u8; 7usize], mut one: bool) { ... }
//!
//!     /// Writes to bits 1 through 32 within `output_byte_buffer`,
//!     /// setting the `two` field of a `SimpleExample` in bitfield form.
//!     pub fn write_two(output_byte_buffer: &mut [u8; 7usize], mut two: f32) { ... }
//!
//!     /// Writes to bits 33 through 46 within `output_byte_buffer`,
//!     /// setting the `three` field of a `SimpleExample` in bitfield form.
//!     pub fn write_three(output_byte_buffer: &mut [u8; 7usize], mut three: u16) { ... }
//!
//!     /// Writes to bits 47 through 52 within `output_byte_buffer`,
//!     /// setting the `four` field of a `SimpleExample` in bitfield form.
//!     pub fn write_four(output_byte_buffer: &mut [u8; 7usize], mut four: i8) { ... }
//! }
//! impl bondrewd::Bitfields<7usize> for SimpleExample {
//!     const BIT_SIZE: usize = 53usize;
//!     fn from_bytes(mut input_byte_buffer: [u8; 7usize]) -> Self { ... }
//!
//!     fn into_bytes(self) -> [u8; 7usize] { ... }
//! }
//! ```

use std::fmt::Display;

use build::field_set::GenericBuilder;
use proc_macro2::TokenStream;
use quote::quote;
use solved::field_set::Solved;
use syn::{parse_macro_input, DeriveInput};

mod build;
mod derive;
mod masked;
mod solved;

#[derive(Clone)]
pub(crate) struct SplitTokenStream {
    read: TokenStream,
    write: TokenStream,
}

impl SplitTokenStream {
    pub(crate) fn merge(self) -> TokenStream {
        let read = self.read;
        let write = self.write;
        quote! {
            #read
            #write
        }
    }
    pub(crate) fn merged(&self) -> TokenStream {
        let read = &self.read;
        let write = &self.write;
        quote! {
            #read
            #write
        }
    }
    pub fn insert(&mut self, other: Self) {
        let my_read = &mut self.read;
        let my_write = &mut self.write;
        let other_read = other.read;
        let other_write = other.write;
        *my_read = quote! {
            #other_read
            #my_read
        };
        *my_write = quote! {
            #other_write
            #my_write
        };
    }
    pub fn clear(&mut self) {
        self.read = TokenStream::default();
        self.write = TokenStream::default();
    }
}

impl Default for SplitTokenStream {
    fn default() -> Self {
        Self {
            read: TokenStream::new(),
            write: TokenStream::new(),
        }
    }
}

#[derive(Clone)]
pub(crate) enum GenerationFlavor {
    Standard {
        /// Functions that belong in `Bitfields` impl for object.
        trait_fns: SplitTokenStream,
        /// Functions that belong in impl for object.
        impl_fns: SplitTokenStream,
    },
    Dynamic {
        /// Functions that belong in `BitfieldsDyn` impl for object.
        trait_fns: SplitTokenStream,
        /// Functions that belong in impl for object.
        impl_fns: SplitTokenStream,
    },
    Slice {
        /// Functions that belong in `BitfieldsSlice` impl for object.
        trait_fns: SplitTokenStream,
        /// Functions that belong in impl for object.
        impl_fns: SplitTokenStream,
        /// Functions that belong in `BitfieldsSlice` impl for object.
        struct_fns: SplitTokenStream,
    },
    Hex {
        /// Functions that belong in `Bitfields` impl for object.
        trait_fns: TokenStream,
    },
    HexDynamic {
        /// Functions that belong in `Bitfields` impl for object.
        trait_fns: TokenStream,
    },
}

impl Display for GenerationFlavor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GenerationFlavor::Standard {
                trait_fns,
                impl_fns,
            } => "standard",
            GenerationFlavor::Dynamic {
                trait_fns,
                impl_fns,
            } => "dynamic",
            GenerationFlavor::Slice {
                trait_fns,
                impl_fns,
                struct_fns,
            } => "slice",
            GenerationFlavor::Hex { trait_fns } => "hex",
            GenerationFlavor::HexDynamic { trait_fns } => "hex_dynamic",
        };
        write!(f, "{s}")
    }
}

impl GenerationFlavor {
    pub(crate) fn clear(&mut self) {
        match self {
            GenerationFlavor::Standard {
                trait_fns,
                impl_fns,
            }
            | GenerationFlavor::Dynamic {
                trait_fns,
                impl_fns,
            } => {
                *trait_fns = SplitTokenStream::default();
                // *impl_fns = SplitTokenStream::default();
            }
            GenerationFlavor::Slice {
                trait_fns,
                impl_fns,
                struct_fns,
            } => {
                *trait_fns = SplitTokenStream::default();
                // *impl_fns = SplitTokenStream::default();
                // *struct_fns = SplitTokenStream::default();
            }
            GenerationFlavor::Hex { trait_fns } | GenerationFlavor::HexDynamic { trait_fns } => {
                *trait_fns = TokenStream::new();
            }
        }
    }
    pub(crate) fn new_from_type(&self) -> Self {
        match self {
            GenerationFlavor::Standard {
                trait_fns,
                impl_fns,
            } => Self::standard(),
            GenerationFlavor::Dynamic {
                trait_fns,
                impl_fns,
            } => Self::dynamic(),
            GenerationFlavor::Slice {
                trait_fns,
                impl_fns,
                struct_fns,
            } => Self::slice(),
            GenerationFlavor::Hex { trait_fns } => Self::hex(),
            GenerationFlavor::HexDynamic { trait_fns } => Self::hex_dynamic(),
        }
    }
    pub(crate) fn standard() -> Self {
        Self::Standard {
            trait_fns: SplitTokenStream::default(),
            impl_fns: SplitTokenStream::default(),
        }
    }
    pub(crate) fn dynamic() -> Self {
        Self::Dynamic {
            trait_fns: SplitTokenStream::default(),
            impl_fns: SplitTokenStream::default(),
        }
    }
    pub(crate) fn slice() -> Self {
        Self::Slice {
            trait_fns: SplitTokenStream::default(),
            impl_fns: SplitTokenStream::default(),
            struct_fns: SplitTokenStream::default(),
        }
    }
    pub(crate) fn hex() -> Self {
        Self::Hex {
            trait_fns: TokenStream::new(),
        }
    }
    pub(crate) fn hex_dynamic() -> Self {
        Self::HexDynamic {
            trait_fns: TokenStream::new(),
        }
    }
    pub(crate) fn merge(&mut self, other: &Self) {
        match (self, other) {
            (
                Self::Standard {
                    trait_fns,
                    impl_fns,
                },
                Self::Standard {
                    trait_fns: other_trait_fns,
                    impl_fns: other_impl_fns,
                },
            ) => {
                let read_trait_fns = &mut trait_fns.read;
                let other_read_trait_fns = &other_trait_fns.read;
                *read_trait_fns = quote! {
                    #read_trait_fns
                    #other_read_trait_fns
                };
                let read_impl_fns = &mut impl_fns.read;
                let other_read_impl_fns = &other_impl_fns.read;
                *read_impl_fns = quote! {
                    #read_impl_fns
                    #other_read_impl_fns
                };
                let write_trait_fns = &mut trait_fns.write;
                let other_write_trait_fns = &other_trait_fns.write;
                *write_trait_fns = quote! {
                    #write_trait_fns
                    #other_write_trait_fns
                };
                let write_impl_fns = &mut impl_fns.write;
                let other_write_impl_fns = &other_impl_fns.write;
                *write_impl_fns = quote! {
                    #write_impl_fns
                    #other_write_impl_fns
                };
            }
            (
                Self::Dynamic {
                    trait_fns,
                    impl_fns,
                },
                Self::Dynamic {
                    trait_fns: other_trait_fns,
                    impl_fns: other_impl_fns,
                },
            ) => {
                let read_trait_fns = &mut trait_fns.read;
                let other_read_trait_fns = &other_trait_fns.read;
                *read_trait_fns = quote! {
                    #read_trait_fns
                    #other_read_trait_fns
                };
                let read_impl_fns = &mut impl_fns.read;
                let other_read_impl_fns = &other_impl_fns.read;
                *read_impl_fns = quote! {
                    #read_impl_fns
                    #other_read_impl_fns
                };
                let write_trait_fns = &mut trait_fns.write;
                let other_write_trait_fns = &other_trait_fns.write;
                *write_trait_fns = quote! {
                    #write_trait_fns
                    #other_write_trait_fns
                };
                let write_impl_fns = &mut impl_fns.write;
                let other_write_impl_fns = &other_impl_fns.write;
                *write_impl_fns = quote! {
                    #write_impl_fns
                    #other_write_impl_fns
                };
            }
            (
                Self::Slice {
                    trait_fns,
                    impl_fns,
                    struct_fns,
                },
                Self::Slice {
                    trait_fns: other_trait_fns,
                    impl_fns: other_impl_fns,
                    struct_fns: other_struct_fns,
                },
            ) => {
                let read_trait_fns = &mut trait_fns.read;
                let other_read_trait_fns = &other_trait_fns.read;
                *read_trait_fns = quote! {
                    #read_trait_fns
                    #other_read_trait_fns
                };
                let read_impl_fns = &mut impl_fns.read;
                let other_read_impl_fns = &other_impl_fns.read;
                *read_impl_fns = quote! {
                    #read_impl_fns
                    #other_read_impl_fns
                };
                let read_struct_fns = &mut struct_fns.read;
                let other_read_struct_fns = &other_struct_fns.read;
                *read_struct_fns = quote! {
                    #read_struct_fns
                    #other_read_struct_fns
                };
                let write_trait_fns = &mut trait_fns.write;
                let other_write_trait_fns = &other_trait_fns.write;
                *write_trait_fns = quote! {
                    #write_trait_fns
                    #other_write_trait_fns
                };
                let write_impl_fns = &mut impl_fns.write;
                let other_write_impl_fns = &other_impl_fns.write;
                *write_impl_fns = quote! {
                    #write_impl_fns
                    #other_write_impl_fns
                };
                let write_struct_fns = &mut struct_fns.write;
                let other_write_struct_fns = &other_struct_fns.write;
                *write_struct_fns = quote! {
                    #write_struct_fns
                    #other_write_struct_fns
                };
            }
            _ => {
                // Hex traits don't actually generate anything other than the trait impl which is 1 line.
            }
        }
    }
}

fn do_thing(input: proc_macro::TokenStream, flavor: GenerationFlavor) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // parse the input into a StructInfo which contains all the information we
    // along with some helpful structures to generate our Bitfield code.
    let struct_info = match GenericBuilder::parse(&input) {
        Ok(parsed_struct) => parsed_struct,
        Err(err) => {
            return proc_macro::TokenStream::from(err.to_compile_error());
        }
    };
    let solved: Solved = match struct_info.try_into() {
        Ok(s) => s,
        Err(err) => {
            return proc_macro::TokenStream::from(err.to_compile_error());
        }
    };
    match solved.gen(flavor) {
        Ok(gen) => gen.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}
/// Generates an implementation of the `bondrewd::Bitfield` trait, as well as read and write functions for direct
/// sized u8 arrays access.
/// 
/// This crate is designed so that attributes are only required for fields that
/// are not what you would expect without the attribute. For example if you provide a u8 fields with no
/// attributes, the field would be assumed to be the next 8 bits after the field before it. If a field
/// of bool type without attributes is defined, the field would be assumed to be the next bit after
/// the field before it.
///
/// # Supported Field Types
/// 
/// - All primitives other than usize and isize (i believe ambiguous sizing is bad for this type of work).
///     - Floats currently must be full sized.
///     - Its important to know that there is a small runtime cost for signed numbers.
/// - Structs or Enums which implement Bondrewd's `Bitfield` trait.
/// 
/// ## Primitive Assumptions
/// 
/// When no `bit_length` is specified the amount of bits bondrewd will assume to use is the same as the number the
/// type specifies (ex. u32 assumes 32 bits), in the case of `bool` bondrewd assumes 1 bit.
///
/// # Attributes
///
/// #### Common Attributes
/// 
/// These attributes can be used on a struct, enum or enum variant. When used with an enum they are
/// defaults for the variants, and each variant can be assigned these attributes as well.
/// 
/// - `endianness = {"le", "be" or "ale"}` Describes a default endianness for primitive fields. as of version
///     `0.3.27` the endianness will default to Little Endianness. [example](#endianness-examples)
/// - `bit_traversal = {"front" or "back"}` Defines which end of the byte array to start at. This is a bit
///     index reversal across the entire array from grabbing fields from. [example](#bit-positioning-examples)
/// - `reverse` Defines that the entire byte array should be read backward (first byte index becomes last
///     byte index). [example](#reverse-example)
/// 
/// #### Object Attributes
/// 
/// - `dump` Dumps the bondrewd code generation output in the `target` directory. I got tried of adding and
///     removing this feature for development, so i just didn't remove it.
///
/// #### Struct and Variant Attributes
/// 
/// These should not be used on an enum type (they act funny if you do).
/// 
/// - `enforce_bytes = {BYTES}` Adds a check that requires total bytes defined by fields to equal provided
///     BYTES. [example](#enforce-bits-examples)
/// - `enforce_bits = {BITS}` Adds a check that requires total bits defined by fields to equal provided
///     BITS. [example](#enforce-bits-examples)
/// - `enforce_full_bytes` Adds a check that requires total bits defined by fields to equal a multiple of 8.
///     [example](#enforce-full-bytes-example)
/// - `fill_bits` When present will add an imaginary reserve field to the end to a structure when its bit
///     total does not evenly divide by 8. The imaginary reserve field will detect how may bit need to be
///     filled to make he structure evenly divide by 8. note that these bits will not effect the `BIT_SIZE`
///     constant generated by `bondrewd-derive`. [example](#fill-bits-examples)
/// - `fill_bits = {BITS}` added an imaginary reserve field to the end to a structure using the amount of
///     `BITS` specified. note that these bits will not effect the `BIT_SIZE` constant.
///     generated by `bondrewd-derive`. [example](#fill-bits-examples)
/// - `fill_bytes = {BYTES}` added an imaginary reserve field to the end to a structure using the amount of
///     `BYTES` specified. note that these bits will not effect the `BIT_SIZE` constant. [example](#fill-bytes-example)
///
/// #### Enum Attributes
/// 
/// - `id_bit_length = {BITS}` Describes the amount of bits bondrewd will use to identify which variant is being
///     stored. [example](#enum-example)
/// - `id_byte_length = {BYTES}` Describes the amount of bytes bondrewd will use to identify which variant is being stored.
///
/// #### Variant Attributes
/// 
/// - `variant_id = {ID}` Tell bondrewd the id value to use for the variant. [example](#enum-example).
///     The id can also be defined by a using discriminates [discriminate-example](#enum-with-discriminates).
/// - `invalid` a single Enum Variant can be marked as the "invalid" variant. The invalid variant acts as
///     a catch all for id's that may not be specified. [example](#invalid-enum-variant).
///
/// # Field Attributes
/// 
/// #### Common Field Attributes
/// 
/// - `bit_length = {BITS}` Define the total amount of bits to use when condensed. [example](#simple-example)
/// - `byte_length = {BYTES}` Define the total amount of bytes to use when condensed. [example](#simple-example)
/// - `block_bit_length = {BITS}` Describes a bit length for the entire array dropping lower indexes first.
///     [example](#bitfield-array-examples)
/// - `block_byte_length = {BYTES}` Describes a byte length for the entire array dropping lower indexes
///     first. [example](#bitfield-array-examples)
/// - `element_bit_length = {BITS}` Describes a bit length for each element of an array. (default array
///     type). [example](#bitfield-array-examples)
/// - `element_byte_length = {BYTES}` Describes a byte length for each element of an array. (default array
///     type). [example](#bitfield-array-examples)
/// - `reserve` Defines that this field should be ignored in from and into bytes functions. [example](#reserve-examples)
///     - Reserve requires the fields type to impl [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html).
///         due to `from_bytes` needed to provided a value.
///
/// #### Enum Variant Field Attributes
/// 
/// - `capture_id` Tells Bondrewd to put the value for id in the field on reads, fields
///     with this attribute do NOT get written to the bytes to prevent users from creating improper
///     byte values. [example](#capture-id)
///
/// # Experimental Field Attributes
/// 
/// if you decide to use these remember that they have not been exhaustively tested. when using
/// experimental attributes please be careful and report unexpected behavior to our github issues.
/// 
/// - `bits = "{RANGE}"` - Define the bit indexes yourself rather than let the proc macro figure
///     it out. using a rust range in quotes. the RANGE must provide a inclusively below and exclusively
///     above bounded range (ex. bits = "0..2" means use bits 0 and 1 but NOT 2).
///     [example](#bits-attribute-example)
/// - `read_only` - Bondrewd will not include `from_bytes` or `into_bytes` logic for the field.
/// - `overlapping_bits = {BITS}` - Tells bondrewd that the provided BITS amount is shared
///     with at least 1 other field and should not be included in the overall structure size.
/// - `redundant` - Tells bondrewd that this field's bits are all shared by at least one other field.
///     Bondrewd will not include the bit length in the structures overall bit length
///     (because they are redundant). [example](#redundant-examples)
///     - Bondrewd will read the assigned bits but will not write.
///     - This behaves exactly as combining the attributes:
///         - `read_only`
///         - `overlapping_bits = {FIELD_BIT_LENGTH}` `FIELD_BIT_LENGTH` being the total amount of bits that
///             the field uses.
///
/// # Simple Example
/// 
/// This example is on the front page for bondrewd-derive. Here i will be adding some asserts to show what
/// to expect.
/// I will be defining a data structure with 7 total bytes as:
/// - A boolean field named one will be the first bit.
/// - A floating point field named two will be the next 32 bits. floats must be full sized currently.
/// - A signed integer field named three will be the next 14 bits.
/// - An unsigned integer field named four will be the next 6 bits.
/// - Because these fields do not add up to a number divisible by 8 the last 3 bits will be unused.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct SimpleExample {
///     // fields that are as expected do not require attributes.
///     one: bool,
///     two: f32,
///     #[bondrewd(bit_length = 14)]
///     three: i16,
///     #[bondrewd(bit_length = 6)]
///     four: u8,
/// }
///
/// assert_eq!(7, SimpleExample::BYTE_SIZE);
/// assert_eq!(53, SimpleExample::BIT_SIZE);
/// let mut bytes = SimpleExample {
///     one: false,
///     two: -4.25,
///     three: -1034,
///     four: 63,
/// }.into_bytes();
/// // check the output binary is correct. (i did math by hand
/// // to get the binary). each field is separated by a underscore
/// // in the binary assert to make it easy to see.
/// assert_eq!([
///     0b0_1100000, // one_two,
///     0b01000100,  // two,
///     0b00000000,  // two,
///     0b00000000,  // two,
///     0b0_1110111, // two_three,
///     0b1110110_1, // three_four,
///     0b11111_000, // four_unused
/// ], bytes);
/// // use read functions to get the fields value without
/// // doing a from_bytes call.
/// assert_eq!(false, SimpleExample::read_one(&bytes));
/// assert_eq!(-4.25, SimpleExample::read_two(&bytes));
/// assert_eq!(-1034, SimpleExample::read_three(&bytes));
/// assert_eq!(63, SimpleExample::read_four(&bytes));
/// // overwrite the values with new ones in the byte array.
/// SimpleExample::write_one(&mut bytes, true);
/// SimpleExample::write_two(&mut bytes, 5.5);
/// SimpleExample::write_three(&mut bytes, 511);
/// SimpleExample::write_four(&mut bytes, 0);
/// // from bytes uses the read function so there is no need to
/// // assert the read functions again.
/// let reconstructed = SimpleExample::from_bytes(bytes);
/// // check the values read by from bytes and check if they are
/// // what we wrote to the bytes NOT the original values.
/// assert_eq!(true,reconstructed.one);
/// assert_eq!(5.5,reconstructed.two);
/// assert_eq!(511,reconstructed.three);
/// assert_eq!(0,reconstructed.four);
/// ```
/// 
/// # Endianness Examples
/// 
/// Currently there are 3 supported "endianness" formats:
/// - "be" = Big Endian. Will layout fields one after another little endian byte order.
/// - "le" = Little Endian. Will layout fields one after another little endian byte order.
/// - "ale" = Aligned Little Endian. Some people think having the fields being laid out one after the other is
///     too easy for people to understand, so select manufactures (even in Aerospace) decided that Little Endian means
///     interweaving bit fields that do not align with bytes evenly. see example below for explanation of my micro-rant.
/// 
/// ```
/// // TODO START_HERE add examples for each endianness. using the simple structs from
/// // the `simple_ale`, `simple_be` and `simple_le` tests should do nicely.
/// ```
/// 
/// # Reverse Example
/// 
/// Reverse simply makes Bondrewd index the bytes in the output/input buffers in the opposite order.
/// First index becomes last index and last index becomes the first.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// struct Example {
///     one: u8,
///     two: u8,
///     three: u8,
///     four: u8,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(reverse)]
/// struct ExampleReversed {
///     one: u8,
///     two: u8,
///     three: u8,
///     four: u8,
/// }
///
/// let test = Example {
///     one: 0,
///     two: u8::MAX,
///     three: 0,
///     four: 0b01010101,
/// };
/// let test_reverse = ExampleReversed {
///     one: 0,
///     two: u8::MAX,
///     three: 0,
///     four: 0b01010101,
/// };
/// assert_eq!(test.into_bytes(), [0b00000000, 0b11111111, 0b000000, 0b01010101]);
/// assert_eq!(test_reverse.into_bytes(), [0b01010101, 0b000000, 0b11111111, 0b00000000]);
/// ```
/// 
/// # Bit Positioning Examples
/// 
/// Here Bit positioning will control where bit 0 is. for example if you have a field with 2 bits then
/// 2 fields with 3 bits each, bit positioning will define the direction in which it traverses bit indices,
/// so in our example if 0 is the least significant bit the first field would be the least significant bit
/// in the last index in the byte array. `msb0` is the default.
///
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(bit_traversal = "front")]
/// struct ExampleFront {
///     #[bondrewd(bit_length = 2)]
///     one: u8,
///     #[bondrewd(bit_length = 3)]
///     two: u8,
///     #[bondrewd(bit_length = 3)]
///     three: u8,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(bit_traversal = "back")]
/// struct ExampleBack {
///     #[bondrewd(bit_length = 2)]
///     one: u8,
///     #[bondrewd(bit_length = 3)]
///     two: u8,
///     #[bondrewd(bit_length = 3)]
///     three: u8,
/// }
///
/// let test_front = ExampleFront {
///     one: 0,
///     two: 5,
///     three: 0,
/// };
/// let test_back = ExampleBack {
///     one: 0,
///     two: 5,
///     three: 0,
/// };
/// // in `bit_traversal = "front"` field one is the first 2 bits followed by field two
/// // then field three is the last 3 bits.
/// assert_eq!(test_front.into_bytes(), [0b00_101_000]);
/// // in `bit_traversal = "back"` field three is the first 3 bits followed by field
/// // 2 then field one being the last 2 bits
/// assert_eq!(test_back.into_bytes(), [0b000_101_00]);
/// ```
/// 
/// When using `reverse` and `bit_traversal` in the same structure:
/// - `front` would begin at the least significant bit in the first byte.
/// - `back` would begin at the most significant bit in the last byte.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(bit_traversal = "front", reverse)]
/// struct ExampleFront {
///     #[bondrewd(bit_length = 5)]
///     one: u8,
///     #[bondrewd(bit_length = 4)]
///     two: u8,
///     #[bondrewd(bit_length = 7)]
///     three: u8,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(bit_traversal = "back", reverse)]
/// struct ExampleBack {
///     #[bondrewd(bit_length = 5)]
///     one: u8,
///     #[bondrewd(bit_length = 4)]
///     two: u8,
///     #[bondrewd(bit_length = 7)]
///     three: u8,
/// }
///
/// let test_front = ExampleFront {
///     one: 0,
///     two: u8::MAX,
///     three: 0,
/// };
/// let test_back = ExampleBack {
///     one: 0,
///     two: u8::MAX,
///     three: 0,
/// };
/// // here the 1's belong to field two. i hope this is understandable.
/// assert_eq!(test_front.into_bytes(), [0b10000000, 0b00000111]);
/// assert_eq!(test_back.into_bytes(), [0b11100000, 0b00000001]);
/// ```
///
/// # Bitfield Struct as Field Examples
/// 
/// Nested structs must implement the
/// [`Bitfields`](https://docs.rs/bondrewd/latest/bondrewd/trait.Bitfields.html) trait and be given the
/// `byte_length = {BYTE_SIZE}`, the `BYTE_SIZE` being the number of bytes in the outputs byte array or
/// value in the traits const `BYTE_SIZE`.
/// 
/// ```
/// // this struct uses 52 total bits which means the total BYTE_SIZE is 7.
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct Simple {
///     #[bondrewd(bit_length = 3)]
///     one: u8,
///     #[bondrewd(bit_length = 27)]
///     two: char,
///     #[bondrewd(bit_length = 14)]
///     three: u16,
///     four: i8,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct SimpleWithStruct {
///     #[bondrewd(byte_length = 7)]
///     one: Simple,
///     // structs can also be used in arrays.
///     #[bondrewd(element_byte_length = 7)]
///     two: [Simple; 2],
/// }
/// ```
/// 
/// We can also trim the struct to a bit length, this can be very useful for struct that do not use the
/// full amount of bits available in the byte array. For example if we have a struct that uses 4 bits
/// leaving the remaining 4 bits as unused data, we can make a structure with 2 of the bits structure
/// that still only uses 1 byte.
/// 
/// ```
/// // this struct uses 4 total bits which means the total BYTE_SIZE is 1.
/// use bondrewd::*;
/// #[derive(Bitfields, Clone)]
/// #[bondrewd(endianness = "be")]
/// struct Simple {
///     #[bondrewd(bit_length = 2)]
///     one: u8,
///     #[bondrewd(bit_length = 2)]
///     two: u8,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct SimpleWithStruct {
///     #[bondrewd(bit_length = 4)]
///     one: Simple,
///     #[bondrewd(bit_length = 4)]
///     two: Simple,
/// }
///
/// // SimpleWithStruct uses the amount of bits that 2
/// // Simple structures would use.
/// assert_eq!(SimpleWithStruct::BIT_SIZE, Simple::BIT_SIZE * 2);
/// // But both structures use 1 byte.
/// assert_eq!(SimpleWithStruct::BYTE_SIZE, 1);
/// assert_eq!(SimpleWithStruct::BYTE_SIZE, Simple::BYTE_SIZE);
/// ```
/// 
/// # Bitfield Array Examples
/// 
/// There are 2 types of arrays in Bondrewd:
/// - Block Arrays are "bit chucks" that define a total-used-bits amount and will drop bits starting
///     at the lowest index.
/// - Element Arrays treat each element of the array as its own field and requires a per element
///     bit-length.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct SimpleWithArray {
///     // each u8 in the array contains 4 bits of useful information.
///     #[bondrewd(element_bit_length = 4)]
///     one: [u8; 4],
///     // due to no attributes being present for field `two`,
///     // no bits are missing and the type of array shouldn't
///     // matter, Bondrewd will use element array logic. also boolean
///     // values are assumed to be 1 bit so this will produce
///     // 5 bits in an output.
///     #[bondrewd(element_bit_length = 1)]
///     two: [bool; 5],
///     // the total amount bits in the array should be 20.
///     // [{4 bits},{8 bits},{8 bits}]
///     #[bondrewd(block_bit_length = 20)]
///     three: [u8; 3],
/// }
///
/// let test = SimpleWithArray {
///     // the first 4 bits in index 0 and 2 are 1's to show
///     // that they will not be in the final result due to
///     // each element being set to 4 bits, meaning the values
///     // in those indices will become 0 after into_bytes is called.
///     one: [0b11110000, 0b00001111, 0b11110000, 0b00001001],
///     two: [false, true, false, true, false],
///     // its also worth noting that index 0 here will lose the 4
///     // most significant bits.
///     three: [u8::MAX, 0, 0b10101010],
/// };
/// assert_eq!(test.into_bytes(),
///     [0b0000_1111,  // one[0 and 1]
///      0b0000_1001,  // one[2 and 3]
///      0b01010_111,  // two and three[0]
///      0b1_0000000,  // remaining three[0] and three[1]
///      0b0_1010101,  // remaining three[1] and three[2]
///      0b0_0000000]);// remaining three[2] and 7 unused bits.
/// ```
/// 
/// Structures and Enums can also be used in arrays but there are some extra things to consider.
/// - If `bit_length` of the structs or enums needs to be smaller than the output of either `into_bytes` or `into_primitive` then it is recommended to use element arrays.
/// - Block Arrays, in my opinion, shouldn't be used for Structs or Enums. because in the below example if the `compressed_structures` field was to use `block_bit_length = 104` the array would use 48 bits for index 0 and 56 bits for index 1.
/// 
/// ```
/// // this struct uses 52 total bits which means the total
/// // BYTE_SIZE is 7.
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct SimpleStruct {
///     #[bondrewd(bit_length = 3)]
///     one: u8,
///     #[bondrewd(bit_length = 27)]
///     two: char,
///     #[bondrewd(bit_length = 14)]
///     three: u16,
///     four: i8,
/// }
///
/// // this enum has 4 variants therefore only uses 2 bits
/// // out of 8 in the primitive type.
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", id_bit_length = 2)]
/// enum SimpleEnum {
///     Zero,
///     One,
///     Two,
///     Three,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct ArraysWithStructsAndEnums {
///     #[bondrewd(element_bit_length = 8)]
///     four_byte_four_values: [SimpleEnum; 4],
///     // if we use the element_bit_length we can say to only use 2
///     // bits per SimpleEnum, and due to SimpleEnum only needing 2
///     // bits, this could be desirable. means instead of using 4
///     // bytes to store 4 SimpleEnums, we can use 1 byte.
///     #[bondrewd(element_bit_length = 2)]
///     one_byte_four_values: [SimpleEnum; 4],
///     #[bondrewd(element_byte_length = 7)]
///     waste_a_byte: [SimpleStruct; 2],
///     // if we want to compress the 2 struct in the array we can
///     // take advantage of the fact our struct is only using 52 out
///     // of 56 bits in the compressed/byte form by adding
///     // element bit length = 52. this will make the total size of
///     // the 2 structs in compressed/byte form 104 bits instead of
///     // 112.
///     #[bondrewd(element_bit_length = 52)]
///     compressed_structures: [SimpleStruct; 2],
/// }
/// ```
/// 
/// # Reserve Examples
/// 
/// Reserve fields tell Bondrewd to not include logic for reading or writing the field in the from and
/// into bytes functions. Currently only primitive types are supported.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct ReserveExample {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
///     #[bondrewd(bit_length = 10, reserve)]
///     reserve: u16
/// }
/// assert_eq!(3, ReserveExample::BYTE_SIZE);
/// assert_eq!(24, ReserveExample::BIT_SIZE);
/// let mut bytes = ReserveExample {
///     one: 127,
///     two: 127,
///     reserve: 1023,
/// }.into_bytes();
/// assert_eq!([0b11111111, 0b11111100, 0b00000000], bytes);
/// assert_eq!(127,ReserveExample::read_one(&bytes));
/// assert_eq!(127,ReserveExample::read_two(&bytes));
/// assert_eq!(0,ReserveExample::read_reserve(&bytes));
/// // quick note write_reserve will actually change the bytes in the byte array.
/// ReserveExample::write_reserve(&mut bytes, 42);
/// assert_eq!(42,ReserveExample::read_reserve(&bytes));
/// // but again from/into bytes doesn't care.
/// let reconstructed = ReserveExample::from_bytes(bytes);
/// assert_eq!(127,reconstructed.one);
/// assert_eq!(127,reconstructed.two);
/// assert_eq!(0,reconstructed.reserve);
/// ```
/// 
/// Reserves do not need to be at the end.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct ReserveExample {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 10, reserve)]
///     reserve: u16,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// assert_eq!(3, ReserveExample::BYTE_SIZE);
/// assert_eq!(24, ReserveExample::BIT_SIZE);
/// let mut bytes = ReserveExample {
///     one: 127,
///     two: 127,
///     reserve: 1023,
/// }.into_bytes();
/// assert_eq!(127, ReserveExample::read_one(&bytes));
/// assert_eq!(127, ReserveExample::read_two(&bytes));
/// assert_eq!(0, ReserveExample::read_reserve(&bytes));
/// ReserveExample::write_reserve(&mut bytes, 42);
/// assert_eq!(42, ReserveExample::read_reserve(&bytes));
/// let reconstructed = ReserveExample::from_bytes(bytes);
/// assert_eq!(127,reconstructed.one);
/// assert_eq!(127,reconstructed.two);
/// assert_eq!(0,reconstructed.reserve);
/// ```
/// 
/// # Fill Bits Examples
/// 
/// > if you are using `fill_bits` on a nested structure please read [Using `fill_bits` on a nested structure](#using-fill_bits-on-a-nested-structure)
///
/// Fill bits is used here to make the total output byte size 2 bytes. If `fill_bits` attribute was not
/// present the total output byte size would be still be 2, but the positioning of bits can be less predicable when
/// using other attributes like `reverse` or `bit_traversal`.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", fill_bits = 2)]
/// struct FilledBits {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// assert_eq!(2, FilledBits::BYTE_SIZE);
/// // Note that the fill_bits are included in `BIT_SIZE` this is because fill just added a reserve field internally.
/// assert_eq!(16, FilledBits::BIT_SIZE);
/// ```
/// 
/// `fill_bits` when no value is provided will detect how many bits are needed to make
/// `BIT_SIZE / BYTE_SIZE == 8`. This example produces the same results as the example above.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", fill_bits)]
/// struct FilledBits {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// assert_eq!(2, FilledBits::BYTE_SIZE);
/// assert_eq!(16, FilledBits::BIT_SIZE);
/// ```
/// 
/// ## Using `fill_bits` on a nested structure
/// 
/// Because bondrewd allows bit reversal with structures that do not have a bit count that divides evenly by 8,
/// the location of bits will change in cases such as `Aligned Little Endian`. This can cause some unexpected results.
/// It is recommended you use `fill_bits` or `reserve` fields to make the `BIT_SIZE` evenly divisible by 8.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "ale", fill_bits)]
/// struct FilledBits {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "ale")]
/// struct UnfilledBits {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// assert_eq!(2, FilledBits::BYTE_SIZE);
/// assert_eq!(16, FilledBits::BIT_SIZE);
/// assert_eq!(2, UnfilledBits::BYTE_SIZE);
/// assert_eq!(14, UnfilledBits::BIT_SIZE);
/// assert_eq!(FilledBits {one: 127, two: 127}.into_bytes(), [0b11111111,0b00111111]);
/// assert_eq!(UnfilledBits {one: 127, two: 127}.into_bytes(), [0b11111100,0b11111111]);
/// ```
/// 
/// # Fill Bytes Example
/// 
/// Fill bytes is used here to make the total output byte size 3 bytes. If fill bytes attribute was not
/// present the total output byte size would be 2.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", fill_bytes = 1)]
/// struct FilledBytes {
///     one: u8,
///     two: u8,
/// }
/// assert_eq!(3, FilledBytes::BYTE_SIZE);
/// assert_eq!(24, FilledBytes::BIT_SIZE);
/// ```
/// 
/// Here im going to compare the example above to the closest alternative using a reserve field:
/// - `FilledBytes` only has 2 field, so only 2 fields are required for instantiation, where as `ReservedBytes` still needs a value for the reserve field despite from/into bytes not using the value anyway.
/// - `ReservedBytes` has 2 extra functions that Filled Bytes does not, `write_reserve` and `read_reserve`.
/// - One more thing to consider is reserve fields are currently confined to primitives, if more than 128 reserve bits are required at the end, `fill_bytes` is the only supported way of doing this.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct ReservedBytes {
///     one: u8,
///     two: u8,
///     #[bondrewd(reserve)]
///     reserve: u8
/// }
/// assert_eq!(3, ReservedBytes::BYTE_SIZE);
/// assert_eq!(24, ReservedBytes::BIT_SIZE);
/// ```
/// 
/// # Enforce Bits Examples
/// 
/// Enforce Bits/Bytes Main purpose is to act as a compile time check to ensure how many bit you think
/// are being use is the actual amount of bits being used.\
/// Here i have 2 fields with a total defined bit-length of 6, and then an undecorated boolean field. I
/// also have trust issues so i want to verify that the bool is only using 1 bit making the total bit
/// length of the struct 7 bits. Adding `enforce_bits = 7` will force a compiler error if the calculated
/// total bit length is not 7.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", enforce_bits = 7)]
/// struct FilledBytesEnforced {
///     #[bondrewd(bit_length = 4)]
///     one: u8,
///     #[bondrewd(bit_length = 2)]
///     two: u8,
///     three: bool
/// }
/// assert_eq!(1, FilledBytesEnforced::BYTE_SIZE);
/// assert_eq!(7, FilledBytesEnforced::BIT_SIZE);
/// ```
/// 
/// Here is the same example where i assigned the "incorrect" the `bit_length` of the first field making the
/// total 8 instead of 7.
/// 
/// ```compile_fail
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", enforce_bits = 7)]
/// struct FilledBytesEnforced {
///     #[bondrewd(bit_length = 5)]
///     one: u8,
///     #[bondrewd(bit_length = 2)]
///     two: u8,
///     three: bool
/// }
/// ```
///   
/// These next 3 examples all attempt to have near the same end results. A total output of 3 bytes, but the
/// last 10 of them will be reserved/unused (should be ignored and assumed to be 0).
///
/// In this first example i will be showing what a struct might look like without fill bytes, then in the
/// second example i will show the the same end result but without a reserve field. First will be defining
/// all 24 total bits as 3 fields marking the last field of 10 bits with the reserve attribute
/// because we don't want from/into bytes functions to process those bytes.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", enforce_bytes = 3)]
/// struct FilledBytesEnforced {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
///     #[bondrewd(bit_length = 10, reserve)]
///     reserve: u16
/// }
/// assert_eq!(3, FilledBytesEnforced::BYTE_SIZE);
/// assert_eq!(24, FilledBytesEnforced::BIT_SIZE);
/// ```
/// 
/// Also note that [`fill_bytes`](#fill-bytes-examples) does NOT effect how `enforce_bytes` works.
/// `enforce_bytes` will check the total bit length before the bits are filled.
///   
/// Here i am telling Bondrewd to make the total byte length 3 using `fill_bytes`.
/// This Example fails to build because only 16 bits are being defined by fields and `enforce_bytes`
/// is telling Bondrewd to expect 16 bits to be used by defined fields.
/// 
/// ```compile_fail
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", fill_bytes = 1, enforce_bytes = 3)]
/// struct FilledBytesEnforced {
///     one: u8,
///     two: u8,
/// }
/// ```
/// 
/// To fix this we need to make sure our enforcement value is the amount of bits defined by the fields NOT
/// the expected `FilledBytesEnforced::BYTE_SIZE`.
///   
/// Here is the Correct usage of these two attributes working together.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", fill_bits = 3, enforce_bits = 14)]
/// struct FilledBytesEnforced {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// let _ = FilledBytesEnforced::from_bytes([0,0,0]);
/// // we are enforcing 14 bits but fill_bytes is creating
/// // an imaginary reserve field from bit index 14 to
/// // index 23
/// assert_eq!(17, FilledBytesEnforced::BIT_SIZE);
/// assert_eq!(3, FilledBytesEnforced::BYTE_SIZE);
/// ```
/// 
/// # Enforce Full Bytes Example
/// 
/// `enforce_full_bytes` adds a check during parsing phase of Bondrewd which will throw an error if the
/// total bits determined from the defined fields is not a multiple of 8. This was included for those
/// like me that get paranoid they entered something in wrong.
/// 
/// ```compile_fail
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", enforce_full_bytes)]
/// struct FilledBytesEnforced {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// ```
/// 
/// In this case if we still wanted fields one and two to remain 7 bits we need to add another field
/// to use the remaining 2 bits.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", enforce_full_bytes)]
/// struct FilledBytesEnforced {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
///     #[bondrewd(bit_length = 2, reserve)]
///     reserve: u16
/// }
/// assert_eq!(2, FilledBytesEnforced::BYTE_SIZE);
/// assert_eq!(16, FilledBytesEnforced::BIT_SIZE);
/// ```
/// 
/// # Enum Examples
///
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", id_bit_length = 2)]
/// enum SimpleEnum {
///     Zero,
///     One,
///     Two,
///     Three,
/// }
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "le")]
/// struct StructWithEnumExample {
///     #[bondrewd(bit_length = 3)]
///     one: u8,
///     #[bondrewd(bit_length = 2)]
///     two: SimpleEnum,
///     #[bondrewd(bit_length = 3)]
///     three: u8,
/// }
/// ```
/// 
/// Enums can also be used in [arrays](#bitfield-array-examples)
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", id_bit_length = 2)]
/// enum Simple {
///     One,
///     Two,
///     Three,
///     Four,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct SimpleWithStruct {
///     // bit length is not required for enums but in this case where only 4 possible variants are in
///     // our enums 2 bits is all that is needed. also note using more bits than possible variants is
///     // not a problem because the catch all system will protect you from bad inputs.
///     #[bondrewd(bit_length = 2)]
///     one: Simple,
///     #[bondrewd(element_bit_length = 2)]
///     two: [Simple; 3],
/// }
/// ```
/// 
/// # Bits Attribute Example
/// 
/// First i will replicate the [Simple Example](#simple-example) to show an equivalent use.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct SimpleExample {
///     // fields that are as expected do not require attributes.
///     // #[bondrewd(bits = "0..1")] this could be used but is not needed.
///     one: bool,
///     // #[bondrewd(bits = "1..33")] this could be used but is not needed.
///     two: f32,
///     #[bondrewd(bits = "33..47")]
///     three: i16,
///     #[bondrewd(bits = "47..53")]
///     four: u8,
/// }
///
/// assert_eq!(7, SimpleExample::BYTE_SIZE);
/// assert_eq!(53, SimpleExample::BIT_SIZE);
/// let mut bytes = SimpleExample {
///     one: false,
///     two: -4.25,
///     three: -1034,
///     four: 63,
/// }.into_bytes();
/// // check the output binary is correct. (i did math by hand
/// // to get the binary). each field is separated by a underscore
/// // in the binary assert to make it easy to see.
/// assert_eq!([
///     0b0_1100000, // one_two,
///     0b01000100,  // two,
///     0b00000000,  // two,
///     0b00000000,  // two,
///     0b0_1110111, // two_three,
///     0b1110110_1, // three_four,
///     0b11111_000, // four_unused
/// ], bytes);
/// // use read functions to get the fields value without
/// // doing a from_bytes call.
/// assert_eq!(false, SimpleExample::read_one(&bytes));
/// assert_eq!(-4.25, SimpleExample::read_two(&bytes));
/// assert_eq!(-1034, SimpleExample::read_three(&bytes));
/// assert_eq!(63, SimpleExample::read_four(&bytes));
/// // overwrite the values with new ones in the byte array.
/// SimpleExample::write_one(&mut bytes, true);
/// SimpleExample::write_two(&mut bytes, 5.5);
/// SimpleExample::write_three(&mut bytes, 511);
/// SimpleExample::write_four(&mut bytes, 0);
/// // from bytes uses the read function so there is no need to
/// // assert the read functions again.
/// let reconstructed = SimpleExample::from_bytes(bytes);
/// // check the values read by from bytes and check if they are
/// // what we wrote to the bytes NOT the original values.
/// assert_eq!(true,reconstructed.one);
/// assert_eq!(5.5,reconstructed.two);
/// assert_eq!(511,reconstructed.three);
/// assert_eq!(0,reconstructed.four);
/// ```
/// 
/// # Redundant Examples
/// 
/// In this example we will has fields share data. flags in the example will represent a u8 storing
/// multiple boolean flags, but all of the flags within are also fields in the struct. if we mark
/// flags as `redundant` above the boolean flag fields then flags will be `read_only` (effects nothing
/// during an `into_bytes` call).
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct SimpleExample {
///     // fields that are as expected do not require attributes.
///     one: bool,
///     two: f32,
///     #[bondrewd(bit_length = 14)]
///     three: i16,
///     // the field is above the bits it shares because bondrewd
///     // will get the last non-shared set of bits to base its start from.
///     #[bondrewd(redundant, bit_length = 6)]
///     flags: u8,
///     flag_one: bool,
///     flag_two: bool,
///     flag_three: bool,
///     flag_four: bool,
///     flag_five: bool,
///     flag_six: bool,
/// }
///
/// assert_eq!(7, SimpleExample::BYTE_SIZE);
/// assert_eq!(53, SimpleExample::BIT_SIZE);
/// let mut bytes = SimpleExample {
///     one: false,
///     two: -4.25,
///     three: -1034,
///     flags: 0,
///     flag_one: true,
///     flag_two: true,
///     flag_three: true,
///     flag_four: true,
///     flag_five: true,
///     flag_six: true,
/// }.into_bytes();
/// // check the output binary is correct. (i did math by hand
/// // to get the binary). each field is separated by a underscore
/// // in the binary assert to make it easy to see.
/// assert_eq!([
///     0b0_1100000, // one_two,
///     0b01000100,  // two,
///     0b00000000,  // two,
///     0b00000000,  // two,
///     0b0_1110111, // two_three,
///     0b1110110_1, // three_four,
///     0b11111_000, // four_unused
/// ], bytes);
/// // use read functions to get the fields value without
/// // doing a from_bytes call.
/// assert_eq!(false, SimpleExample::read_one(&bytes));
/// assert_eq!(-4.25, SimpleExample::read_two(&bytes));
/// assert_eq!(-1034, SimpleExample::read_three(&bytes));
/// // notice i can still use the read calls for the redundant field.
/// assert_eq!(63, SimpleExample::read_flags(&bytes));
/// assert_eq!(true,SimpleExample::read_flag_one(&bytes));
/// assert_eq!(true,SimpleExample::read_flag_two(&bytes));
/// assert_eq!(true,SimpleExample::read_flag_three(&bytes));
/// assert_eq!(true,SimpleExample::read_flag_four(&bytes));
/// assert_eq!(true,SimpleExample::read_flag_five(&bytes));
/// assert_eq!(true,SimpleExample::read_flag_six(&bytes));
/// // overwrite the values with new ones in the byte array.
/// SimpleExample::write_one(&mut bytes, true);
/// SimpleExample::write_two(&mut bytes, 5.5);
/// SimpleExample::write_three(&mut bytes, 511);
/// // notice i can still use the write calls for the redundant field.
/// SimpleExample::write_flags(&mut bytes, 0);
/// // from bytes uses the read function so there is no need to
/// // assert the read functions again.
/// let reconstructed = SimpleExample::from_bytes(bytes);
/// // check the values read by from bytes and check if they are
/// // what we wrote to the bytes NOT the original values.
/// assert_eq!(true,reconstructed.one);
/// assert_eq!(5.5,reconstructed.two);
/// assert_eq!(511,reconstructed.three);
/// assert_eq!(0,reconstructed.flags);
/// assert_eq!(false,reconstructed.flag_one);
/// assert_eq!(false,reconstructed.flag_two);
/// assert_eq!(false,reconstructed.flag_three);
/// assert_eq!(false,reconstructed.flag_four);
/// assert_eq!(false,reconstructed.flag_five);
/// assert_eq!(false,reconstructed.flag_six);
/// ```
/// 
/// we can also have the flags below if we use the `bits` attribute.
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be")]
/// struct SimpleExample {
///     // fields that are as expected do not require attributes.
///     one: bool,
///     two: f32,
///     #[bondrewd(bit_length = 14)]
///     three: i16,
///     // the field is above the bits it shares because bondrewd
///     // will get the last non-shared set of bits to base its start from.
///     flag_one: bool,
///     flag_two: bool,
///     flag_three: bool,
///     flag_four: bool,
///     flag_five: bool,
///     flag_six: bool,
///     #[bondrewd(redundant, bits = "47..53")]
///     flags: u8,
/// }
///
/// assert_eq!(7, SimpleExample::BYTE_SIZE);
/// assert_eq!(53, SimpleExample::BIT_SIZE);
/// let mut bytes = SimpleExample {
///     one: false,
///     two: -4.25,
///     three: -1034,
///     flags: 0,
///     flag_one: true,
///     flag_two: true,
///     flag_three: true,
///     flag_four: true,
///     flag_five: true,
///     flag_six: true,
/// }.into_bytes();
/// // check the output binary is correct. (i did math by hand
/// // to get the binary). each field is separated by a underscore
/// // in the binary assert to make it easy to see.
/// assert_eq!([
///     0b0_1100000, // one_two,
///     0b01000100,  // two,
///     0b00000000,  // two,
///     0b00000000,  // two,
///     0b0_1110111, // two_three,
///     0b1110110_1, // three_four,
///     0b11111_000, // four_unused
/// ], bytes);
/// // use read functions to get the fields value without
/// // doing a from_bytes call.
/// assert_eq!(false, SimpleExample::read_one(&bytes));
/// assert_eq!(-4.25, SimpleExample::read_two(&bytes));
/// assert_eq!(-1034, SimpleExample::read_three(&bytes));
/// // notice i can still use the read calls for the redundant field.
/// assert_eq!(63, SimpleExample::read_flags(&bytes));
/// assert_eq!(true, SimpleExample::read_flag_one(&bytes));
/// assert_eq!(true, SimpleExample::read_flag_two(&bytes));
/// assert_eq!(true, SimpleExample::read_flag_three(&bytes));
/// assert_eq!(true, SimpleExample::read_flag_four(&bytes));
/// assert_eq!(true, SimpleExample::read_flag_five(&bytes));
/// assert_eq!(true, SimpleExample::read_flag_six(&bytes));
/// // overwrite the values with new ones in the byte array.
/// SimpleExample::write_one(&mut bytes, true);
/// SimpleExample::write_two(&mut bytes, 5.5);
/// SimpleExample::write_three(&mut bytes, 511);
/// // notice i can still use the write calls for the redundant field.
/// SimpleExample::write_flags(&mut bytes, 0);
/// // from bytes uses the read function so there is no need to
/// // assert the read functions again.
/// let reconstructed = SimpleExample::from_bytes(bytes);
/// // check the values read by from bytes and check if they are
/// // what we wrote to the bytes NOT the original values.
/// assert_eq!(true, reconstructed.one);
/// assert_eq!(5.5, reconstructed.two);
/// assert_eq!(511, reconstructed.three);
/// assert_eq!(0, reconstructed.flags);
/// assert_eq!(false, reconstructed.flag_one);
/// assert_eq!(false, reconstructed.flag_two);
/// assert_eq!(false, reconstructed.flag_three);
/// assert_eq!(false, reconstructed.flag_four);
/// assert_eq!(false, reconstructed.flag_five);
/// assert_eq!(false, reconstructed.flag_six);
/// ```
/// 
/// # Enum Example
/// 
/// Because enums can provide a lot of ambiguity there is a requirement that The last variant is
/// always considered the "Invalid Variant", which simply means that it will be a
/// catch-all in the match statement for the generated `Bitfields::from_bytes()` function.
/// See [Generated From Bytes](#generated-from-bytes) below.
///
/// ```
/// use bondrewd::*;
///
/// #[derive(Bitfields)]
/// #[bondrewd(endianness = "be", id_bit_length = 2, enforce_bytes = 3)]
/// enum Thing {
///     One {
///         a: u16,
///     },
///     Two {
///         a: u16,
///         #[bondrewd(bit_length = 6)]
///         b: u8,
///     },
///     Three {
///         #[bondrewd(bit_length = 7)]
///         d: u8,
///         #[bondrewd(bit_length = 15)]
///         e: u16,
///     },
///     #[bondrewd(id = 0)]
///     Idk,
/// }
///
/// let thing = Thing::One { a: 1 };
/// let bytes = thing.into_bytes();
/// // the first two bits are the id followed by Variant One's `a` field.
/// assert_eq!(bytes[0], 0b01_000000);
/// assert_eq!(bytes[1], 0b00000000);
/// // because Variant One doesn't use the full amount of bytes so the last 6 bytes are just filler.
/// assert_eq!(bytes[2], 0b01_000000);
/// ```
/// 
/// # Enum With Discriminates
/// 
/// This example has Variant's `Three`, `Two`, `One`, and `Idk`. The variants with
/// numbers as their names are listed from highest to lowest to show case an easy issue you may run into.
///
/// #### Issue You May Run Into
/// 
/// Because I am:
/// - Setting the last variant's, "Idk" variant, id to `0`,
/// - Setting the first variant's, `Three` variant, id to `3`,
/// - Setting the second variant's, `One` variant, id to `1`,
/// - And variant `Two` does not have a defined id.
///
///
/// variant `Two` will be assigned an id of the next lowest value not already used, if more than 1 was undefined
/// the assignment would go from top to bottom. This happens internally in bondrewd for its code generation
/// but the `#[repr(u8)]` attribute assigns values for if you want to represent the variant as a number,
/// and you should be aware `repr` does not look forward or backward for used numbers, meaning you will
/// get an error from `repr` if you:
/// - Remove the first variant's, `Three`, id assignment of `3`. The first variant will be assigned
///     zero regardless of the last variant being manually assigned that number already.
/// - Change the second variant's, `One`, id assignment of `1` to `2`. `repr` will assume that this should
///     be that last variant's value plus one which is `3` and already used.
/// 
/// #### Discriminate Example
/// 
/// ```
/// use bondrewd::*;
///
/// #[derive(Bitfields)]
/// #[repr(u8)]
/// #[bondrewd(endianness = "be", id_bit_length = 2, enforce_bytes = 3)]
/// enum Thing {
///     Three {
///         #[bondrewd(bit_length = 7)]
///         d: u8,
///         #[bondrewd(bit_length = 15)]
///         e: u16,
///     } = 3,
///     One {
///         a: u16,
///     } = 1,
///     Two {
///         a: u16,
///         #[bondrewd(bit_length = 6)]
///         b: u8,
///     },
///     Idk = 0,
/// }
///
/// let thing = Thing::One { a: 1 };
/// let bytes = thing.into_bytes();
/// // the first two bits are the id followed by Variant One's `a` field.
/// assert_eq!(bytes[0], 0b01_000000);
/// assert_eq!(bytes[1], 0b00000000);
/// // because Variant One doesn't use the full amount of bytes so the last 6 bytes are just filler.
/// assert_eq!(bytes[2], 0b01_000000);
/// let two = Thing::Two{
///     a:0,
///     b: 0
/// };
/// assert_eq!(two.id(), 2);
/// ```
/// 
/// #### Capture Id
/// 
/// ```
/// use bondrewd::*;
///
/// #[derive(Bitfields)]
/// #[repr(u8)]
/// #[bondrewd(endianness = "be", id_bit_length = 2, enforce_bits = 18)]
/// enum Thing {
///     One {
///         a: u16,
///     } = 1,
///     Two {
///         #[bondrewd(bit_length = 10)]
///         a: u16,
///         #[bondrewd(bit_length = 6)]
///         b: u8,
///     } = 2,
///     Idk {
///         #[bondrewd(capture_id)]
///         id: u8,
///         a: u16,
///     } = 0,
/// }
///
/// // fields with capture_id will use the id_bit_length so defining the bit_length is unnecessary.
/// assert_eq!(Thing::BYTE_SIZE, 3);
/// assert_eq!(Thing::BIT_SIZE, 18);
/// // fields that are capturing the id do not write.
/// let mut bytes = Thing::Idk { id: 3, a: 0 }.into_bytes();
/// // despite setting the id to 3 it will be 0 on output, this is to prevent
/// // users from providing a valid id when it should not be.
/// assert_eq!(bytes[0], 0b11000000);
/// assert_eq!(bytes[1], 0b00000000);
/// assert_eq!(bytes[2], 0b00000000);
/// // but the id can be set to anything using the write_variant_id function.
/// Thing::write_variant_id(&mut bytes, 3);
/// // the id is now 3
/// assert_eq!(bytes[0], 0b11000000);
/// assert_eq!(bytes[1], 0b00000000);
/// assert_eq!(bytes[2], 0b00000000);
/// let reconstructed = Thing::from_bytes(bytes);
/// // other than into_bytes everything else with give you the stored value.
/// assert_eq!(reconstructed.id(), 3);
/// match reconstructed {
///     Thing::Idk { id, .. } => assert_eq!(id, 3),
///     _ => panic!("id wasn't 3"),
/// }
/// ```
/// 
/// #### Invalid Enum Variant
/// 
/// For this example we will be show casing why enums do not panic on an invalid case. The generative code
/// for enums always has an invalid variant even when all possible values have a variant. If an Id value
/// does not have an associated variant, `Bitfield::from_bytes` will return the "invalid" variant. This
/// can be combine with [capture-id](#capture-id) to enable nice error handling/reporting.
///
/// > the invalid variant is always the [catch-all](https://doc.rust-lang.org/rust-by-example/flow_control/match.html) in the generated code's match statements
///
/// In this first example we are just accounting for the fact that bondrewd, by default, uses the last
/// variant as the catch all meaning:
/// - a value of 0 as the id would result in a `Thing::Zero` variant
/// - a value of 1 as the id would result in a `Thing::One` variant
/// - a value of 2 or 3 as the id would result in a `Thing::Invalid` variant
///
/// ```
/// use bondrewd::*;
///
/// #[derive(Bitfields, Debug, PartialEq, Eq)]
/// #[bondrewd(endianness = "be", id_bit_length = 2)]
/// enum Thing {
///     Zero, // value of 0
///     One, // value of 1
///     Invalid, // value of 2 or 3
/// }
///
/// assert_eq!(Thing::from_bytes([0b00000000]), Thing::Zero);
/// assert_eq!(Thing::from_bytes([0b01000000]), Thing::One);
/// assert_eq!(Thing::from_bytes([0b10000000]), Thing::Invalid);
/// assert_eq!(Thing::from_bytes([0b11000000]), Thing::Invalid);
/// ```
///
/// > Note that when no id values are specified they will be assigned automatically starting at zero, incrementing 1 for each variant.r
///
/// If for some reason the last variant should not be the catch all you can specify a variant.
/// So for this next example:
/// - a value of 0 as the id would result in a `Thing::Zero` variant
/// - a value of 1 or 3 as the id would result in a `Thing::Invalid` variant
/// - a value of 2 as the id would result in a `Thing::Two` variant
/// 
/// ```
/// use bondrewd::*;
///
/// #[derive(Bitfields, Debug, PartialEq, Eq)]
/// #[bondrewd(endianness = "be", id_bit_length = 2)]
/// enum Thing {
///     /// value of 0
///     Zero,
///     /// value of 1 or 3
///     #[bondrewd(invalid, id = 1)]
///     Invalid,
///     /// value of 2
///     Two,
/// }
///
/// assert_eq!(Thing::from_bytes([0b00000000]), Thing::Zero);
/// assert_eq!(Thing::from_bytes([0b01000000]), Thing::Invalid);
/// assert_eq!(Thing::from_bytes([0b10000000]), Thing::Two);
/// assert_eq!(Thing::from_bytes([0b11000000]), Thing::Invalid);
/// ```
/// 
/// Note that if the id is not specified for the invalid variant it would be assigned 2 as
/// its default value because the invalid variant is processed last within bondrewd.
///
/// ```
/// use bondrewd::*;
///
/// #[derive(Bitfields, Debug, PartialEq, Eq)]
/// #[bondrewd(endianness = "be", id_bit_length = 2)]
/// enum Thing {
///     /// value of 0
///     Zero,
///     /// value of 2 or 3, NOT 1 because Invalid case is handled outside on normal id assignment
///     #[bondrewd(invalid)]
///     Invalid,
///     /// value of 1
///     One,
/// }
///
/// assert_eq!(Thing::from_bytes([0b00000000]), Thing::Zero);
/// assert_eq!(Thing::from_bytes([0b01000000]), Thing::One);
/// assert_eq!(Thing::from_bytes([0b10000000]), Thing::Invalid);
/// assert_eq!(Thing::from_bytes([0b11000000]), Thing::Invalid);
/// ```
#[proc_macro_derive(Bitfields, attributes(bondrewd,))]
pub fn derive_bitfields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_thing(input, GenerationFlavor::standard())
}

/// Slice functions are convenience functions for reading/wring single or multiple fields without reading
/// the entire structure. Bondrewd will provide 2 ways to access the field: slice functions and checked structs.
/// 
/// # Slice Functions
/// 
/// These are functions that are added along side the standard read/write field
/// functions in the impl for the input structure. read/write slice functions will check the length of
/// the slice to insure the amount to bytes needed for the field (NOT the entire structure) are present
/// and return `BitfieldLengthError` if not enough bytes are present.
/// 
/// functions implemented to object by derive:
///
/// `fn read_slice_{field}(&[u8]) -> Result<{field_type}, bondrewd::BondrewdSliceError> { .. }`
/// `fn write_slice_{field}(&mut [u8], {field_type}) -> Result<(), bondrewd::BondrewdSliceError> { .. }`
///
/// # Checked Structs
/// 
/// Deriving the `BitfieldsSlice` trait provides "checked structures" (structures containing a reference to a slice
/// we checked the size of). These checked structs contain all of the same read/write field functions that `Bitfields`
/// provides without the requirement of providing a statically sized array because we checked the size while creating
/// the checked struct.
///
/// > Enums will generate a separate "Checked" structure set for each variant.
#[proc_macro_derive(BitfieldsSlice, attributes(bondrewd,))]
pub fn derive_bitfields_slice(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_thing(input, GenerationFlavor::slice())
}

/// `BitfieldsDyn` trait implementation allows easier creation of the object without needing an array
/// that has the exact `Bitfield::BYTE_SIZE`.
///
/// `from_vec` will consume bytes from provided buffer.
/// `from_slice` copy the bytes of packet at the start of the buffer provided. if you want to consume those
/// bytes you must manually remove `YouObject::BYTE_SIZE` bytes from the start of the buffer.
#[proc_macro_derive(BitfieldsDyn, attributes(bondrewd,))]
pub fn derive_bitfields_dyn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_thing(input, GenerationFlavor::dynamic())
}

/// `BitfieldsHex` provides from/into hex functions like from/into bytes.
/// 
/// The hex inputs/outputs are \[u8;N\] where N is double the calculated bondrewd `STRUCT_SIZE`.
/// Hex encoding and decoding is based off the [hex](https://crates.io/crates/hex) crate's
/// from/into slice functions but with statically sized arrays so we could eliminate sizing errors.
#[proc_macro_derive(BitfieldsHex, attributes(bondrewd,))]
pub fn derive_bitfields_hex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_thing(input, GenerationFlavor::hex())
}

/// `BitfieldsHexDyn` provides the functions from [`BitfieldsDyn`] for hex encoded bytes.
#[proc_macro_derive(BitfieldsHexDyn, attributes(bondrewd,))]
pub fn derive_bitfields_hex_dyn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_thing(input, GenerationFlavor::hex_dynamic())
}
