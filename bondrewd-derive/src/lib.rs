//! Fast and easy bitfield proc macro
//!
//! Provides a proc macro used on Structures or Enums which provides functions for:
//! - outputting your native rust structure/enum as a statically sized byte array.
//! - re-creating your structure/enum from a statically sized byte array (no possible errors). Or if the
//! `dyn_fns` feature is enabled a `&[u8]` or `Vec<u8>` (possible length error)
//! - read/writing individual fields within a byte array (or slice with `dyn_fns` feature enabled) without re-writing unaffected bytes.
//!
//! # Under Development
//!
//! This project is worked on as needed for my job. Public releases and requests are low priority,
//! because they have to be done during my free time which i spend on other/more-fun projects. These are
//! the only changes i am currently considering at the moment:
//!
//! - Full support for floating point numbers. I would like to be able to be able to
//!     have fully dynamic floating point with customizable exponent and mantissa as well
//!     and unsigned option.
//! - Allow assumed id sizing for enums. we do check the provided id size is large enough so if non is defined
//!     we could just use the calculated smallest allowable.
//! - Getters and Setters. Most of this is actually done already, just need:
//!     - Write array setters.
//!
//! ### Other Planned Changes
//! - Merge `from_bytes` and `into_bytes` generation to have less duplicated code/runtime-logic.
//! - LOTS of in code documentation, this is greatly needed in the math code.
//!
//! # Derive Bitfields
//! - Implements the [`Bitfields`](https://docs.rs/bondrewd/latest/bondrewd/trait.Bitfields.html) trait
//! which offers from\into bytes functions that are non-failable and convert the struct from/into sized
//! u8 arrays.
//! - `read` and `write` functions that allow fields to be accessed or overwritten within a properly sized u8 array.
//! - See the [`Bitfields Derive`](Bitfields) page for more information about how to change details for how
//! each field is handled (bit length, endianness, ..), or structure wide effects
//! (starting bit position, default field endianness, ..).
//! .
//!
//! For example we can define a data structure with 7 total bytes as:
//! - A boolean field named one will be the first bit.
//! - A floating point field named two will be the next 32 bits. floats must be full sized
//! currently.
//! - A signed integer field named three will be the next 14 bits.
//! - An unsigned integer field named four will be the next 6 bits.
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
//! Generated Code with function logic omitted. [Full Generated Code](#full-example-generated-code)
//! ```compile_fail
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
//!
//! # Crate Features
//! ### `dyn_fns`
//! Slice functions are convenience functions for reading/wring single or multiple fields without reading
//! the entire structure. Bondrewd will provided 2 ways to access the field:
//!
//! * Single field access. These are functions that are added along side the standard read/write field
//! functions in the impl for the input structure. read/write slice functions will check the length of
//! the slice to insure the amount to bytes needed for the field (NOT the entire structure) are present
//! and return `BitfieldLengthError` if not enough bytes are present.
//!
//!     * `fn read_slice_{field}(&[u8]) -> Result<{field_type}, bondrewd::BondrewdSliceError> { .. }`
//!
//!     * `fn write_slice_{field}(&mut [u8], {field_type}) -> Result<(), bondrewd::BondrewdSliceError> { .. }`
//!
//! * Multiple field access.
//!
//!     * `fn check_slice(&[u8]) -> Result<{struct_name}Checked, bondrewd::BondrewdSliceError> { .. }`
//!       This function will check the size of the slice, if the slice is big enough it will return
//!       a checked structure. the structure will be the same name as the input structure with
//!       "Checked" tacked onto the end. the Checked Structure will have getters for each of the input
//!       structures fields, the naming is the same as the standard `read_{field}` functions.
//!
//!         * `fn read_{field}(&self) -> {field_type} { .. }`
//!
//!     * `fn check_slice_mut(&mut [u8]) -> Result<{struct_name}CheckedMut, bondrewd::BondrewdSliceError> { .. }`
//!       This function will check the size of the slice, if the slice is big enough it will return
//!       a checked structure. the structure will be the same name as the input structure with
//!       `CheckedMut` tacked onto the end. the Checked Structure will have getters and setters for each
//!       of the input structures fields, the naming is the same as the standard `read_{field}` and
//!       `write_{field}` functions.
//!
//!         * `fn read_{field}(&self) -> {field_type} { .. }`
//!
//!         * `fn write_{field}(&mut self) -> {field_type} { .. }`
//!
//! * `BitfieldsDyn` trait implementation. This allows easier creation of the object without needing an array
//! that has the exact `Bitfield::BYTE_SIZE`.
//!   
//! Example Cargo.toml Bondrewd dependency  
//! `bondrewd = { version = "^0.1", features = ["derive", "dyn_fns"] }`  
//! Example Generated Slice Api:
//! ```compile_fail
//! impl SimpleExample {
//!     const BIT_SIZE: usize = 53usize;
//!     pub fn read_one(input_byte_buffer: &[u8; 7usize]) -> bool {.. }
//!     pub fn read_two(input_byte_buffer: &[u8; 7usize]) -> f32 { .. }
//!     pub fn read_three(input_byte_buffer: &[u8; 7usize]) -> i16 { .. }
//!     pub fn read_four(input_byte_buffer: &[u8; 7usize]) -> u8 { .. }
//!     pub fn read_slice_one(
//!         input_byte_buffer: &[u8],
//!     ) -> Result<bool, bondrewd::BitfieldLengthError> { .. }
//!     pub fn read_slice_two(
//!         input_byte_buffer: &[u8],
//!     ) -> Result<f32, bondrewd::BitfieldLengthError> { .. }
//!     pub fn read_slice_three(
//!         input_byte_buffer: &[u8],
//!     ) -> Result<i16, bondrewd::BitfieldLengthError> { .. }
//!     pub fn read_slice_four(
//!         input_byte_buffer: &[u8],
//!     ) -> Result<u8, bondrewd::BitfieldLengthError> { .. }
//!     pub fn write_one(output_byte_buffer: &mut [u8; 7usize], mut one: bool) { .. }
//!     pub fn write_two(output_byte_buffer: &mut [u8; 7usize], mut two: f32) { .. }
//!     pub fn write_three(output_byte_buffer: &mut [u8; 7usize], mut three: i16) { .. }
//!     pub fn write_four(output_byte_buffer: &mut [u8; 7usize], mut four: u8) { .. }
//!     pub fn write_slice_one(
//!         output_byte_buffer: &mut [u8],
//!         one: bool,
//!     ) -> Result<(), bondrewd::BitfieldLengthError> { .. }
//!     pub fn write_slice_two(
//!         output_byte_buffer: &mut [u8],
//!         two: f32,
//!     ) -> Result<(), bondrewd::BitfieldLengthError> { .. }
//!     pub fn write_slice_three(
//!         output_byte_buffer: &mut [u8],
//!         three: i16,
//!     ) -> Result<(), bondrewd::BitfieldLengthError> { .. }
//!     pub fn write_slice_four(
//!         output_byte_buffer: &mut [u8],
//!         four: u8,
//!     ) -> Result<(), bondrewd::BitfieldLengthError> { .. }
//!     pub fn check_slice(
//!         buffer: &[u8],
//!     ) -> Result<SimpleExampleChecked, bondrewd::BitfieldLengthError> { .. }
//!     pub fn check_slice_mut(
//!         buffer: &mut [u8],
//!     ) -> Result<SimpleExampleCheckedMut, bondrewd::BitfieldLengthError> { .. }
//! }
//! struct SimpleExampleChecked<'a> {
//!     buffer: &'a [u8],
//! }
//! impl<'a> SimpleExampleChecked<'a> {
//!     pub fn read_one(&self) -> bool { .. }
//!     pub fn read_two(&self) -> f32 { .. }
//!     pub fn read_three(&self) -> i16 { .. }
//!     pub fn read_four(&self) -> u8 { .. }
//! }
//! struct SimpleExampleCheckedMut<'a> {
//!     buffer: &'a mut [u8],
//! }
//! impl<'a> SimpleExampleCheckedMut<'a> {
//!     pub fn read_one(&self) -> bool { .. }
//!     pub fn read_two(&self) -> f32 { .. }
//!     pub fn read_three(&self) -> i16 { .. }
//!     pub fn read_four(&self) -> u8 { .. }
//!     pub fn write_one(&mut self, one: bool) { .. }
//!     pub fn write_two(&mut self, two: f32) { .. }
//!     pub fn write_three(&mut self, three: i16) { .. }
//!     pub fn write_four(&mut self, four: u8) { .. }
//! }
//! impl bondrewd::BitfieldsDyn<7usize> for SimpleExample {
//!     fn from_vec(
//!         input_byte_buffer: &mut Vec<u8>,
//!     ) -> Result<Self, bondrewd::BitfieldLengthError> { .. }
//!     fn from_slice(
//!         input_byte_buffer: &[u8],
//!     ) -> Result<Self, bondrewd::BitfieldLengthError> { .. }
//! }
//! ```
//! ### `part_eq_enums`
//! Implements [`PartialEq`] for the type which fits the bits of the a [`Bitfields`] enum's id on the enum.
//! ### `hex_fns`
//! `hex_fns` provided from/into hex functions like from/into bytes. The hex inputs/outputs are \[u8;N\]
//! where N is double the calculated bondrewd `STRUCT_SIZE`. Hex encoding and decoding is based off the
//! [hex](https://crates.io/crates/hex) crate's from/into slice functions but with statically sized
//! arrays so we could eliminate sizing errors.
//!
//! ### Full Example Generated Code
//! ```
//! use bondrewd::*;
//! struct SimpleExample {
//!     one: bool,
//!     two: f32,
//!     three: i16,
//!     four: u8,
//! }
//!
//! impl bondrewd::Bitfields<7usize> for SimpleExample {
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
//!         Self { one, two, three, four }
//!     }
//! }
//! impl SimpleExample {
//!     #[inline]
//!     ///Reads bits 0 through 0 within `input_byte_buffer`, getting the `one` field of a `SimpleExample` in
//!     pub fn read_one(input_byte_buffer: &[u8; 7usize]) -> bool {
//!         (((input_byte_buffer[0usize] & 128u8)) != 0)
//!     }
//!     #[inline]
//!     ///Reads bits 1 through 32 within `input_byte_buffer`, getting the `two` field of a `SimpleExample` i
//!     pub fn read_two(input_byte_buffer: &[u8; 7usize]) -> f32 {
//!         f32::from_bits(
//!             u32::from_be_bytes({
//!                     let mut two_bytes: [u8; 4usize] = [0u8; 4usize];
//!                     two_bytes[0usize] |= input_byte_buffer[0usize] & 127u8;
//!                     two_bytes[1usize] |= input_byte_buffer[1usize];
//!                     two_bytes[2usize] |= input_byte_buffer[2usize];
//!                     two_bytes[3usize] |= input_byte_buffer[3usize];
//!                     two_bytes[0] |= input_byte_buffer[4usize] & 128u8;
//!                     two_bytes
//!                 })
//!                 .rotate_left(1u32),
//!         )
//!     }
//!     #[inline]
//!     ///Reads bits 33 through 46 within `input_byte_buffer`, getting the `three` field of a `SimpleExample
//!     pub fn read_three(input_byte_buffer: &[u8; 7usize]) -> i16 {
//!         i16::from_be_bytes({
//!                 let mut three_bytes: [u8; 2usize] = if (input_byte_buffer[4usize] & 64u8)
//!                     == 64u8
//!                 {
//!                     [1u8, 128u8]
//!                 } else {
//!                     [0u8; 2usize]
//!                 };
//!                 three_bytes[1usize] |= input_byte_buffer[4usize] & 127u8;
//!                 three_bytes[0] |= input_byte_buffer[5usize] & 254u8;
//!                 three_bytes
//!             })
//!             .rotate_left(7u32)
//!     }
//!     #[inline]
//!     ///Reads bits 47 through 52 within `input_byte_buffer`, getting the `four` field of a `SimpleExample`
//!     pub fn read_four(input_byte_buffer: &[u8; 7usize]) -> u8 {
//!         u8::from_be_bytes({
//!                 let mut four_bytes: [u8; 1usize] = [0u8; 1usize];
//!                 four_bytes[0usize] |= input_byte_buffer[5usize] & 1u8;
//!                 four_bytes[0] |= input_byte_buffer[6usize] & 248u8;
//!                 four_bytes
//!             })
//!             .rotate_left(5u32)
//!     }
//!     ///Returns a [SimpleExampleChecked] which allows you to read any field for a `SimpleExample` from pro
//!     pub fn check_slice(
//!         buffer: &[u8],
//!     ) -> Result<SimpleExampleChecked, bondrewd::BitfieldLengthError> {
//!         let buf_len = buffer.len();
//!         if buf_len >= 7usize {
//!             Ok(SimpleExampleChecked { buffer })
//!         } else {
//!             Err(bondrewd::BitfieldLengthError(buf_len, 7usize))
//!         }
//!     }
//!     #[inline]
//!     ///Returns the value for the `one` field of a `SimpleExample` in bitfield form by reading  bits 0 thr
//!     pub fn read_slice_one(
//!         input_byte_buffer: &[u8],
//!     ) -> Result<bool, bondrewd::BitfieldLengthError> {
//!         let slice_length = input_byte_buffer.len();
//!         if slice_length < 1usize {
//!             Err(bondrewd::BitfieldLengthError(slice_length, 1usize))
//!         } else {
//!             Ok((((input_byte_buffer[0usize] & 128u8)) != 0))
//!         }
//!     }
//!     #[inline]
//!     ///Returns the value for the `two` field of a `SimpleExample` in bitfield form by reading  bits 1 thr
//!     pub fn read_slice_two(
//!         input_byte_buffer: &[u8],
//!     ) -> Result<f32, bondrewd::BitfieldLengthError> {
//!         let slice_length = input_byte_buffer.len();
//!         if slice_length < 5usize {
//!             Err(bondrewd::BitfieldLengthError(slice_length, 5usize))
//!         } else {
//!             Ok(
//!                 f32::from_bits(
//!                     u32::from_be_bytes({
//!                             let mut two_bytes: [u8; 4usize] = [0u8; 4usize];
//!                             two_bytes[0usize] |= input_byte_buffer[0usize] & 127u8;
//!                             two_bytes[1usize] |= input_byte_buffer[1usize];
//!                             two_bytes[2usize] |= input_byte_buffer[2usize];
//!                             two_bytes[3usize] |= input_byte_buffer[3usize];
//!                             two_bytes[0] |= input_byte_buffer[4usize] & 128u8;
//!                             two_bytes
//!                         })
//!                         .rotate_left(1u32),
//!                 ),
//!             )
//!         }
//!     }
//!     #[inline]
//!     ///Returns the value for the `three` field of a `SimpleExample` in bitfield form by reading  bits 33
//!     pub fn read_slice_three(
//!         input_byte_buffer: &[u8],
//!     ) -> Result<i16, bondrewd::BitfieldLengthError> {
//!         let slice_length = input_byte_buffer.len();
//!         if slice_length < 6usize {
//!             Err(bondrewd::BitfieldLengthError(slice_length, 6usize))
//!         } else {
//!             Ok(
//!                 i16::from_be_bytes({
//!                         let mut three_bytes: [u8; 2usize] = if (input_byte_buffer[4usize]
//!                             & 64u8) == 64u8
//!                         {
//!                             [1u8, 128u8]
//!                         } else {
//!                             [0u8; 2usize]
//!                         };
//!                         three_bytes[1usize] |= input_byte_buffer[4usize] & 127u8;
//!                         three_bytes[0] |= input_byte_buffer[5usize] & 254u8;
//!                         three_bytes
//!                     })
//!                     .rotate_left(7u32),
//!             )
//!         }
//!     }
//!     #[inline]
//!     ///Returns the value for the `four` field of a `SimpleExample` in bitfield form by reading  bits 47 t
//!     pub fn read_slice_four(
//!         input_byte_buffer: &[u8],
//!     ) -> Result<u8, bondrewd::BitfieldLengthError> {
//!         let slice_length = input_byte_buffer.len();
//!         if slice_length < 7usize {
//!             Err(bondrewd::BitfieldLengthError(slice_length, 7usize))
//!         } else {
//!             Ok(
//!                 u8::from_be_bytes({
//!                         let mut four_bytes: [u8; 1usize] = [0u8; 1usize];
//!                         four_bytes[0usize] |= input_byte_buffer[5usize] & 1u8;
//!                         four_bytes[0] |= input_byte_buffer[6usize] & 248u8;
//!                         four_bytes
//!                     })
//!                     .rotate_left(5u32),
//!             )
//!         }
//!     }
//!     #[inline]
//!     ///Writes to bits 0 through 0 within `output_byte_buffer`, setting the `one` field of a `SimpleExampl
//!     pub fn write_one(output_byte_buffer: &mut [u8; 7usize], mut one: bool) {
//!         output_byte_buffer[0usize] &= 127u8;
//!         output_byte_buffer[0usize] |= ((one as u8) << 7usize) & 128u8;
//!     }
//!     #[inline]
//!     ///Writes to bits 1 through 32 within `output_byte_buffer`, setting the `two` field of a `SimpleExamp
//!     pub fn write_two(output_byte_buffer: &mut [u8; 7usize], mut two: f32) {
//!         output_byte_buffer[0usize] &= 128u8;
//!         output_byte_buffer[1usize] &= 0u8;
//!         output_byte_buffer[2usize] &= 0u8;
//!         output_byte_buffer[3usize] &= 0u8;
//!         output_byte_buffer[4usize] &= 127u8;
//!         let two_bytes = (two.to_bits().rotate_right(1u32)).to_be_bytes();
//!         output_byte_buffer[0usize] |= two_bytes[0usize] & 127u8;
//!         output_byte_buffer[1usize] |= two_bytes[1usize];
//!         output_byte_buffer[2usize] |= two_bytes[2usize];
//!         output_byte_buffer[3usize] |= two_bytes[3usize];
//!         output_byte_buffer[4usize] |= two_bytes[0] & 128u8;
//!     }
//!     #[inline]
//!     ///Writes to bits 33 through 46 within `output_byte_buffer`, setting the `three` field of a `SimpleEx
//!     pub fn write_three(output_byte_buffer: &mut [u8; 7usize], mut three: i16) {
//!         output_byte_buffer[4usize] &= 128u8;
//!         output_byte_buffer[5usize] &= 1u8;
//!         let three_bytes = (three.rotate_right(7u32)).to_be_bytes();
//!         output_byte_buffer[4usize] |= three_bytes[1usize] & 127u8;
//!         output_byte_buffer[5usize] |= three_bytes[0] & 254u8;
//!     }
//!     #[inline]
//!     ///Writes to bits 47 through 52 within `output_byte_buffer`, setting the `four` field of a `SimpleExa
//!     pub fn write_four(output_byte_buffer: &mut [u8; 7usize], mut four: u8) {
//!         output_byte_buffer[5usize] &= 254u8;
//!         output_byte_buffer[6usize] &= 7u8;
//!         let four_bytes = (four.rotate_right(5u32)).to_be_bytes();
//!         output_byte_buffer[5usize] |= four_bytes[0usize] & 1u8;
//!         output_byte_buffer[6usize] |= four_bytes[0] & 248u8;
//!     }
//!     #[inline]
//!     ///Writes to bits 0 through 0 in `input_byte_buffer` if enough bytes are present in slice, setting th
//!     pub fn write_slice_one(
//!         output_byte_buffer: &mut [u8],
//!         one: bool,
//!     ) -> Result<(), bondrewd::BitfieldLengthError> {
//!         let slice_length = output_byte_buffer.len();
//!         if slice_length < 1usize {
//!             Err(bondrewd::BitfieldLengthError(slice_length, 1usize))
//!         } else {
//!             output_byte_buffer[0usize] &= 127u8;
//!             output_byte_buffer[0usize] |= ((one as u8) << 7usize) & 128u8;
//!             Ok(())
//!         }
//!     }
//!     #[inline]
//!     ///Writes to bits 1 through 32 in `input_byte_buffer` if enough bytes are present in slice, setting t
//!     pub fn write_slice_two(
//!         output_byte_buffer: &mut [u8],
//!         two: f32,
//!     ) -> Result<(), bondrewd::BitfieldLengthError> {
//!         let slice_length = output_byte_buffer.len();
//!         if slice_length < 5usize {
//!             Err(bondrewd::BitfieldLengthError(slice_length, 5usize))
//!         } else {
//!             output_byte_buffer[0usize] &= 128u8;
//!             output_byte_buffer[1usize] &= 0u8;
//!             output_byte_buffer[2usize] &= 0u8;
//!             output_byte_buffer[3usize] &= 0u8;
//!             output_byte_buffer[4usize] &= 127u8;
//!             let two_bytes = (two.to_bits().rotate_right(1u32)).to_be_bytes();
//!             output_byte_buffer[0usize] |= two_bytes[0usize] & 127u8;
//!             output_byte_buffer[1usize] |= two_bytes[1usize];
//!             output_byte_buffer[2usize] |= two_bytes[2usize];
//!             output_byte_buffer[3usize] |= two_bytes[3usize];
//!             output_byte_buffer[4usize] |= two_bytes[0] & 128u8;
//!             Ok(())
//!         }
//!     }
//!     #[inline]
//!     ///Writes to bits 33 through 46 in `input_byte_buffer` if enough bytes are present in slice, setting
//!     pub fn write_slice_three(
//!         output_byte_buffer: &mut [u8],
//!         three: i16,
//!     ) -> Result<(), bondrewd::BitfieldLengthError> {
//!         let slice_length = output_byte_buffer.len();
//!         if slice_length < 6usize {
//!             Err(bondrewd::BitfieldLengthError(slice_length, 6usize))
//!         } else {
//!             output_byte_buffer[4usize] &= 128u8;
//!             output_byte_buffer[5usize] &= 1u8;
//!             let three_bytes = (three.rotate_right(7u32)).to_be_bytes();
//!             output_byte_buffer[4usize] |= three_bytes[1usize] & 127u8;
//!             output_byte_buffer[5usize] |= three_bytes[0] & 254u8;
//!             Ok(())
//!         }
//!     }
//!     #[inline]
//!     ///Writes to bits 47 through 52 in `input_byte_buffer` if enough bytes are present in slice, setting
//!     pub fn write_slice_four(
//!         output_byte_buffer: &mut [u8],
//!         four: u8,
//!     ) -> Result<(), bondrewd::BitfieldLengthError> {
//!         let slice_length = output_byte_buffer.len();
//!         if slice_length < 7usize {
//!             Err(bondrewd::BitfieldLengthError(slice_length, 7usize))
//!         } else {
//!             output_byte_buffer[5usize] &= 254u8;
//!             output_byte_buffer[6usize] &= 7u8;
//!             let four_bytes = (four.rotate_right(5u32)).to_be_bytes();
//!             output_byte_buffer[5usize] |= four_bytes[0usize] & 1u8;
//!             output_byte_buffer[6usize] |= four_bytes[0] & 248u8;
//!             Ok(())
//!         }
//!     }
//!     ///Returns a [SimpleExampleCheckedMut] which allows you to read/write any field for a `SimpleExample`
//!     pub fn check_slice_mut(
//!         buffer: &mut [u8],
//!     ) -> Result<SimpleExampleCheckedMut, bondrewd::BitfieldLengthError> {
//!         let buf_len = buffer.len();
//!         if buf_len >= 7usize {
//!             Ok(SimpleExampleCheckedMut { buffer })
//!         } else {
//!             Err(bondrewd::BitfieldLengthError(buf_len, 7usize))
//!         }
//!     }
//! }
//! ///A Structure which provides functions for getting the fields of a [SimpleExample] in its bitfield form.
//! struct SimpleExampleChecked<'a> {
//!     buffer: &'a [u8],
//! }
//! impl<'a> SimpleExampleChecked<'a> {
//!     #[inline]
//!     ///Reads bits 0 through 0 in pre-checked slice, getting the `one` field of a [SimpleExample] in bitfi
//!     pub fn read_one(&self) -> bool {
//!         let input_byte_buffer: &[u8] = self.buffer;
//!         (((input_byte_buffer[0usize] & 128u8)) != 0)
//!     }
//!     #[inline]
//!     ///Reads bits 1 through 32 in pre-checked slice, getting the `two` field of a [SimpleExample] in bitf
//!     pub fn read_two(&self) -> f32 {
//!         let input_byte_buffer: &[u8] = self.buffer;
//!         f32::from_bits(
//!             u32::from_be_bytes({
//!                     let mut two_bytes: [u8; 4usize] = [0u8; 4usize];
//!                     two_bytes[0usize] |= input_byte_buffer[0usize] & 127u8;
//!                     two_bytes[1usize] |= input_byte_buffer[1usize];
//!                     two_bytes[2usize] |= input_byte_buffer[2usize];
//!                     two_bytes[3usize] |= input_byte_buffer[3usize];
//!                     two_bytes[0] |= input_byte_buffer[4usize] & 128u8;
//!                     two_bytes
//!                 })
//!                 .rotate_left(1u32),
//!         )
//!     }
//!     #[inline]
//!     ///Reads bits 33 through 46 in pre-checked slice, getting the `three` field of a [SimpleExample] in b
//!     pub fn read_three(&self) -> i16 {
//!         let input_byte_buffer: &[u8] = self.buffer;
//!         i16::from_be_bytes({
//!                 let mut three_bytes: [u8; 2usize] = if (input_byte_buffer[4usize] & 64u8)
//!                     == 64u8
//!                 {
//!                     [1u8, 128u8]
//!                 } else {
//!                     [0u8; 2usize]
//!                 };
//!                 three_bytes[1usize] |= input_byte_buffer[4usize] & 127u8;
//!                 three_bytes[0] |= input_byte_buffer[5usize] & 254u8;
//!                 three_bytes
//!             })
//!             .rotate_left(7u32)
//!     }
//!     #[inline]
//!     ///Reads bits 47 through 52 in pre-checked slice, getting the `four` field of a [SimpleExample] in bi
//!     pub fn read_four(&self) -> u8 {
//!         let input_byte_buffer: &[u8] = self.buffer;
//!         u8::from_be_bytes({
//!                 let mut four_bytes: [u8; 1usize] = [0u8; 1usize];
//!                 four_bytes[0usize] |= input_byte_buffer[5usize] & 1u8;
//!                 four_bytes[0] |= input_byte_buffer[6usize] & 248u8;
//!                 four_bytes
//!             })
//!             .rotate_left(5u32)
//!     }
//!     ///Panics if resulting `SimpleExampleChecked` does not contain enough bytes to read a field that is a
//!     pub fn from_unchecked_slice(data: &'a [u8]) -> Self {
//!         Self { buffer: data }
//!     }
//! }
//! ///A Structure which provides functions for getting and setting the fields of a [SimpleExample] in its bi
//! struct SimpleExampleCheckedMut<'a> {
//!     buffer: &'a mut [u8],
//! }
//! impl<'a> SimpleExampleCheckedMut<'a> {
//!     #[inline]
//!     ///Reads bits 0 through 0 in pre-checked slice, getting the `one` field of a [SimpleExample] in bitfi
//!     pub fn read_one(&self) -> bool {
//!         let input_byte_buffer: &[u8] = self.buffer;
//!         (((input_byte_buffer[0usize] & 128u8)) != 0)
//!     }
//!     #[inline]
//!     ///Reads bits 1 through 32 in pre-checked slice, getting the `two` field of a [SimpleExample] in bitf
//!     pub fn read_two(&self) -> f32 {
//!     let input_byte_buffer: &[u8] = self.buffer;
//!         f32::from_bits(
//!             u32::from_be_bytes({
//!                     let mut two_bytes: [u8; 4usize] = [0u8; 4usize];
//!                     two_bytes[0usize] |= input_byte_buffer[0usize] & 127u8;
//!                     two_bytes[1usize] |= input_byte_buffer[1usize];
//!                     two_bytes[2usize] |= input_byte_buffer[2usize];
//!                     two_bytes[3usize] |= input_byte_buffer[3usize];
//!                     two_bytes[0] |= input_byte_buffer[4usize] & 128u8;
//!                     two_bytes
//!                 })
//!                 .rotate_left(1u32),
//!         )
//!     }
//!     #[inline]
//!     ///Reads bits 33 through 46 in pre-checked slice, getting the `three` field of a [SimpleExample] in b
//!     pub fn read_three(&self) -> i16 {
//!         let input_byte_buffer: &[u8] = self.buffer;
//!         i16::from_be_bytes({
//!                 let mut three_bytes: [u8; 2usize] = if (input_byte_buffer[4usize] & 64u8)
//!                     == 64u8
//!                 {
//!                     [1u8, 128u8]
//!                 } else {
//!                     [0u8; 2usize]
//!                 };
//!                 three_bytes[1usize] |= input_byte_buffer[4usize] & 127u8;
//!                 three_bytes[0] |= input_byte_buffer[5usize] & 254u8;
//!                 three_bytes
//!             })
//!             .rotate_left(7u32)
//!     }
//!     #[inline]
//!     ///Reads bits 47 through 52 in pre-checked slice, getting the `four` field of a [SimpleExample] in bi
//!     pub fn read_four(&self) -> u8 {
//!         let input_byte_buffer: &[u8] = self.buffer;
//!         u8::from_be_bytes({
//!                 let mut four_bytes: [u8; 1usize] = [0u8; 1usize];
//!                 four_bytes[0usize] |= input_byte_buffer[5usize] & 1u8;
//!                 four_bytes[0] |= input_byte_buffer[6usize] & 248u8;
//!                 four_bytes
//!             })
//!             .rotate_left(5u32)
//!     }
//!     #[inline]
//!     ///Writes to bits 0 through 0 in pre-checked mutable slice, setting the `one` field of a [SimpleExamp
//!     pub fn write_one(&mut self, one: bool) {
//!         let output_byte_buffer: &mut [u8] = self.buffer;
//!         output_byte_buffer[0usize] &= 127u8;
//!         output_byte_buffer[0usize] |= ((one as u8) << 7usize) & 128u8;
//!     }
//!     #[inline]
//!     ///Writes to bits 1 through 32 in pre-checked mutable slice, setting the `two` field of a [SimpleExam
//!     pub fn write_two(&mut self, two: f32) {
//!         let output_byte_buffer: &mut [u8] = self.buffer;
//!         output_byte_buffer[0usize] &= 128u8;
//!         output_byte_buffer[1usize] &= 0u8;
//!         output_byte_buffer[2usize] &= 0u8;
//!         output_byte_buffer[3usize] &= 0u8;
//!         output_byte_buffer[4usize] &= 127u8;
//!         let two_bytes = (two.to_bits().rotate_right(1u32)).to_be_bytes();
//!         output_byte_buffer[0usize] |= two_bytes[0usize] & 127u8;
//!         output_byte_buffer[1usize] |= two_bytes[1usize];
//!         output_byte_buffer[2usize] |= two_bytes[2usize];
//!         output_byte_buffer[3usize] |= two_bytes[3usize];
//!         output_byte_buffer[4usize] |= two_bytes[0] & 128u8;
//!     }
//!     #[inline]
//!     ///Writes to bits 33 through 46 in pre-checked mutable slice, setting the `three` field of a [SimpleE
//!     pub fn write_three(&mut self, three: i16) {
//!         let output_byte_buffer: &mut [u8] = self.buffer;
//!         output_byte_buffer[4usize] &= 128u8;
//!         output_byte_buffer[5usize] &= 1u8;
//!         let three_bytes = (three.rotate_right(7u32)).to_be_bytes();
//!         output_byte_buffer[4usize] |= three_bytes[1usize] & 127u8;
//!         output_byte_buffer[5usize] |= three_bytes[0] & 254u8;
//!     }
//!     #[inline]
//!     ///Writes to bits 47 through 52 in pre-checked mutable slice, setting the `four` field of a [SimpleEx
//!     pub fn write_four(&mut self, four: u8) {
//!         let output_byte_buffer: &mut [u8] = self.buffer;
//!         output_byte_buffer[5usize] &= 254u8;
//!         output_byte_buffer[6usize] &= 7u8;
//!         let four_bytes = (four.rotate_right(5u32)).to_be_bytes();
//!         output_byte_buffer[5usize] |= four_bytes[0usize] & 1u8;
//!         output_byte_buffer[6usize] |= four_bytes[0] & 248u8;
//!     }
//!     ///Panics if resulting `SimpleExampleCheckedMut` does not contain enough bytes to read a field that i
//!     pub fn from_unchecked_slice(data: &'a mut [u8]) -> Self {
//!         Self { buffer: data }
//!     }
//! }
//! impl bondrewd::BitfieldsDyn<7usize> for SimpleExample {
//!     /**Creates a new instance of `Self` by copying field from the bitfields, removing bytes that where us
//!     # Errors
//!     If the provided `Vec<u8>` does not have enough bytes an error will be returned.*/
//!     fn from_vec(
//!         input_byte_buffer: &mut Vec<u8>,
//!     ) -> Result<Self, bondrewd::BitfieldLengthError> {
//!         if input_byte_buffer.len() < Self::BYTE_SIZE {
//!             return Err(
//!                 bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE),
//!             );
//!         }
//!         let out = {
//!             let one = (((input_byte_buffer[0usize] & 128u8)) != 0);
//!             let two = f32::from_bits(
//!                 u32::from_be_bytes({
//!                         let mut two_bytes: [u8; 4usize] = [0u8; 4usize];
//!                         two_bytes[0usize] |= input_byte_buffer[0usize] & 127u8;
//!                         two_bytes[1usize] |= input_byte_buffer[1usize];
//!                         two_bytes[2usize] |= input_byte_buffer[2usize];
//!                         two_bytes[3usize] |= input_byte_buffer[3usize];
//!                         two_bytes[0] |= input_byte_buffer[4usize] & 128u8;
//!                         two_bytes
//!                     })
//!                     .rotate_left(1u32),
//!             );
//!             let three = i16::from_be_bytes({
//!                     let mut three_bytes: [u8; 2usize] = if (input_byte_buffer[4usize]
//!                         & 64u8) == 64u8
//!                     {
//!                         [1u8, 128u8]
//!                     } else {
//!                         [0u8; 2usize]
//!                     };
//!                     three_bytes[1usize] |= input_byte_buffer[4usize] & 127u8;
//!                     three_bytes[0] |= input_byte_buffer[5usize] & 254u8;
//!                     three_bytes
//!                 })
//!                 .rotate_left(7u32);
//!             let four = u8::from_be_bytes({
//!                     let mut four_bytes: [u8; 1usize] = [0u8; 1usize];
//!                     four_bytes[0usize] |= input_byte_buffer[5usize] & 1u8;
//!                     four_bytes[0] |= input_byte_buffer[6usize] & 248u8;
//!                     four_bytes
//!                 })
//!                 .rotate_left(5u32);
//!             Self { one, two, three, four }
//!         };
//!         let _ = input_byte_buffer.drain(..Self::BYTE_SIZE);
//!         Ok(out)
//!     }
//!     /**Creates a new instance of `Self` by copying field from the bitfields.
//!     # Errors
//!     If the provided `Vec<u8>` does not have enough bytes an error will be returned.*/
//!     fn from_slice(
//!         input_byte_buffer: &[u8],
//!     ) -> Result<Self, bondrewd::BitfieldLengthError> {
//!         if input_byte_buffer.len() < Self::BYTE_SIZE {
//!             return Err(
//!                 bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE),
//!             );
//!         }
//!         let out = {
//!             let one = (((input_byte_buffer[0usize] & 128u8)) != 0);
//!             let two = f32::from_bits(
//!                 u32::from_be_bytes({
//!                         let mut two_bytes: [u8; 4usize] = [0u8; 4usize];
//!                         two_bytes[0usize] |= input_byte_buffer[0usize] & 127u8;
//!                         two_bytes[1usize] |= input_byte_buffer[1usize];
//!                         two_bytes[2usize] |= input_byte_buffer[2usize];
//!                         two_bytes[3usize] |= input_byte_buffer[3usize];
//!                         two_bytes[0] |= input_byte_buffer[4usize] & 128u8;
//!                         two_bytes
//!                     })
//!                     .rotate_left(1u32),
//!             );
//!             let three = i16::from_be_bytes({
//!                     let mut three_bytes: [u8; 2usize] = if (input_byte_buffer[4usize]
//!                         & 64u8) == 64u8
//!                     {
//!                         [1u8, 128u8]
//!                     } else {
//!                         [0u8; 2usize]
//!                     };
//!                     three_bytes[1usize] |= input_byte_buffer[4usize] & 127u8;
//!                     three_bytes[0] |= input_byte_buffer[5usize] & 254u8;
//!                     three_bytes
//!                 })
//!                 .rotate_left(7u32);
//!             let four = u8::from_be_bytes({
//!                     let mut four_bytes: [u8; 1usize] = [0u8; 1usize];
//!                     four_bytes[0usize] |= input_byte_buffer[5usize] & 1u8;
//!                     four_bytes[0] |= input_byte_buffer[6usize] & 248u8;
//!                     four_bytes
//!                 })
//!                 .rotate_left(5u32);
//!             Self { one, two, three, four }
//!         };
//!         Ok(out)
//!     }
//! }
//! ```
extern crate proc_macro;
mod enums;
use enums::parse::EnumInfo;
mod structs;
use structs::into_bytes::{
    create_into_bytes_field_quotes_enum, create_into_bytes_field_quotes_struct,
};
use structs::{common::ObjectInfo, from_bytes::create_from_bytes_field_quotes};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

