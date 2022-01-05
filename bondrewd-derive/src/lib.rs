//! Fast and easy bitfield proc macro
//!
//! Provides a proc macro for compressing a data structure with data which can be expressed with bit
//! lengths that are not a power of Two.
//!
//! # Derive Bitfields
//! - Implements the [`Bitfields`](https://docs.rs/bondrewd/latest/bondrewd/trait.Bitfields.html) trait
//! which offers from\into bytes functions that are non-failable and convert the struct from/into sized
//! u8 arrays ([u8; {total_bit_length * 8}]).
//! - read and write functions that allow the field to be accessed or overwritten within a sized u8 array.
//! - More information about how each field is handled (bit length, endianness, ..), as well as structure
//! wide effects (bit position, default field endianness, ..), can be found on the
//! [`Bitfields Derive`](Bitfields) page.
//!
//! For example we can define a data structure with 7 total bytes as:
//! - a boolean field named one will be the first bit.
//! - a floating point field named two will be the next 32 bits. floats must be full sized
//! currently.
//! - a signed integer field named three will be the next 14 bits.
//! - an unsigned integer field named four will be the next 6 bits.
//!
//! ```
//! // Users code
//! use bondrewd::*;
//! #[derive(Bitfields)]
//! #[bondrewd(default_endianness = "be")]
//! struct SimpleExample {
//!     // fields that are as expected do not require attributes.
//!     one: bool,
//!     two: f32,
//!     #[bondrewd(bit_length = 14)]
//!     three: i16,
//!     #[bondrewd(bit_length = 6)]
//!     four: u8,
//! }
//! ```
//! ```compile_fail
//! // Generated Code
//! impl Bitfields<7usize> for SimpleExample {
//!     const BIT_SIZE: usize = 53usize;
//!     fn into_bytes(self) -> [u8; 7usize] { .. }
//!     fn from_bytes(mut input_byte_buffer: [u8; 7usize]) -> Self { .. }
//! }
//! impl SimpleExample {
//!     pub fn read_one(input_byte_buffer: &[u8; 7usize]) -> bool { .. }
//!     pub fn read_two(input_byte_buffer: &[u8; 7usize]) -> f32 { .. }
//!     pub fn read_three(input_byte_buffer: &[u8; 7usize]) -> i16 { .. }
//!     pub fn read_four(input_byte_buffer: &[u8; 7usize]) -> u8 { .. }
//!     pub fn write_one(output_byte_buffer: &mut [u8; 7usize], mut one: bool) { .. }
//!     pub fn write_two(output_byte_buffer: &mut [u8; 7usize], mut two: f32) { .. }
//!     pub fn write_three(output_byte_buffer: &mut [u8; 7usize], mut three: i16) { .. }
//!     pub fn write_four(output_byte_buffer: &mut [u8; 7usize], mut four: u8) { .. }
//! }
//! ```
//! # Derive BitfieldEnum
//! - Implements the [`BitfieldEnum`](https://docs.rs/bondrewd/latest/bondrewd/trait.BitfieldEnum.html)
//! trait which offers from\into primitive functions that are non-failable and convert the enum from/into
//! a primitive type (u8 is the only currently testing primitive).
//! - more information about controlling the end result (define variant values, define a catch/invalid
//! variant) can be found on the [`BitfieldEnum Derive`](BitfieldEnum) page.
//!
//! ```
//! // Users code
//! use bondrewd::BitfieldEnum;
//! #[derive(BitfieldEnum)]
//! enum SimpleEnum {
//!     Zero,
//!     One,
//!     Six = 6,
//!     Two,
//! }
//! ```
//! ```compile_fail
//! // Generated Struct Code
//! impl bondrewd::BitfieldEnum for SimpleEnum {
//!     type Primitive = u8;
//!     fn into_primitive(self) -> u8 {
//!         match self {
//!             Self::Zero => 0,
//!             Self::One => 1,
//!             Self::Six => 6,
//!             Self::Two => 2,
//!         }
//!     }
//!     fn from_primitive(input: u8) -> Self {
//!         match input {
//!             0 => Self::Zero,
//!             1 => Self::One,
//!             6 => Self::Six,
//!             _ => Self::Two,
//!         }
//!     }
//! }
//! ```
//!
//! # Other Crate Features
//! * `slice_fns` generates slice functions:
//!     * `fn read_slice_{field}(&[u8]) -> [Result<{field_type}, bondrewd::BondrewdSliceError>] {}`
//!     * `fn set_slice_{field}(&mut [u8], {field_type}) -> [Result<(), bondrewd::BondrewdSliceError>] {}`
//! * `hex_fns` provided from/into hex functions like from/into bytes. the hex inputs/outputs are \[u8;N\]
//! where N is double the calculated bondrewd STRUCT_SIZE. hex encoding and decoding is based off the
//! [hex](https://crates.io/crates/hex) crate's from/into slice functions but with statically sized
//! arrays so we could eliminate sizing errors.
//!
//! ### Full Example Generated code
//! ```
//! use bondrewd::*;
//! struct SimpleExample {
//!     one: bool,
//!     two: f32,
//!     three: i16,
//!     four: u8,
//! }
//! impl Bitfields<7usize> for SimpleExample {
//!     const BIT_SIZE: usize = 53usize;
//!     fn into_bytes(self) -> [u8; 7usize] {
//!         let mut output_byte_buffer: [u8; 7usize] = [0u8; 7usize];
//!         let one = self.one;
//!         output_byte_buffer[0usize] |= ((one as u8) << 7usize) & 128u8;
//!         let two = self.two;
//!         let two_bytes = (two.to_bits().rotate_right(1u32)).to_be_bytes();
//!         output_byte_buffer[0usize] |= two_bytes[0usize] & 127u8;
//!         output_byte_buffer[1usize] |= two_bytes[1usize];
//!         output_byte_buffer[2usize] |= two_bytes[2usize];
//!         output_byte_buffer[3usize] |= two_bytes[3usize];
//!         output_byte_buffer[4usize] |= two_bytes[0] & 128u8;
//!         let three = self.three;
//!         let three_bytes = (three.rotate_right(7u32)).to_be_bytes();
//!         output_byte_buffer[4usize] |= three_bytes[1usize] & 127u8;
//!         output_byte_buffer[5usize] |= three_bytes[0] & 254u8;
//!         let four = self.four;
//!         let four_bytes = (four.rotate_right(5u32)).to_be_bytes();
//!         output_byte_buffer[5usize] |= four_bytes[0usize] & 1u8;
//!         output_byte_buffer[6usize] |= four_bytes[0] & 248u8;
//!         output_byte_buffer
//!     }
//!     fn from_bytes(mut input_byte_buffer: [u8; 7usize]) -> Self {
//!         let one = Self::read_one(&input_byte_buffer);
//!         let two = Self::read_two(&input_byte_buffer);
//!         let three = Self::read_three(&input_byte_buffer);
//!         let four = Self::read_four(&input_byte_buffer);
//!         Self {
//!             one,
//!             two,
//!             three,
//!             four,
//!         }
//!     }
//! }
//! impl SimpleExample {
//!     #[inline]
//!     pub fn read_one(input_byte_buffer: &[u8; 7usize]) -> bool {
//!         ((input_byte_buffer[0usize] & 128u8) != 0)
//!     }
//!     #[inline]
//!     pub fn read_two(input_byte_buffer: &[u8; 7usize]) -> f32 {
//!         f32::from_bits(
//!             u32::from_be_bytes({
//!                 let mut two_bytes: [u8; 4usize] = [0u8; 4usize];
//!                 two_bytes[0usize] |= input_byte_buffer[0usize] & 127u8;
//!                 two_bytes[1usize] |= input_byte_buffer[1usize];
//!                 two_bytes[2usize] |= input_byte_buffer[2usize];
//!                 two_bytes[3usize] |= input_byte_buffer[3usize];
//!                 two_bytes[0] |= input_byte_buffer[4usize] & 128u8;
//!                 two_bytes
//!             })
//!             .rotate_left(1u32),
//!         )
//!     }
//!     #[inline]
//!     pub fn read_three(input_byte_buffer: &[u8; 7usize]) -> i16 {
//!         i16::from_be_bytes({
//!             let mut three_bytes: [u8; 2usize] = if (input_byte_buffer[4usize] & 64u8) == 64u8 {
//!                 [1u8, 128u8]
//!             } else {
//!                 [0u8; 2usize]
//!             };
//!             three_bytes[1usize] |= input_byte_buffer[4usize] & 127u8;
//!             three_bytes[0] |= input_byte_buffer[5usize] & 254u8;
//!             three_bytes
//!         })
//!         .rotate_left(7u32)
//!     }
//!     #[inline]
//!     pub fn read_four(input_byte_buffer: &[u8; 7usize]) -> u8 {
//!         u8::from_be_bytes({
//!             let mut four_bytes: [u8; 1usize] = [0u8; 1usize];
//!             four_bytes[0usize] |= input_byte_buffer[5usize] & 1u8;
//!             four_bytes[0] |= input_byte_buffer[6usize] & 248u8;
//!             four_bytes
//!         })
//!         .rotate_left(5u32)
//!     }
//!     #[inline]
//!     pub fn write_one(output_byte_buffer: &mut [u8; 7usize], mut one: bool) {
//!         output_byte_buffer[0usize] &= 127u8;
//!         output_byte_buffer[0usize] |= ((one as u8) << 7usize) & 128u8;
//!     }
//!     #[inline]
//!     pub fn write_two(output_byte_buffer: &mut [u8; 7usize], mut two: f32) {
//!         output_byte_buffer[0usize] &= 128u8;
//!         output_byte_buffer[1usize] = 0u8;
//!         output_byte_buffer[2usize] = 0u8;
//!         output_byte_buffer[3usize] = 0u8;
//!         output_byte_buffer[4usize] &= 127u8;
//!         let two_bytes = (two.to_bits().rotate_right(1u32)).to_be_bytes();
//!         output_byte_buffer[0usize] |= two_bytes[0usize] & 127u8;
//!         output_byte_buffer[1usize] |= two_bytes[1usize];
//!         output_byte_buffer[2usize] |= two_bytes[2usize];
//!         output_byte_buffer[3usize] |= two_bytes[3usize];
//!         output_byte_buffer[4usize] |= two_bytes[0] & 128u8;
//!     }
//!     #[inline]
//!     pub fn write_three(output_byte_buffer: &mut [u8; 7usize], mut three: i16) {
//!         output_byte_buffer[4usize] &= 128u8;
//!         output_byte_buffer[5usize] &= 1u8;
//!         let three_bytes = (three.rotate_right(7u32)).to_be_bytes();
//!         output_byte_buffer[4usize] |= three_bytes[1usize] & 127u8;
//!         output_byte_buffer[5usize] |= three_bytes[0] & 254u8;
//!     }
//!     #[inline]
//!     pub fn write_four(output_byte_buffer: &mut [u8; 7usize], mut four: u8) {
//!         output_byte_buffer[5usize] &= 254u8;
//!         output_byte_buffer[6usize] &= 7u8;
//!         let four_bytes = (four.rotate_right(5u32)).to_be_bytes();
//!         output_byte_buffer[5usize] |= four_bytes[0usize] & 1u8;
//!         output_byte_buffer[6usize] |= four_bytes[0] & 248u8;
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
/// sized u8 arrays access. this crate is designed so that attributes are only required for fields that
/// are not what you would expect without the attribute. for example if you provide a u8 fields with no 
/// attributes, the field would be assumed to be the next 8 bits after the field before it. if a field 
/// of bool type without attributes is defined, the field would be assumed to be the next bit after
/// the field before it.
///
/// # Supported Field Types
/// - All primitives other than usize and isize (i believe ambiguous sizing is bad for this type of work).
///     - Floats currently must be full sized.
///     - Its important to know that there is a small runtime cost for signed numbers.
/// - Enums which implement the BitfieldEnum trait in bondrewd.
/// - Structs which implement the Bitfield trait in bondrewd.
///
/// # Struct Attributes
/// - `default_endianness = {"le" or "be"}` describes a default endianness for primitive fields. 
/// [example](#simple-example)
/// - `read_from = {"msb0" or "lsb0"}` defines bit positioning. which end of the byte array to start at.
/// - `enforce_bytes = {BYTES}` adds a check that requires total bytes defined by fields to equal provided
/// BYTES. [example](#enforce-bits-example)
/// - `enforce_bits = {BITS}` adds a check that requires total bits defined by fields to equal provided
/// BITS. [example](#enforce-bits-example)
/// - `enforce_full_bytes` adds a check that requires total bits defined by fields to equal a multiple of 8.
/// - `fill_bytes = {BYTES}` will force the output/input byte array size to be the provided SIZE amount of
/// bytes. [example](#fill-bytes-example)
/// - `reverse` defines that the entire byte array should be read backward (first index becomes last index).
/// no runtime cost.
///
/// # Field Attributes
/// - `bit_length = {BITS}` define the total amount of bits to use when condensed. [example](#simple-example)
/// - `byte_length = {BYTES}` define the total amount of bytes to use when condensed. [example](#simple-example)
/// - `endianness = {"le" or "be"}` define per field endianess.
/// - `block_bit_length = {BITS}` describes a bit length for the entire array dropping lower indexes first.
/// (default array type). [example](#bitfield-array-example)
/// - `block_byte_length = {BYTES}` describes a byte length for the entire array dropping lower indexes
/// first. (default array type). [example](#bitfield-array-example)
/// - `element_bit_length = {BITS}` describes a bit length for each element of an array.
/// [example](#bitfield-array-example)
/// - `element_byte_length = {BYTES}` describes a byte length for each element of an array.
/// [example](#bitfield-array-example)
/// - `enum_primitive = "u8"` defines the size of the enum. the BitfieldEnum currently only supports u8.
/// [example](#enum-examples)
/// - `struct_size = {SIZE}` defines the field as a struct which implements the Bitfield trait and the
/// BYTE_SIZE const defined in said trait. [example](#bitfield-struct-as-field-example)
/// - `reserve` defines that this field should be ignored in from and into bytes functions.
/// [example](#reserve-examples)
///     - reserve attribute is only supported for primitive types currently.
/// - /!Untested!\ `bits = "RANGE"` - define the bit indexes yourself rather than let the proc macro figure
/// it out. using a rust range in quotes.
/// 
/// # Simple Example
/// this example is on the front page for bondrewd-derive. here we will be adding some asserts to show what
/// to expect.
/// i will be defining a data structure with 7 total bytes as:
/// - a boolean field named one will be the first bit.
/// - a floating point field named two will be the next 32 bits. floats must be full sized
/// currently.
/// - a signed integer field named three will be the next 14 bits.
/// - an unsigned integer field named four will be the next 6 bits.
/// - because these fields do not add up to a power of 2 the last 3 bits will be unused.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be")]
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
/// fn main(){
///     assert_eq!(7, SimpleExample::BYTE_SIZE);
///     assert_eq!(53, SimpleExample::BIT_SIZE);
///     let mut bytes = SimpleExample {
///         one: false,
///         two: -4.25,
///         three: -1034,
///         four: 63,
///     }.into_bytes();
///     // check the output binary is correct. (i did math by hand
///     // to get the binary). each field is separated by a underscore
///     // in the binary assert to make it easy to see.
///     assert_eq!([
///         0b0_1100000, // one_two,
///         0b01000100,  // two,
///         0b00000000,  // two,
///         0b00000000,  // two,
///         0b0_1110111, // two_three,
///         0b1110110_1, // three_four,
///         0b11111_000, // four_unused
///     ], bytes);
///     // use read functions to get the fields value without
///     // doing a from_bytes call.
///     assert_eq!(false, SimpleExample::read_one(&bytes));
///     assert_eq!(-4.25, SimpleExample::read_two(&bytes));
///     assert_eq!(-1034, SimpleExample::read_three(&bytes));
///     assert_eq!(63, SimpleExample::read_four(&bytes));
///     // overwrite the values with new ones in the byte array.
///     SimpleExample::write_one(&mut bytes, true);
///     SimpleExample::write_two(&mut bytes, 5.5);
///     SimpleExample::write_three(&mut bytes, 511);
///     SimpleExample::write_four(&mut bytes, 0);
///     // from bytes uses the read function so there is no need to
///     // assert the read functions again.
///     let reconstructed = SimpleExample::from_bytes(bytes);
///     // check the values read by from bytes and check if they are
///     // what we wrote to the bytes NOT the origanal values.
///     assert_eq!(true,reconstructed.one);
///     assert_eq!(5.5,reconstructed.two);
///     assert_eq!(511,reconstructed.three);
///     assert_eq!(0,reconstructed.four);
/// }
/// ```
///
/// # Bitfield Array Example
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be")]
/// struct SimpleWithArray {
///     // each u8 in the array contains 4 bits of useful information.
///     #[bondrewd(element_bit_length = 4)]
///     one: [u8; 4],
///     // due to no attributes being present for field `two`, no bits are missing and the type of array
///     // shouldn't matter bondrewd will use block array logic. also boolean values are assumed to be 1
///     // bit so this will produce 5 bits in an output.
///     two: [bool; 5],
///     // the total amount bits in the array. [{4 bits},{8 bits},{8 bits}]
///     #[bondrewd(block_bit_length = 20)]
///     three: [u8; 3],
/// }
/// ```
/// # Bitfield Struct as Field Example
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be")]
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
/// #[bondrewd(default_endianness = "be")]
/// struct SimpleWithStruct {
///     #[bondrewd(struct_size = 7)]
///     one: Simple,
///     #[bondrewd(struct_size = 7)]
///     two: [Simple; 2],
/// }
/// ```
/// # Reserve Examples
/// reserve fields tell bondrewd to not include logic for reading or writing the field in the from and
/// into bytes functions. currently only primitive types are supported.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be")]
/// struct ReserveExample {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
///     #[bondrewd(bit_length = 10, reserve)]
///     reserve: u16
/// }
/// fn main() {
///     assert_eq!(3, ReserveExample::BYTE_SIZE);
///     assert_eq!(24, ReserveExample::BIT_SIZE);
///     let mut bytes = ReserveExample {
///         one: 127,
///         two: 127,
///         reserve: 1023,
///     }.into_bytes();
///     assert_eq!([0b11111111, 0b11111100, 0b00000000], bytes);
///     assert_eq!(127,ReserveExample::read_one(&bytes));
///     assert_eq!(127,ReserveExample::read_two(&bytes));
///     assert_eq!(0,ReserveExample::read_reserve(&bytes));
///     // quick note write_reserve will actually change the bytes in the byte array.
///     ReserveExample::write_reserve(&mut bytes, 42);
///     assert_eq!(42,ReserveExample::read_reserve(&bytes));
///     // but again from/into bytes doesn't care.
///     let reconstructed = ReserveExample::from_bytes(bytes);
///     assert_eq!(127,reconstructed.one);
///     assert_eq!(127,reconstructed.two);
///     assert_eq!(0,reconstructed.reserve);
/// }
/// ```
/// reserves do not need to be at the end.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", fill_bytes = 3)]
/// struct ReserveExample {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 10, reserve)]
///     reserve: u16,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// fn main() {
///     assert_eq!(3, ReserveExample::BYTE_SIZE);
///     assert_eq!(24, ReserveExample::BIT_SIZE);
///     let mut bytes = ReserveExample {
///         one: 127,
///         two: 127,
///         reserve: 1023,
///     }.into_bytes();
///     assert_eq!(127, ReserveExample::read_one(&bytes));
///     assert_eq!(127, ReserveExample::read_two(&bytes));
///     assert_eq!(0, ReserveExample::read_reserve(&bytes));
///     ReserveExample::write_reserve(&mut bytes, 42);
///     assert_eq!(42, ReserveExample::read_reserve(&bytes));
///     let reconstructed = ReserveExample::from_bytes(bytes);
///     assert_eq!(127,reconstructed.one);
///     assert_eq!(127,reconstructed.two);
///     assert_eq!(0,reconstructed.reserve);
/// }
/// ```
/// # Fill Bytes Example
/// fill bytes is used here to make the total output byte size 3 bytes. if fill bytes attribute was not
/// present the total output byte size would be 2.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", fill_bytes = 3)]
/// struct FilledBytes {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// fn main() {
///     assert_eq!(3, FilledBytes::BYTE_SIZE);
///     assert_eq!(24, FilledBytes::BIT_SIZE);
/// }
/// ```
/// here im going to compare the example above to the closest alternative using a reserve field:
/// - FilledBytes only has 2 field, so only 2 fields are required for instantiation, where as ReservedBytes
/// still needs a value for the reserve field despite from/into bytes not using the value anyway.
/// - ReservedBytes has 2 extra function that FilledBytes does not, write_reserve and read_reserve.
/// - one more thing to consider is reserve fields are currently confined to primitives, if more than 128
/// reserve bits are required at the end, fill_bytes is the only supported way of doing this.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be")]
/// struct ReservedBytes {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
///     #[bondrewd(bit_length = 10, reserve)]
///     reserve: u16
/// }
/// fn main() {
///     assert_eq!(3, ReservedBytes::BYTE_SIZE);
///     assert_eq!(24, ReservedBytes::BIT_SIZE);
/// }
/// ```
/// # Enforce Bits Example
/// these 3 examples all attempt to have near the same end results. a total output of 3 bytes, but the last
/// 10 of them will be reserved (should be ignored and assumed to be 0).
/// 
/// in this first example we are defining all 24 total bits as 3 fields marking the last field of 10 bits
/// with the reserve attribute, this reserve attribute is only here for making a comparison in the next
/// example and should be ignored in this context because it is not necessary.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", enforce_bytes = 3)]
/// struct FilledBytesEnforced {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
///     #[bondrewd(bit_length = 10, reserve)]
///     reserve: u16
/// }
/// fn main() {
///     assert_eq!(3, FilledBytesEnforced::BYTE_SIZE);
///     assert_eq!(24, FilledBytesEnforced::BIT_SIZE);
/// }
/// ```
/// fill bytes is used here to show that fill_bytes does NOT effect how enforce bytes works. enforce bytes
/// will check the total bit length before the bits are filled.
/// 
/// 
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", fill_bytes = 3, enforce_bits = 14)]
/// struct FilledBytesEnforced {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// fn main() {
///     assert_eq!(3, FilledBytesEnforced::BYTE_SIZE);
///     assert_eq!(24, FilledBytesEnforced::BIT_SIZE);
/// }
/// ```
/// here we can see that enforce bits fails when you include the filled bits in the enforcement
/// attributes value.
/// ```compile_fail
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", fill_bytes = 3, enforce_bytes = 3)]
/// struct FilledBytesEnforced {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// fn main() {
///     assert_eq!(3, FilledBytesEnforced::BYTE_SIZE);
///     assert_eq!(24, FilledBytesEnforced::BIT_SIZE);
/// }
/// ```
/// # Enum Examples
/// for enum derive examples goto [BitfieldEnum Derive](BitfieldEnum).
/// ```
/// use bondrewd::*;
/// #[derive(BitfieldEnum)]
/// enum SimpleEnum {
///     Zero,
///     One,
///     Two,
///     Three,
/// }
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "le")]
/// struct StructWithEnumExample {
///     #[bondrewd(bit_length = 3)]
///     one: u8,
///     #[bondrewd(enum_primitive = "u8", bit_length = 2)]
///     two: SimpleEnum,
///     #[bondrewd(bit_length = 3)]
///     three: u8,
/// }
/// ```
/// ```
/// use bondrewd::*;
/// #[derive(BitfieldEnum)]
/// enum Simple {
///     One,
///     Two,
///     Three,
///     Four,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be")]
/// struct SimpleWithStruct {
///     // bit length is not required for enums but in this case where only 4 possible variants are in
///     // our enums 2 bits is all that is needed. also note using more bits than possible variants is
///     // not a problem because the catch all system will protect you from bad inputs.
///     #[bondrewd(bit_length = 2, enum_primitive = "u8")]
///     one: Simple,
///     #[bondrewd(element_bit_length = 2, enum_primitive = "u8")]
///     two: [Simple; 3],
/// }
/// ```
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
    // println!("{:?}", struct_info);
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
    {
        setters = false;
    }
    #[cfg(feature = "setters")]
    {
        setters = true;
    }
    let setters_quote = if setters {
        match structs::struct_fns::create_into_bytes_field_quotes(&struct_info) {
            Ok(parsed_struct) => parsed_struct,
            Err(err) => {
                return TokenStream::from(err.to_compile_error());
            }
        }
    } else {
        quote! {}
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
/// # Features
/// - Generates code for the BitfieldEnum trait which allows an enum to be used by Bitfield structs.
/// - Literal values. ex. `Variant = 0,`
/// - Automatic Value Assignment for non-literal variants. Variants are assigned values starting from 0
/// incrementing by 1 skipping values taken by literal definitions (That means you can mix and match
/// inferred values a code defined literal values).
/// - Catch Variants
///     - Catch Value is a variant that will store values that don't match the reset of the variants.
///     using a Catch Value is as simple as making a variant with a primitive value (if the bondrewd_enum
///     attribute is present the primitive types must match). ex `InvalidVariant(u8),`.
///     - Catch All variant is used to insure that Results are not needed. Catch all will generate a
///     `_ => {..}` match arm so that enums don't need to have as many variants as there are values in
///     the defined primitive. Catch all can be defined with a `#[bondrewd_enum(invalid)]` attribute or last variant will
///     Automatically become a catch all if no Catch is defined.
///
/// # Other Features
/// - Support for implementation of [`std::cmp::PartialEq`] for the given primitive (currently only u8)
///
/// # Typical Example
/// ```
/// use bondrewd::BitfieldEnum;
/// // the primitive type will be assumed to be u8 because there are less than
/// // 256 variants. also any value above 3 passed into from_primitive will
/// // will be caught as a Three due to the catch all system.
/// #[derive(BitfieldEnum, PartialEq, Debug)]
/// enum SimpleEnum {
///     Zero,
///     One,
///     Two,
///     Three,
/// }
/// 
/// fn main(){
///     assert_eq!(SimpleEnum::Zero.into_primitive(), 0);
///     assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(0));
///     assert_eq!(SimpleEnum::One.into_primitive(), 1);
///     assert_eq!(SimpleEnum::One, SimpleEnum::from_primitive(1));
///     assert_eq!(SimpleEnum::Two.into_primitive(), 2);
///     assert_eq!(SimpleEnum::Two, SimpleEnum::from_primitive(2));
///     assert_eq!(SimpleEnum::Three.into_primitive(), 3);
///     for i in 3..=u8::MAX {
///         assert_eq!(SimpleEnum::Three, SimpleEnum::from_primitive(i));
///     }
/// }
/// ```
/// # Literal Example
/// ```
/// use bondrewd::BitfieldEnum;
/// #[derive(BitfieldEnum, PartialEq, Debug)]
/// enum SimpleEnum {
///     Life = 42,
///     Min = 0,
///     U8Max = 255,
///     Unlucky = 13,
/// }
/// 
/// fn main(){
///     assert_eq!(SimpleEnum::Life.into_primitive(), 42);
///     assert_eq!(SimpleEnum::Life, SimpleEnum::from_primitive(42));
///     assert_eq!(SimpleEnum::Min.into_primitive(), 0);
///     assert_eq!(SimpleEnum::Min, SimpleEnum::from_primitive(0));
///     assert_eq!(SimpleEnum::U8Max.into_primitive(), 255);
///     assert_eq!(SimpleEnum::U8Max, SimpleEnum::from_primitive(255));
///     assert_eq!(SimpleEnum::Unlucky.into_primitive(), 13);
///     // check all values not defined and 13 get detected as Unlucky
///     for i in 1..42 {
///         assert_eq!(SimpleEnum::Unlucky, SimpleEnum::from_primitive(i));
///     }
///     for i in 43..u8::MAX {
///         assert_eq!(SimpleEnum::Unlucky, SimpleEnum::from_primitive(i));
///     }
/// }
/// ```
/// # Custom Catch All Example
/// This example shows that we can mark any variant as the catch all variant.
/// in this case Bondrewd will give One the value of 1 and make One catch all values not defined because
/// of the invalid attribute. because no literals are present variants will be assigned values, the lower
/// the variant in the list the higher the value assigned.
/// ```
/// use bondrewd::BitfieldEnum;
/// #[derive(BitfieldEnum, PartialEq, Debug)]
/// enum SimpleEnum {
///     Zero, // assigned 0
///     #[bondrewd_enum(invalid)]
///     One, // assigned 1
///     Two, // assigned 2
///     Three, // assigned 3
/// }
/// 
/// fn main(){
///     assert_eq!(SimpleEnum::Zero.into_primitive(), 0);
///     assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(0));
///     assert_eq!(SimpleEnum::One.into_primitive(), 1);
///     assert_eq!(SimpleEnum::One, SimpleEnum::from_primitive(1));
///     assert_eq!(SimpleEnum::Two.into_primitive(), 2);
///     assert_eq!(SimpleEnum::Two, SimpleEnum::from_primitive(2));
///     assert_eq!(SimpleEnum::Three.into_primitive(), 3);
///     assert_eq!(SimpleEnum::Three, SimpleEnum::from_primitive(3));
///     // remaining possible values are caught as One.
///     for i in 4..=u8::MAX {
///         assert_eq!(SimpleEnum::One, SimpleEnum::from_primitive(i));
///     }
/// }
/// ```
/// # Catch Value Example
/// in some cases we might need to know what the invalid value passed into from_primitive actually was. in
/// my own code there is a enum field that gets encrypted and would become pretty much any value and cause
/// panics in the library i used before writing Bondrewd. to fix this Bondrewd offers the ability to make 1
/// variant a tuple or struct variant with exactly one field which must be the primitive type the enum
/// gets converted to/from, than the variant values not covered will be stored in the field.
/// ```
/// use bondrewd::BitfieldEnum;
/// #[derive(BitfieldEnum, PartialEq, Debug)]
/// enum SimpleEnum {
///     Zero,
///     One,
///     Two,
///     Three(u8),
/// }
/// 
/// fn main(){
///     assert_eq!(SimpleEnum::Zero.into_primitive(), 0);
///     assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(0));
///     assert_eq!(SimpleEnum::One.into_primitive(), 1);
///     assert_eq!(SimpleEnum::One, SimpleEnum::from_primitive(1));
///     assert_eq!(SimpleEnum::Two.into_primitive(), 2);
///     assert_eq!(SimpleEnum::Two, SimpleEnum::from_primitive(2));
///     for i in 3..=u8::MAX {
///         assert_eq!(SimpleEnum::Three(i), SimpleEnum::from_primitive(i));
///     }
/// }
/// ```
/// # Complex Example
/// here we expect:
/// - SimpleEnum::Nine = 9,
/// - SimpleEnum::One  = 1,
/// - SimpleEnum::Zero = 0 and accept 3, 4, 6, 7, 8, and 10..u8::MAX in from_primitive(),
/// - SimpleEnum::Five = 5,
/// - SimpleEnum::Two  = 2,
/// ```
/// use bondrewd::BitfieldEnum;
/// #[derive(BitfieldEnum, PartialEq, Debug)]
/// enum SimpleEnum {
///     Nine = 9,
///     // because variant `One` is the first non-literal variant it will be
///     // given the first available value
///     One,
///     // Literals can still be a catch all.
///     #[bondrewd_enum(invalid)]
///     Zero = 0,
///     Five = 5,
///     // because variant `One` is the second non-literal variant it will be
///     // given the second available value
///     Two,
/// }
///
/// fn main(){
///     assert_eq!(SimpleEnum::Nine.into_primitive(), 9);
///     assert_eq!(SimpleEnum::Nine, SimpleEnum::from_primitive(9));
///     assert_eq!(SimpleEnum::One.into_primitive(), 1);
///     assert_eq!(SimpleEnum::One, SimpleEnum::from_primitive(1));
///     assert_eq!(SimpleEnum::Zero.into_primitive(), 0);
///     assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(0));
///     assert_eq!(SimpleEnum::Five.into_primitive(), 5);
///     assert_eq!(SimpleEnum::Five, SimpleEnum::from_primitive(5));
///     assert_eq!(SimpleEnum::Two.into_primitive(), 2);
///     assert_eq!(SimpleEnum::Two, SimpleEnum::from_primitive(2));
///     // Invalid tests
///     assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(3));
///     assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(4));
///     assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(6));
///     assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(7));
///     assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(8));
///     for i in 10..=u8::MAX {
///         assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(i));
///     }
/// }
/// ```
#[proc_macro_derive(BitfieldEnum, attributes(bondrewd_enum))]
pub fn derive_bondrewd_enum(input: TokenStream) -> TokenStream {
    // TODO added the ability to give a Catch Value Variant a Literal value.
    let input = parse_macro_input!(input as DeriveInput);
    let enum_info = match EnumInfo::parse(&input) {
        Ok(parsed_enum) => parsed_enum,
        Err(err) => {
            return TokenStream::from(err.to_compile_error());
        }
    };
    let into = match enums::into_bytes::generate_into_bytes(&enum_info) {
        Ok(i) => i,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };
    let from = match enums::from_bytes::generate_from_bytes(&enum_info) {
        Ok(f) => f,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };
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