use crate::structs::from_bytes::create_from_bytes_field_quotes_enum;

/// Generates an implementation of the `bondrewd::Bitfield` trait, as well as peek and set functions for direct
/// sized u8 arrays access. This crate is designed so that attributes are only required for fields that
/// are not what you would expect without the attribute. For example if you provide a u8 fields with no
/// attributes, the field would be assumed to be the next 8 bits after the field before it. If a field
/// of bool type without attributes is defined, the field would be assumed to be the next bit after
/// the field before it.
///
/// # Supported Field Types
/// - All primitives other than usize and isize (i believe ambiguous sizing is bad for this type of work).
///     - Floats currently must be full sized.
///     - Its important to know that there is a small runtime cost for signed numbers.
/// - Enums which implement the `BitfieldEnum` trait in Bondrewd.
/// - Structs or Enums which implement the Bitfield trait in Bondrewd.
///
/// # Struct/Enum/Variant Attributes
///
/// #### Common Attributes
/// These attributes can be used on a struct, enum or a n enum variant. When used with an enum they are
/// defaults for the variants, and each variant can be assigned these attributes as well.
/// - `default_endianness = {"le" or "be"}` Describes a default endianness for primitive fields. as of version
/// `0.3.27` the endianness will default to Little Endianness.
/// [example](#endianness-examples)
/// - `read_from = {"msb0" or "lsb0"}` Defines bit positioning. which end of the byte array to start at.
/// [example](#bit-positioning-examples)
/// - `reverse` Defines that the entire byte array should be read backward (first byte index becomes last
/// byte index). This has no runtime cost. [example](#reverse-example)
///
/// #### Struct/Variant Attributes
/// - `enforce_bytes = {BYTES}` Adds a check that requires total bytes defined by fields to equal provided
/// BYTES. [example](#enforce-bits-examples)
/// - `enforce_bits = {BITS}` Adds a check that requires total bits defined by fields to equal provided
/// BITS. [example](#enforce-bits-examples)
/// - `enforce_full_bytes` Adds a check that requires total bits defined by fields to equal a multiple of 8.
/// [example](#enforce-full-bytes-example)
/// - `fill_bytes = {BYTES}` Will force the output/input byte array size to be the provided SIZE amount of
/// bytes. [example](#fill-bytes-examples)
///
/// #### Enum Attributes
/// - `id_bit_length = {BITS}` Describes the amount of bits bondrewd will use to identify which variant is being stored.
/// [example](#enum-example)
/// - `id_byte_length = {BYTES}` Describes the amount of bytes bondrewd will use to identify which variant is being stored.
///
/// #### Variant Attributes
/// - `variant_id = {ID}` Tell bondrewd the id value to use for the variant.
/// [example](#enum-example).
/// The id can also be defined by a using discriminates [discriminate-example](#enum-with-discriminates).
/// - `invalid` a single Enum Variant can be marked as the "invalid" variant. The invalid variant acts as
/// a catch all for id's that may not be specified. [example](#invalid-enum-variant).
///
/// # Field Attributes
/// #### Common Field Attributes
/// - `bit_length = {BITS}` Define the total amount of bits to use when condensed. [example](#simple-example)
/// - `byte_length = {BYTES}` Define the total amount of bytes to use when condensed. [example](#simple-example)
/// - `endianness = {"le" or "be"}` Define per field endianess. [example](#endianness-examples)
/// - `block_bit_length = {BITS}` Describes a bit length for the entire array dropping lower indexes first.
/// [example](#bitfield-array-examples)
/// - `block_byte_length = {BYTES}` Describes a byte length for the entire array dropping lower indexes
/// first. [example](#bitfield-array-examples)
/// - `element_bit_length = {BITS}` Describes a bit length for each element of an array. (default array
/// type). [example](#bitfield-array-examples)
/// - `element_byte_length = {BYTES}` Describes a byte length for each element of an array. (default array
/// type). [example](#bitfield-array-examples)
/// - `enum_primitive = "u8"` Defines the size of the enum. the `BitfieldEnum` currently only supports u8.
/// [example](#enum-examples)
/// - `struct_size = {SIZE}` Defines the field as a struct which implements the Bitfield trait and the
/// `BYTE_SIZE` const defined in said trait. [example](#bitfield-struct-as-field-examples)
/// - `reserve` Defines that this field should be ignored in from and into bytes functions.
/// [example](#reserve-examples)
///     - Reserve requires the fields type to impl [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html).
/// due to `from_bytes` needed to provided a value.
///
/// #### Enum Variant Field Attributes
/// - `capture_id` Tells Bondrewd to put the value for id in the field on reads, fields
/// with this attribute do NOT get written to the bytes to prevent users from creating improper
/// byte values. [example](#capture-id)
///
/// # Experimental Field Attributes
/// if you decide to use these remember that they have not been exhaustively tested. when using
/// experimental attributes please be careful and report unexpected behavior to our github issues.
/// - `bits = "{RANGE}"` - Define the bit indexes yourself rather than let the proc macro figure
/// it out. using a rust range in quotes. the RANGE must provide a inclusively below and exclusively
/// above bounded range (ex. bits = "0..2" means use bits 0 and 1 but NOT 2).
/// [example](#bits-attribute-example)
/// - `read_only` - Bondrewd will not include `from_bytes` or `into_bytes` logic for the field.
/// - `overlapping_bits = {BITS}` - Tells bondrewd that the provided BITS amount is shared
///  with at least 1 other field and should not be included in the overall structure size.
/// - `redundant` - Tells bondrewd that this field's bits are all shared by at least one other field.
/// Bondrewd will not include the bit length in the structures overall bit length
/// (because they are redundant).
/// [example](#redundant-examples)
///     - Bondrewd will read the assigned bits but will not write.
///     - This behaves exactly as combining the attributes:
///         - `read_only`
///         - `overlapping_bits = {FIELD_BIT_LENGTH}` `FIELD_BIT_LENGTH` being the total amount of bits that the field uses.
///
/// # Simple Example
/// This example is on the front page for bondrewd-derive. Here i will be adding some asserts to show what
/// to expect.
/// I will be defining a data structure with 7 total bytes as:
/// - A boolean field named one will be the first bit.
/// - A floating point field named two will be the next 32 bits. floats must be full sized
/// currently.
/// - A signed integer field named three will be the next 14 bits.
/// - An unsigned integer field named four will be the next 6 bits.
/// - Because these fields do not add up to a number divisible by 8 the last 3 bits will be unused.
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
/// // what we wrote to the bytes NOT the origanal values.
/// assert_eq!(true,reconstructed.one);
/// assert_eq!(5.5,reconstructed.two);
/// assert_eq!(511,reconstructed.three);
/// assert_eq!(0,reconstructed.four);
/// ```
/// # Reverse Example
/// Reverse simply makes Bondrewd index the bytes in the output/input buffers in the opposite order.
/// First index becomes last index and last index becomes the first.
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
/// # Bit Positioning Examples
/// Here Bit positioning will control where bit 0 is. for example if you have a field with 2 bits then
/// 2 fields with 3 bits each, bit positioning will define the direction in which it traverses bit indices,
/// so in our example if 0 is the least significant bit the first field would be the least significant bit
/// in the last index in the byte array. `msb0` is the default.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(read_from = "msb0")]
/// struct ExampleMSB {
///     #[bondrewd(bit_length = 2)]
///     one: u8,
///     #[bondrewd(bit_length = 3)]
///     two: u8,
///     #[bondrewd(bit_length = 3)]
///     three: u8,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(read_from = "lsb0")]
/// struct ExampleLSB {
///     #[bondrewd(bit_length = 2)]
///     one: u8,
///     #[bondrewd(bit_length = 3)]
///     two: u8,
///     #[bondrewd(bit_length = 3)]
///     three: u8,
/// }
///
/// let test_msb = ExampleMSB {
///     one: 0,
///     two: 5,
///     three: 0,
/// };
/// let test_lsb = ExampleLSB {
///     one: 0,
///     two: 5,
///     three: 0,
/// };
/// // in msb0 field one is the first 2 bits followed by field two
/// // then field three is the last 3 bits.
/// assert_eq!(test_msb.into_bytes(), [0b00_101_000]);
/// // in msb0 field three is the first 3 bits followed by field
/// // 2 then field one being the last 2 bits
/// assert_eq!(test_lsb.into_bytes(), [0b000_101_00]);
/// ```
/// When using `reverse` and `read_from` in the same structure:
/// - `lsb0` would begin at the least significant bit in the first byte.
/// - `msb0` would begin at the most significant bit in the last byte.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(read_from = "msb0", reverse)]
/// struct ExampleMSB {
///     #[bondrewd(bit_length = 5)]
///     one: u8,
///     #[bondrewd(bit_length = 4)]
///     two: u8,
///     #[bondrewd(bit_length = 7)]
///     three: u8,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(read_from = "lsb0", reverse)]
/// struct ExampleLSB {
///     #[bondrewd(bit_length = 5)]
///     one: u8,
///     #[bondrewd(bit_length = 4)]
///     two: u8,
///     #[bondrewd(bit_length = 7)]
///     three: u8,
/// }
///
/// let test_msb = ExampleMSB {
///     one: 0,
///     two: u8::MAX,
///     three: 0,
/// };
/// let test_lsb = ExampleLSB {
///     one: 0,
///     two: u8::MAX,
///     three: 0,
/// };
/// // here the 1's belong to feild two. i hope this is understandable.
/// assert_eq!(test_msb.into_bytes(), [0b10000000, 0b00000111]);
/// assert_eq!(test_lsb.into_bytes(), [0b11100000, 0b00000001]);
/// ```
/// # Endianness Examples
/// There are 2 ways to define endianess of fields that require endianness (multi-byte numbers, char, ...)
/// - Default endianness which will give provided endianness to all fields that require endianness but
/// do not have it defined.
/// - Per field endianess which defines the endianness of a particular field.
///
/// Default endianness and per fields endianness can also be used in the same struct
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// // tell bondrewd to default to Big Endian
/// #[bondrewd(default_endianness = "be")]
/// struct SimpleExample {
///     // this field will be given the default endianness
///     one: u16,
///     // here we give the field endianness which means it will not use default endianness.
///     #[bondrewd(endianness = "le")]
///     two: u16,
/// }
///
/// let test = SimpleExample {
///     one: 5,
///     two: 5,
/// };
/// // check that each field are in the correct endianness
/// assert_eq!(test.into_bytes(),[0b00000000, 0b00000101, 0b00000101, 0b00000000]);
/// ```
/// If you define the endianness of all values that require it `default_endianness` is not required.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// struct SimpleExample {
///     #[bondrewd(endianness = "be")]
///     one: u16,
///     #[bondrewd(endianness = "le")]
///     two: u16,
///     // because this field does not use more than 1 byte, endianness is not required.
///     three: bool,
/// }
///
/// let test = SimpleExample {
///     one: 5,
///     two: 5,
///     three: true,
/// };
/// // check that each field are in the correct endianness
/// assert_eq!(test.into_bytes(),[0b00000000, 0b00000101, 0b00000101, 0b00000000, 0b10000000]);
/// ```
/// # Bitfield Struct as Field Examples
/// Inner structs must implement the
/// [`Bitfields`](https://docs.rs/bondrewd/latest/bondrewd/trait.Bitfields.html) trait and be given the
/// `struct_size = {BYTE_SIZE}`, the `BYTE_SIZE` being the number of bytes in the outputs byte array or
/// value in the traits const `BYTE_SIZE`.
/// ```
/// // this struct uses 52 total bits which means the total BYTE_SIZE is 7.
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
///     // structs can also be used in arrays.
///     #[bondrewd(struct_size = 7)]
///     two: [Simple; 2],
/// }
/// ```
/// We can also trim the struct to a bit length, this can be very useful for struct that do not use the
/// full amount of bits available in the byte array. For example if we have a struct that uses 4 bits
/// leaving the remaining 4 bits as unused data, we can make a structure with 2 of the bits structure
/// that still only uses 1 byte.
/// ```
/// // this struct uses 4 total bits which means the total BYTE_SIZE is 1.
/// use bondrewd::*;
/// #[derive(Bitfields, Clone)]
/// #[bondrewd(default_endianness = "be")]
/// struct Simple {
///     #[bondrewd(bit_length = 2)]
///     one: u8,
///     #[bondrewd(bit_length = 2)]
///     two: u8,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be")]
/// struct SimpleWithStruct {
///     #[bondrewd(struct_size = 1, bit_length = 4)]
///     one: Simple,
///     #[bondrewd(struct_size = 1, bit_length = 4)]
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
/// # Bitfield Array Examples
/// There are 2 types of arrays in Bondrewd:
/// - Block Arrays are "bit chucks" that define a total-used-bits amount and will drop bits starting
/// at the lowest index.
/// - Element Arrays treat each element of the array as its own field and requires a per element
/// bit-length.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be")]
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
/// Structures and Enums can also be used in arrays but there are some extra things to consider.
/// - If `bit_length` of the structs or enums needs to be smaller than the output of either `into_bytes` or
/// `into_primitive` then it is recommended to use element arrays.
/// - Block Arrays, in my opinion, shouldn't be used for Structs or Enums. because in the below example
/// if the `compressed_structures` field was to use `block_bit_length = 104` the array would use
/// 48 bits for index 0 and 56 bits for index 1.
/// ```
/// // this struct uses 52 total bits which means the total
/// // BYTE_SIZE is 7.
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be")]
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
/// #[bondrewd(default_endianness = "be", id_bit_length = 2)]
/// enum SimpleEnum {
///     Zero,
///     One,
///     Two,
///     Three,
/// }
///
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be")]
/// struct ArraysWithStructsAndEnums {
///     #[bondrewd(element_bit_length = 8)]
///     four_byte_four_values: [SimpleEnum; 4],
///     // if we use the element_bit_length we can say to only use 2
///     // bits per SimpleEnum, and due to SimpleEnum only needing 2
///     // bits, this could be desirable. means instead of using 4
///     // bytes to store 4 SimpleEnums, we can use 1 byte.
///     #[bondrewd(element_bit_length = 2)]
///     one_byte_four_values: [SimpleEnum; 4],
///     // again if the size doesn't need to change, no array attribute
///     // is needed.
///     #[bondrewd(struct_size = 7)]
///     waste_a_byte: [SimpleStruct; 2],
///     // if we want to compress the 2 struct in the array we can
///     // take advantage of the fact our struct is only using 52 out
///     // of 56 bits in the compressed/byte form by adding
///     // element bit length = 52. this will make the total size of
///     // the 2 structs in compressed/byte form 104 bits instead of
///     // 112.
///     #[bondrewd(struct_size = 7, element_bit_length = 52)]
///     compressed_structures: [SimpleStruct; 2],
/// }
/// ```
/// # Reserve Examples
/// Reserve fields tell Bondrewd to not include logic for reading or writing the field in the from and
/// into bytes functions. Currently only primitive types are supported.
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
/// Reserves do not need to be at the end.
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
/// # Fill Bytes Examples
/// Fill bytes is used here to make the total output byte size 3 bytes. If fill bytes attribute was not
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
/// assert_eq!(3, FilledBytes::BYTE_SIZE);
/// assert_eq!(24, FilledBytes::BIT_SIZE);
/// ```
/// Here im going to compare the example above to the closest alternative using a reserve field:
/// - `FilledBytes` only has 2 field, so only 2 fields are required for instantiation, where as `ReservedBytes`
/// still needs a value for the reserve field despite from/into bytes not using the value anyway.
/// - `ReservedBytes` has 2 extra functions that Filled Bytes does not, `write_reserve` and `read_reserve`.
/// - One more thing to consider is reserve fields are currently confined to primitives, if more than 128
/// reserve bits are required at the end, `fill_bytes` is the only supported way of doing this.
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
/// assert_eq!(3, ReservedBytes::BYTE_SIZE);
/// assert_eq!(24, ReservedBytes::BIT_SIZE);
/// ```
/// # Enforce Bits Examples
/// Enforce Bits/Bytes Main purpose is to act as a compile time check to ensure how many bit you think
/// are being use is the actual amount of bits being used.  
/// Here i have 2 fields with a total defined bit-length of 6, and then an undecorated boolean field. I
/// also have trust issues so i want to verify that the bool is only using 1 bit making the total bit
/// length of the struct 7 bits. Adding `enforce_bits = 7` will force a compiler error if the calculated
/// total bit length is not 7.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", enforce_bits = 7)]
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
/// Here is the same example where i assigned the "incorrect" the `bit_length` of the first field making the
/// total 8 instead of 7.
/// ```compile_fail
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", enforce_bits = 7)]
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
/// assert_eq!(3, FilledBytesEnforced::BYTE_SIZE);
/// assert_eq!(24, FilledBytesEnforced::BIT_SIZE);
/// ```
/// Also note that [`fill_bytes`](#fill-bytes-examples) does NOT effect how `enforce_bytes` works.
/// `enforce_bytes` will check the total bit length before the bits are filled.
///   
/// Here i am telling Bondrewd to make the total bit length 3 bytes using `fill_bytes`.
/// This Example fails to build because only 14 bits are being defined by fields and `enforce_bytes`
/// is telling Bondrewd to expect 24 bits to be used by defined fields.
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
/// ```
/// To fix this we need to make sure our enforcement value is the amount fo bits defined by the fields NOT
/// the expected `FilledBytesEnforced::BYTE_SIZE`.
///   
/// Here is the Correct usage of these two attributes working together.
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
/// assert_eq!(3, FilledBytesEnforced::BYTE_SIZE);
/// // we are enforcing 14 bits but fill_bytes is creating
/// // an imaginary reserve field from bit index 14 to
/// // index 23
/// assert_eq!(24, FilledBytesEnforced::BIT_SIZE);
/// ```
/// # Enforce Full Bytes Example
/// `enforce_full_bytes` adds a check during parsing phase of Bondrewd which will throw an error if the
/// total bits determined from the defined fields is not a multiple of 8. This was included for those
/// like me that get paranoid they entered something in wrong.
/// ```compile_fail
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", enforce_full_bytes)]
/// struct FilledBytesEnforced {
///     #[bondrewd(bit_length = 7)]
///     one: u8,
///     #[bondrewd(bit_length = 7)]
///     two: u8,
/// }
/// ```
/// In this case if we still wanted fields one and two to remain 7 bits we need to add another field
/// to use the remaining 2 bits.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", enforce_full_bytes)]
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
/// # Enum Examples (Deprecated)
/// For enum derive examples goto [`BitfieldEnum` Derive](BitfieldEnum).
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", id_bit_length = 2)]
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
///     #[bondrewd(bit_length = 2)]
///     two: SimpleEnum,
///     #[bondrewd(bit_length = 3)]
///     three: u8,
/// }
/// ```
/// Enums can also be used in [arrays](#bitfield-array-examples)
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", id_bit_length = 2)]
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
///     #[bondrewd(bit_length = 2)]
///     one: Simple,
///     #[bondrewd(element_bit_length = 2)]
///     two: [Simple; 3],
/// }
/// ```
/// # Bits Attribute Example
/// First i will replicate the [Simple Example](#simple-example) to show an equivalent use.
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be")]
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
/// // what we wrote to the bytes NOT the origanal values.
/// assert_eq!(true,reconstructed.one);
/// assert_eq!(5.5,reconstructed.two);
/// assert_eq!(511,reconstructed.three);
/// assert_eq!(0,reconstructed.four);
/// ```
/// # Redundant Examples
/// In this example we will has fields share data. flags in the example will represent a u8 storing
/// multiple boolean flags, but all of the flags within are also fields in the struct. if we mark
/// flags as `redundant` above the boolean flag fields then flags will be `read_only` (effects nothing
/// during an [`into_bytes`] call).
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
/// // what we wrote to the bytes NOT the origanal values.
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
/// we can also have the flags below if we use the `bits` attribute.
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
/// // what we wrote to the bytes NOT the origanal values.
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
/// # Enum Example
/// Because enums can provide a lot of ambiguity there is a requirement that The last variant is
/// always considered the "Invalid Variant", which simply means that it will be a
/// catch-all in the match statement for the generated `Bitfields::from_bytes()` function.
/// See [Generated From Bytes](#generated-from-bytes) below.
///
/// ```
/// use bondrewd::*;
///
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", id_bit_length = 2, enforce_bytes = 3)]
///
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
///     #[bondrewd(variant_id = 0)]
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
/// #### Generated From Bytes
/// ```
/// enum Thing {
///     One {
///         a: u16
///     },
///     Two {
///         a: u16,
///         b: u8
///     },
///     Three {
///         d: u8,
///         e: u16
///     },
///     Idk,
/// }
/// impl bondrewd::Bitfields<3usize> for Thing {
///     const BIT_SIZE: usize = 24usize;
///     fn into_bytes(self) -> [u8; 3usize] {
///         let mut output_byte_buffer = [0u8; 3usize];
///         match self {
///             Self::One { a } => {
///                 Self::write_id(&mut output_byte_buffer, 1);
///                 Self::write_one_a(&mut output_byte_buffer, a);
///             }
///             Self::Two { a, b } => {
///                 Self::write_id(&mut output_byte_buffer, 2);
///                 Self::write_two_a(&mut output_byte_buffer, a);
///                 Self::write_two_b(&mut output_byte_buffer, b);
///             }
///             Self::Three { d, e } => {
///                 Self::write_id(&mut output_byte_buffer, 3);
///                 Self::write_three_d(&mut output_byte_buffer, d);
///                 Self::write_three_e(&mut output_byte_buffer, e);
///             }
///             Self::Idk {} => {
///                 Self::write_id(&mut output_byte_buffer, 0);
///             }
///         }
///         output_byte_buffer
///     }
///     fn from_bytes(mut input_byte_buffer: [u8; 3usize]) -> Self {
///         let id = Self::read_id(&input_byte_buffer);
///         match id {
///             1 => {
///                 let a = Self::read_one_a(&input_byte_buffer);
///                 Self::One { a }
///             }
///             2 => {
///                 let a = Self::read_two_a(&input_byte_buffer);
///                 let b = Self::read_two_b(&input_byte_buffer);
///                 Self::Two { a, b }
///             }
///             3 => {
///                 let d = Self::read_three_d(&input_byte_buffer);
///                 let e = Self::read_three_e(&input_byte_buffer);
///                 Self::Three { d, e }
///             }
///             _ => Self::Idk,
///         }
///     }
///     }
///     impl Thing {
///     #[inline]
///     ///Reads bits 0 through 1 within `input_byte_buffer`, getting the `id` field of a `Thing` in bitfield form.
///     pub fn read_id(input_byte_buffer: &[u8; 3usize]) -> u8 {
///         ((input_byte_buffer[0usize] & 192u8) >> 6usize) as u8
///     }
///     #[inline]
///     ///Reads bits 2 through 17 within `input_byte_buffer`, getting the `one_a` field of a `One` in bitfield form.
///     pub fn read_one_a(input_byte_buffer: &[u8; 3usize]) -> u16 {
///         u16::from_be_bytes({
///                 let mut a_bytes: [u8; 2usize] = [0u8; 2usize];
///                 a_bytes[0usize] |= input_byte_buffer[0usize] & 63u8;
///                 a_bytes[1usize] |= input_byte_buffer[1usize];
///                 a_bytes[0] |= input_byte_buffer[2usize] & 192u8;
///                 a_bytes
///             })
///             .rotate_left(2u32)
///     }
///     #[inline]
///     ///Reads bits 18 through 23 within `input_byte_buffer`, getting the `one_fill_bits` field of a `One` in bitfield form.
///     pub fn read_one_fill_bits(input_byte_buffer: &[u8; 3usize]) -> [u8; 1usize] {
///         [{ ((input_byte_buffer[2usize] & 63u8) >> 0usize) as u8 }]
///     }
///     #[inline]
///     ///Reads bits 2 through 17 within `input_byte_buffer`, getting the `two_a` field of a `Two` in bitfield form.
///     pub fn read_two_a(input_byte_buffer: &[u8; 3usize]) -> u16 {
///         u16::from_be_bytes({
///                 let mut a_bytes: [u8; 2usize] = [0u8; 2usize];
///                 a_bytes[0usize] |= input_byte_buffer[0usize] & 63u8;
///                 a_bytes[1usize] |= input_byte_buffer[1usize];
///                 a_bytes[0] |= input_byte_buffer[2usize] & 192u8;
///                 a_bytes
///             })
///             .rotate_left(2u32)
///     }
///     #[inline]
///     ///Reads bits 18 through 23 within `input_byte_buffer`, getting the `two_b` field of a `Two` in bitfield form.
///     pub fn read_two_b(input_byte_buffer: &[u8; 3usize]) -> u8 {
///         ((input_byte_buffer[2usize] & 63u8) >> 0usize) as u8
///     }
///     #[inline]
///     ///Reads bits 2 through 8 within `input_byte_buffer`, getting the `three_d` field of a `Three` in bitfield form.
///     pub fn read_three_d(input_byte_buffer: &[u8; 3usize]) -> u8 {
///         u8::from_be_bytes({
///                 let mut d_bytes: [u8; 1usize] = [0u8; 1usize];
///                 d_bytes[0usize] |= input_byte_buffer[0usize] & 63u8;
///                 d_bytes[0] |= input_byte_buffer[1usize] & 128u8;
///                 d_bytes
///             })
///             .rotate_left(1u32)
///     }
///     #[inline]
///     ///Reads bits 9 through 23 within `input_byte_buffer`, getting the `three_e` field of a `Three` in bitfield form.
///     pub fn read_three_e(input_byte_buffer: &[u8; 3usize]) -> u16 {
///         u16::from_be_bytes({
///             let mut e_bytes: [u8; 2usize] = [0u8; 2usize];
///             e_bytes[0usize] |= input_byte_buffer[1usize] & 127u8;
///             e_bytes[1usize] |= input_byte_buffer[2usize];
///             e_bytes
///         })
///     }
///     #[inline]
///     ///Reads bits 2 through 23 within `input_byte_buffer`, getting the `idk_fill_bits` field of a `Idk` in bitfield form.
///     pub fn read_idk_fill_bits(input_byte_buffer: &[u8; 3usize]) -> [u8; 3usize] {
///         [
///             { ((input_byte_buffer[0usize] & 63u8) >> 0usize) as u8 },
///             { ((input_byte_buffer[1usize] & 255u8) >> 0usize) as u8 },
///             { ((input_byte_buffer[2usize] & 255u8) >> 0usize) as u8 },
///         ]
///     }
///     #[inline]
///     ///Writes to bits 0 through 1 within `output_byte_buffer`, setting the `id` field of a `Thing` in bitfield form.
///     pub fn write_id(output_byte_buffer: &mut [u8; 3usize], mut id: u8) {
///         output_byte_buffer[0usize] &= 63u8;
///         output_byte_buffer[0usize] |= ((id as u8) << 6usize) & 192u8;
///     }
///     #[inline]
///     ///Writes to bits 2 through 17 within `output_byte_buffer`, setting the `one_a` field of a `One` in bitfield form.
///     pub fn write_one_a(output_byte_buffer: &mut [u8; 3usize], mut a: u16) {
///         output_byte_buffer[0usize] &= 192u8;
///         output_byte_buffer[1usize] &= 0u8;
///         output_byte_buffer[2usize] &= 63u8;
///         let a_bytes = (a.rotate_right(2u32)).to_be_bytes();
///         output_byte_buffer[0usize] |= a_bytes[0usize] & 63u8;
///         output_byte_buffer[1usize] |= a_bytes[1usize];
///         output_byte_buffer[2usize] |= a_bytes[0] & 192u8;
///     }
///     #[inline]
///     ///Writes to bits 2 through 17 within `output_byte_buffer`, setting the `two_a` field of a `Two` in bitfield form.
///     pub fn write_two_a(output_byte_buffer: &mut [u8; 3usize], mut a: u16) {
///         output_byte_buffer[0usize] &= 192u8;
///         output_byte_buffer[1usize] &= 0u8;
///         output_byte_buffer[2usize] &= 63u8;
///         let a_bytes = (a.rotate_right(2u32)).to_be_bytes();
///         output_byte_buffer[0usize] |= a_bytes[0usize] & 63u8;
///         output_byte_buffer[1usize] |= a_bytes[1usize];
///         output_byte_buffer[2usize] |= a_bytes[0] & 192u8;
///     }
///     #[inline]
///     ///Writes to bits 18 through 23 within `output_byte_buffer`, setting the `two_b` field of a `Two` in bitfield form.
///     pub fn write_two_b(output_byte_buffer: &mut [u8; 3usize], mut b: u8) {
///         output_byte_buffer[2usize] &= 192u8;
///         output_byte_buffer[2usize] |= ((b as u8) << 0usize) & 63u8;
///     }
///     #[inline]
///     ///Writes to bits 2 through 8 within `output_byte_buffer`, setting the `three_d` field of a `Three` in bitfield form.
///     pub fn write_three_d(output_byte_buffer: &mut [u8; 3usize], mut d: u8) {
///         output_byte_buffer[0usize] &= 192u8;
///         output_byte_buffer[1usize] &= 127u8;
///         let d_bytes = (d.rotate_right(1u32)).to_be_bytes();
///         output_byte_buffer[0usize] |= d_bytes[0usize] & 63u8;
///         output_byte_buffer[1usize] |= d_bytes[0] & 128u8;
///     }
///     #[inline]
///     ///Writes to bits 9 through 23 within `output_byte_buffer`, setting the `three_e` field of a `Three` in bitfield form.
///     pub fn write_three_e(output_byte_buffer: &mut [u8; 3usize], mut e: u16) {
///         output_byte_buffer[1usize] &= 128u8;
///         output_byte_buffer[2usize] &= 0u8;
///         let e_bytes = e.to_be_bytes();
///         output_byte_buffer[1usize] |= e_bytes[0usize] & 127u8;
///         output_byte_buffer[2usize] |= e_bytes[1usize];
///     }
/// }
/// ```
/// # Enum With Discriminates
/// In this example i have create Variant's `Three`, `Two`, `One`, and `Idk` variants. The variants with
/// numbers as their names are listed from highest to lowest to show case an easy issue you may run into.
///
/// #### Issue You May Run Into
/// Because i am:
/// - Setting the last variant's, "Idk" variant, id to `0`,
/// - Setting the first variant's, `Three` variant, id to `3`,
/// - Setting the third variant's, `Two` variant, id to `2`,
/// - And variant `One` does not have a defined id.
///
///
/// variant `One` will be assigned an id of the next lowest value not already used, if more than 1 was undefined
/// the assignment would go from top to bottom. This happens internally in bondrewd for its code generation
/// but the `#[repr(u8)]` attribute assigns values for if you want to represent the variant as a number,
/// and you should be aware `repr` does not look forward or backward for used numbers, meaning you will
/// get an error from `repr` if you:
/// - Remove the first variant's, `Three`, id assignment of `3`. The first variant will be assigned
///     zero regardless of the last variant being manually assigned that number already.
/// - Change the second variant's, `One`, id assignment of `1` to `2`. `repr` will assume that this should
///     be that last variant's value plus one which is `3` and already used.
/// #### Discriminate Example
/// ```
/// use bondrewd::*;
///
/// #[derive(Bitfields)]
/// #[repr(u8)]
/// #[bondrewd(default_endianness = "be", id_bit_length = 2, enforce_bytes = 3)]
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
/// ```
/// #### Capture Id
/// ```
/// use bondrewd::*;
///
/// #[derive(Bitfields)]
/// #[repr(u8)]
/// #[bondrewd(default_endianness = "be", id_bit_length = 2, enforce_bits = 18)]
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
/// #### Invalid Enum Variant
/// For this example we will be show casing why enums do not panic on an invalid case. The generative code
/// for enums always has an invalid variant even when all possible values have a variant. If an Id value
/// does not have an associated variant, `Bitfield::from_bytes` will return the "invalid" variant. This
/// can be combine with [capture-id](#capture-id) to enable nice error handling/reporting.
///
/// > the invalid variant is always the
/// [catch-all](https://doc.rust-lang.org/rust-by-example/flow_control/match.html) in the generated
/// code's match statements
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
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", id_bit_length = 2)]
/// enum Thing {
///     Zero, // value of 0
///     One, // value of 1
///     Invalid, // value of 2 or 3
/// }
/// ```
///
/// > Note that when no id values are specified they will be assigned automatically starting at zero,
/// incrementing 1 for each variant.
///
/// If for some reason the last variant should not be the catch all you can specify a specific variant.
/// So for this next example:
/// - a value of 0 as the id would result in a `Thing::Zero` variant
/// - a value of 1 or 3 as the id would result in a `Thing::Invalid` variant
/// - a value of 2 as the id would result in a `Thing::Two` variant
/// ```
/// use bondrewd::*;
///
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "be", id_bit_length = 2)]
/// enum Thing {
///     Zero, // value of 0
///     Invalid, // value of 1 or 3
///     Two, // value of 2
/// }
/// ```
#[proc_macro_derive(Bitfields, attributes(bondrewd,))]
#[allow(clippy::too_many_lines)]
pub fn derive_bitfields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // parse the input into a StructInfo which contains all the information we
    // along with some helpful structures to generate our Bitfield code.
    let struct_info = match ObjectInfo::parse(&input) {
        Ok(parsed_struct) => parsed_struct,
        Err(err) => {
            return TokenStream::from(err.to_compile_error());
        }
    };
    // println!("{:?}", struct_info);
    // get the struct size and name so we can use them in a quote.
    let struct_size = struct_info.total_bytes();
    let struct_name = struct_info.name();

    let dyn_fns: bool;
    let setters: bool;
    let hex: bool;
    #[cfg(not(feature = "dyn_fns"))]
    {
        dyn_fns = false;
    }
    #[cfg(feature = "dyn_fns")]
    {
        dyn_fns = true;
    }
    #[cfg(not(feature = "setters"))]
    {
        setters = false;
    }
    #[cfg(feature = "setters")]
    {
        setters = true;
    }
    #[cfg(feature = "hex_fns")]
    {
        hex = true;
    }
    #[cfg(not(feature = "hex_fns"))]
    {
        hex = false;
    }
    match struct_info {
        ObjectInfo::Struct(struct_info) => {
            // get a list of all fields into_bytes logic which puts there bytes into an array called
            // output_byte_buffer.
            let fields_into_bytes =
                match create_into_bytes_field_quotes_struct(&struct_info, dyn_fns) {
                    Ok(ftb) => ftb,
                    Err(err) => return TokenStream::from(err.to_compile_error()),
                };
            let fields_from_bytes = match create_from_bytes_field_quotes(&struct_info, dyn_fns) {
                Ok(ffb) => ffb,
                Err(err) => return TokenStream::from(err.to_compile_error()),
            };
            // combine all of the write_ function quotes separated by newlines
            let into_bytes_quote = fields_into_bytes.into_bytes_fn;
            let mut set_quotes = fields_into_bytes.set_field_fns;

            if let Some(set_slice_quote) = fields_into_bytes.set_slice_field_fns {
                set_quotes = quote! {
                    #set_quotes
                    #set_slice_quote
                }
            }

            // combine all of the read_ function quotes separated by newlines
            let from_bytes_quote = fields_from_bytes.from_bytes_fn;
            let mut peek_quotes = fields_from_bytes.peek_field_fns;

            if let Some(peek_slice_quote) = fields_from_bytes.peek_slice_field_fns {
                peek_quotes = quote! {
                    #peek_quotes
                    #peek_slice_quote
                }
            }
            // TODO get setters working fully.
            // get the setters, functions that set a field disallowing numbers
            // outside of the range the Bitfield.
            let setters_quote = if setters {
                match structs::struct_fns::create_setters_quotes(&struct_info) {
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
            let hex_size = struct_size * 2;
            let mut hex_fns_quote = if hex {
                quote! {
                    impl bondrewd::BitfieldHex<#hex_size, #struct_size> for #struct_name {}
                }
            } else {
                quote! {}
            };
            if dyn_fns && hex {
                hex_fns_quote = quote! {
                    #hex_fns_quote
                    impl bondrewd::BitfieldHexDyn<#hex_size, #struct_size> for #struct_name {}
                };
            }

            // get the bit size of the entire set of fields to fill in trait requirement.
            let bit_size = struct_info.total_bits();

            // put it all together.
            // to_bytes_quote will put all of the fields in self into a array called output_byte_buffer.
            // so for into_bytes all we need is the fn declaration, the output_byte_buffer, and to return
            // that buffer.
            // from_bytes is essentially the same minus a variable because input_byte_buffer is the input.
            // slap peek quotes inside a impl block at the end and we good to go
            let to_bytes_quote = quote! {
                impl bondrewd::Bitfields<#struct_size> for #struct_name {
                    const BIT_SIZE: usize = #bit_size;
                    #into_bytes_quote
                    #from_bytes_quote
                }
                #getter_setters_quotes
                #hex_fns_quote
            };

            if dyn_fns {
                let from_vec_quote = fields_from_bytes.from_slice_field_fns;
                let vis = struct_info.vis;
                let checked_ident = format_ident!("{}Checked", &struct_name);
                let checked_mut_ident = format_ident!("{}CheckedMut", &struct_name);
                let unchecked_functions = fields_from_bytes.peek_slice_field_unchecked_fns;
                let unchecked_mut_functions = fields_into_bytes.set_slice_field_unchecked_fns;
                let comment = format!("A Structure which provides functions for getting the fields of a [{struct_name}] in its bitfield form.");
                let comment_mut = format!("A Structure which provides functions for getting and setting the fields of a [{struct_name}] in its bitfield form.");
                let unchecked_comment = format!("Panics if resulting `{checked_ident}` does not contain enough bytes to read a field that is attempted to be read.");
                let unchecked_comment_mut = format!("Panics if resulting `{checked_mut_ident}` does not contain enough bytes to read a field that is attempted to be read or written.");
                let to_bytes_quote = quote! {
                    #to_bytes_quote
                    #[doc = #comment]
                    #vis struct #checked_ident<'a> {
                        buffer: &'a [u8],
                    }
                    impl<'a> #checked_ident<'a> {
                        #unchecked_functions
                        #[doc = #unchecked_comment]
                        pub fn from_unchecked_slice(data: &'a [u8]) -> Self {
                            Self{
                                buffer: data
                            }
                        }
                    }
                    #[doc = #comment_mut]
                    #vis struct #checked_mut_ident<'a> {
                        buffer: &'a mut [u8],
                    }
                    impl<'a> #checked_mut_ident<'a> {
                        #unchecked_functions
                        #unchecked_mut_functions
                        #[doc = #unchecked_comment_mut]
                        pub fn from_unchecked_slice(data: &'a mut [u8]) -> Self {
                            Self{
                                buffer: data
                            }
                        }
                    }
                    impl bondrewd::BitfieldsDyn<#struct_size> for #struct_name {
                        #from_vec_quote
                    }
                };
                TokenStream::from(to_bytes_quote)
            } else {
                TokenStream::from(to_bytes_quote)
            }
        }
        ObjectInfo::Enum(enum_info) => {
            // let dyn_fns = false;
            // get a list of all fields into_bytes logic which puts there bytes into an array called
            // output_byte_buffer.
            let fields_into_bytes = match create_into_bytes_field_quotes_enum(&enum_info, dyn_fns) {
                Ok(ftb) => ftb,
                Err(err) => return TokenStream::from(err.to_compile_error()),
            };
            let fields_from_bytes = match create_from_bytes_field_quotes_enum(&enum_info, dyn_fns) {
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
            let struct_size = enum_info.total_bytes();
            let hex_size = struct_size * 2;
            let mut hex_fns_quote = if hex {
                quote! {
                    impl bondrewd::BitfieldHex<#hex_size, #struct_size> for #struct_name {}
                }
            } else {
                quote! {}
            };
            if dyn_fns && hex {
                hex_fns_quote = quote! {
                    #hex_fns_quote
                    impl bondrewd::BitfieldHexDyn<#hex_size, #struct_size> for #struct_name {}
                };
            }

            // get the bit size of the entire set of fields to fill in trait requirement.
            let bit_size = enum_info.total_bits();

            // put it all together.
            // to_bytes_quote will put all of the fields in self into a array called output_byte_buffer.
            // so for into_bytes all we need is the fn declaration, the output_byte_buffer, and to return
            // that buffer.
            // from_bytes is essentially the same minus a variable because input_byte_buffer is the input.
            // slap peek quotes inside a impl block at the end and we good to go
            let to_bytes_quote = quote! {
                impl bondrewd::Bitfields<#struct_size> for #struct_name {
                    const BIT_SIZE: usize = #bit_size;
                    #into_bytes_quote
                    #from_bytes_quote
                }
                #getter_setters_quotes
                #hex_fns_quote
            };
            if dyn_fns {
                let from_vec_quote = fields_from_bytes.from_slice_field_fns;
                let vis = &enum_info.vis;
                let checked_ident = format_ident!("{}Checked", &struct_name);
                let checked_mut_ident = format_ident!("{}CheckedMut", &struct_name);
                let unchecked_functions = fields_from_bytes.peek_slice_field_unchecked_fns;
                let unchecked_mut_functions = fields_into_bytes.set_slice_field_unchecked_fns;
                let comment = format!("A Structure which provides functions for getting the fields of a [{struct_name}] in its bitfield form.");
                let comment_mut = format!("A Structure which provides functions for getting and setting the fields of a [{struct_name}] in its bitfield form.");
                let unchecked_comment = format!("Panics if resulting `{checked_ident}` does not contain enough bytes to read a field that is attempted to be read.");
                let unchecked_comment_mut = format!("Panics if resulting `{checked_mut_ident}` does not contain enough bytes to read a field that is attempted to be read or written.");
                let to_bytes_quote = quote! {
                    #to_bytes_quote
                    #[doc = #comment]
                    #vis struct #checked_ident<'a> {
                        buffer: &'a [u8],
                    }
                    impl<'a> #checked_ident<'a> {
                        #unchecked_functions
                        #[doc = #unchecked_comment]
                        pub fn from_unchecked_slice(data: &'a [u8]) -> Self {
                            Self{
                                buffer: data
                            }
                        }
                    }
                    #[doc = #comment_mut]
                    #vis struct #checked_mut_ident<'a> {
                        buffer: &'a mut [u8],
                    }
                    impl<'a> #checked_mut_ident<'a> {
                        #unchecked_functions
                        #unchecked_mut_functions
                        #[doc = #unchecked_comment_mut]
                        pub fn from_unchecked_slice(data: &'a mut [u8]) -> Self {
                            Self{
                                buffer: data
                            }
                        }
                    }
                    impl bondrewd::BitfieldsDyn<#struct_size> for #struct_name {
                        #from_vec_quote
                    }
                };
                #[cfg(feature = "part_eq_enums")]
                let id_ident = match enum_info.id_ident() {
                    Ok(ii) => ii,
                    Err(err) => {
                        return err.to_compile_error().into();
                    }
                };
                #[cfg(feature = "part_eq_enums")]
                let to_bytes_quote = quote! {
                    #to_bytes_quote
                    impl PartialEq<#id_ident> for #struct_name {
                        fn eq(&self, other: &#id_ident) -> bool {
                            self.id() == *other
                        }
                    }
                };
                TokenStream::from(to_bytes_quote)
            } else {
                TokenStream::from(to_bytes_quote)
            }
        }
    }
}

/// Generates an implementation of [`bondrewd::BitfieldEnum`] trait.
///   
/// Important Note: u8 is the only primitive type i have tested. My newest code should be able to handle
/// all primitive types but, to reiterate, i have NOT tested any primitive type other than u8.
///
/// # Features
/// - Generates code for the `BitfieldEnum` trait which allows an enum to be used by Bitfield structs.
/// - Literal values. [example](#literal-example)
/// - Automatic Value Assignment for non-literal variants. Variants are assigned values starting from 0
/// incrementing by 1 skipping values taken by literal definitions (That means you can mix and match
/// inferred values a code defined literal values). [example](#typical-example)
/// - Catch Variants
///     - Catch All variant is used to insure that Results are not needed. Catch all will generate a
///     `_ => {..}` match arm so that enums don't need to have as many variants as there are values in
///     the defined primitive. Catch all can be defined with a `#[bondrewd_enum(invalid)]` attribute or last variant will
///     Automatically become a catch all if no Catch is defined. [example](#custom-catch-all-example)
///     - Catch Value is a variant that will store values that don't match the reset of the variants.
///     using a Catch Value is as simple as making a variant with a primitive value (if the `bondrewd_enum`
///     attribute is present the primitive types must match). [example](#catch-value-example)
///
/// # Other Features
/// - Support for implementation of [`std::cmp::PartialEq`] for the given primitive (currently only u8)
///
/// # Typical Example
/// Here i am letting the Derive do all of the work. The primitive type will be assumed to be u8 because
/// there are less than 256 variants. Variants that do not define a value will be assigned a value
/// starting with the lowest available value. Also due to the catch all system we can ignore the fact
/// i have not covered all 255 values of a u8 because the last Variant, `SimpleEnum::Three` is this example,
/// will be used a a default to insure not errors can occur.
/// ```
/// use bondrewd::BitfieldEnum;
/// #[derive(BitfieldEnum, PartialEq, Debug)]
/// enum SimpleEnum {
///     Zero, // assigned value 0
///     One,  // assigned value 1
///     Two,  // assigned value 2
///     Three,// assigned value 3
/// }
///
/// assert_eq!(SimpleEnum::Zero.into_primitive(), 0);
/// assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(0));
/// assert_eq!(SimpleEnum::One.into_primitive(), 1);
/// assert_eq!(SimpleEnum::One, SimpleEnum::from_primitive(1));
/// assert_eq!(SimpleEnum::Two.into_primitive(), 2);
/// assert_eq!(SimpleEnum::Two, SimpleEnum::from_primitive(2));
/// assert_eq!(SimpleEnum::Three.into_primitive(), 3);
/// for i in 3..=u8::MAX {
///     assert_eq!(SimpleEnum::Three, SimpleEnum::from_primitive(i));
/// }
///
/// ```
/// If you do not want the last variant to be a
/// catch all you must either:
/// - Cover all possible values in the primitive type with a variant each.
/// - Mark the variant you would like to be the catch all [example](#custom-catch-all-example).
/// - Add a catch primitive variant [example](#catch-value-example)
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
/// assert_eq!(SimpleEnum::Life.into_primitive(), 42);
/// assert_eq!(SimpleEnum::Life, SimpleEnum::from_primitive(42));
/// assert_eq!(SimpleEnum::Min.into_primitive(), 0);
/// assert_eq!(SimpleEnum::Min, SimpleEnum::from_primitive(0));
/// assert_eq!(SimpleEnum::U8Max.into_primitive(), 255);
/// assert_eq!(SimpleEnum::U8Max, SimpleEnum::from_primitive(255));
/// assert_eq!(SimpleEnum::Unlucky.into_primitive(), 13);
/// // check all values not defined and 13 get detected as Unlucky
/// for i in 1..42 {
///     assert_eq!(SimpleEnum::Unlucky, SimpleEnum::from_primitive(i));
/// }
/// for i in 43..u8::MAX {
///     assert_eq!(SimpleEnum::Unlucky, SimpleEnum::from_primitive(i));
/// }
/// ```
/// # Custom Catch All Example
/// If you don't decorate the Enum at all the last variant will be assumed to be an Invalid variant. This
/// means if the input value doesn't match any defined value we can use the Invalid variant as a default.
/// ```
/// use bondrewd::BitfieldEnum;
/// #[derive(BitfieldEnum, PartialEq, Debug)]
/// enum SimpleEnum {
///     Zero, // assigned 0
///     One, // assigned 1
///     Two, // assigned 2
///     Three, // assigned 3 and catches invalid values
/// }
///
/// assert_eq!(SimpleEnum::Zero.into_primitive(), 0);
/// assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(0));
/// assert_eq!(SimpleEnum::One.into_primitive(), 1);
/// assert_eq!(SimpleEnum::One, SimpleEnum::from_primitive(1));
/// assert_eq!(SimpleEnum::Two.into_primitive(), 2);
/// assert_eq!(SimpleEnum::Two, SimpleEnum::from_primitive(2));
/// assert_eq!(SimpleEnum::Three.into_primitive(), 3);
/// assert_eq!(SimpleEnum::Three, SimpleEnum::from_primitive(3));
/// // remaining possible values are caught as One.
/// for i in 4..=u8::MAX {
///     assert_eq!(SimpleEnum::Three, SimpleEnum::from_primitive(i));
/// }
/// ```
/// This example shows that we can mark any variant as the catch all variant. In this case Bondrewd will
/// give `SimpleEnum::One` the value of 1 and make One catch all values not defined because
/// of the invalid attribute.
/// ```
/// use bondrewd::BitfieldEnum;
/// #[derive(BitfieldEnum, PartialEq, Debug)]
/// enum SimpleEnum {
///     Zero, // assigned 0
///     #[bondrewd_enum(invalid)]
///     One, // assigned 1 and catches invalid values
///     Two, // assigned 2
///     Three, // assigned 3
/// }
///
/// assert_eq!(SimpleEnum::Zero.into_primitive(), 0);
/// assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(0));
/// assert_eq!(SimpleEnum::One.into_primitive(), 1);
/// assert_eq!(SimpleEnum::One, SimpleEnum::from_primitive(1));
/// assert_eq!(SimpleEnum::Two.into_primitive(), 2);
/// assert_eq!(SimpleEnum::Two, SimpleEnum::from_primitive(2));
/// assert_eq!(SimpleEnum::Three.into_primitive(), 3);
/// assert_eq!(SimpleEnum::Three, SimpleEnum::from_primitive(3));
/// // remaining possible values are caught as One.
/// for i in 4..=u8::MAX {
///     assert_eq!(SimpleEnum::One, SimpleEnum::from_primitive(i));
/// }
/// ```
/// # Catch Value Example
/// In some cases we might need to know what the invalid value passed into [`from_primitive`](bondrewd::BitfieldEnum::from_primitive) actually was. In
/// my own code there is an enum field that gets encrypted and can become pretty much any value and cause
/// panics in the library i used before writing Bondrewd. To fix this Bondrewd offers the ability to make 1
/// variant a tuple or struct variant with exactly one field which must be the primitive type the enum
/// gets converted into/from, than the variant values not covered will be stored in the variants field.
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
/// assert_eq!(SimpleEnum::Zero.into_primitive(), 0);
/// assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(0));
/// assert_eq!(SimpleEnum::One.into_primitive(), 1);
/// assert_eq!(SimpleEnum::One, SimpleEnum::from_primitive(1));
/// assert_eq!(SimpleEnum::Two.into_primitive(), 2);
/// assert_eq!(SimpleEnum::Two, SimpleEnum::from_primitive(2));
/// for i in 3..=u8::MAX {
///     assert_eq!(SimpleEnum::Three(i), SimpleEnum::from_primitive(i));
/// }
/// ```
/// # Complex Example
/// Here i just want to show that Literals, Auto Value Assignment, and Invalid catch all can all be used
/// together. As you might expect Catch Primitive can not have a Literal value because it stores a value.
/// Here we expect:
/// - `SimpleEnum::Nine` = 9,
/// - `SimpleEnum::One`  = 1,
/// - `SimpleEnum::Zero` = 0 and accept 3, 4, 6, 7, 8, and 10-255 in [`from_primitive`](bondrewd::BitfieldEnum::from_primitive),
/// - `SimpleEnum::Five` = 5,
/// - `SimpleEnum::Two`  = 2,
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
/// assert_eq!(SimpleEnum::Nine.into_primitive(), 9);
/// assert_eq!(SimpleEnum::Nine, SimpleEnum::from_primitive(9));
/// assert_eq!(SimpleEnum::One.into_primitive(), 1);
/// assert_eq!(SimpleEnum::One, SimpleEnum::from_primitive(1));
/// assert_eq!(SimpleEnum::Zero.into_primitive(), 0);
/// assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(0));
/// assert_eq!(SimpleEnum::Five.into_primitive(), 5);
/// assert_eq!(SimpleEnum::Five, SimpleEnum::from_primitive(5));
/// assert_eq!(SimpleEnum::Two.into_primitive(), 2);
/// assert_eq!(SimpleEnum::Two, SimpleEnum::from_primitive(2));
/// // Invalid tests
/// assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(3));
/// assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(4));
/// assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(6));
/// assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(7));
/// assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(8));
/// for i in 10..=u8::MAX {
///     assert_eq!(SimpleEnum::Zero, SimpleEnum::from_primitive(i));
/// }
/// ```
#[deprecated(
    since = "0.3.25",
    note = "please use `Bitfields` instead of `BitfieldEnum`"
)]
#[proc_macro_derive(BitfieldEnum, attributes(bondrewd_enum))]
pub fn derive_bondrewd_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_info = match EnumInfo::parse(&input) {
        Ok(parsed_enum) => parsed_enum,
        Err(err) => {
            return TokenStream::from(err.to_compile_error());
        }
    };
    let into = match enums::into_bytes::generate_enum_into_bytes_fn(&enum_info) {
        Ok(i) => i,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };
    let from = match enums::from_bytes::generate_enum_from_bytes_fn(&enum_info) {
        Ok(f) => f,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };
    let partial_eq = enums::partial_eq::generate_partial_eq_fn(&enum_info);
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
