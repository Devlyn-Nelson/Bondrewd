//! This file is an effort to merge from and into bytes, which is being delayed for now.
mod from;
mod into;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::common::{field::Info as FieldInfo, r#struct::Info as StructInfo, EndiannessMode};

/// Returns a u8 mask with provided `num` amount of 1's on the left side (most significant bit)
pub fn get_left_and_mask(num: usize) -> u8 {
    match num {
        8 => 0b1111_1111,
        7 => 0b1111_1110,
        6 => 0b1111_1100,
        5 => 0b1111_1000,
        4 => 0b1111_0000,
        3 => 0b1110_0000,
        2 => 0b1100_0000,
        1 => 0b1000_0000,
        _ => 0b0000_0000,
    }
}

/// Returns a u8 mask with provided `num` amount of 1's on the right side (least significant bit)
pub fn get_right_and_mask(num: usize) -> u8 {
    match num {
        8 => 0b1111_1111,
        7 => 0b0111_1111,
        6 => 0b0011_1111,
        5 => 0b0001_1111,
        4 => 0b0000_1111,
        3 => 0b0000_0111,
        2 => 0b0000_0011,
        1 => 0b0000_0001,
        _ => 0b0000_0000,
    }
}

/// calculate the starting bit index for a field.
///
/// Returns the index of the byte the first bits of the field
///
/// # Arguments
/// * `amount_of_bits` - amount of bits the field will be after `into_bytes`.
/// * `right_rotation` - amount of bit Rotations to preform on the field. Note if rotation is not needed
///                         to retain all used bits then a shift could be used.
/// * `last_index` - total struct bytes size minus 1.
#[inline]
#[allow(
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
pub fn get_be_starting_index(
    amount_of_bits: usize,
    right_rotation: i8,
    last_index: usize,
) -> Result<usize, String> {
    let first = ((amount_of_bits as f64 - right_rotation as f64) / 8.0f64).ceil() as usize;
    if last_index < first {
        Err("Failed getting the starting index for big endianness, field's type doesn't fix the bit size".to_string())
    } else {
        Ok(last_index - first)
    }
}

/// Stores [`TokenStream`] that contain the access (write/read/clear) code for a field.
pub struct GeneratedQuotes {
    read: proc_macro2::TokenStream,
    write: proc_macro2::TokenStream,
    zero: proc_macro2::TokenStream,
}
impl GeneratedQuotes {
    /// Returns the quote that reads a value from bytes
    pub fn read(&self) -> &TokenStream {
        &self.read
    }
    /// Returns the quote that write a value to bytes
    pub fn write(&self) -> &TokenStream {
        &self.write
    }
    /// Returns the quote that set the bytes this field are in to zero. (clears the bits so writes can work on dirty set of bits that already had a value)
    pub fn zero(&self) -> &TokenStream {
        &self.zero
    }
}

pub struct BigQuoteInfo {
    pub right_shift: i8,
    pub first_bit_mask: u8,
    pub last_bit_mask: u8,
    pub bits_in_last_byte: usize,
}
impl From<&QuoteInfo> for BigQuoteInfo {
    fn from(qi: &QuoteInfo) -> Self {
        let bits_in_last_byte = (qi.amount_of_bits() - qi.available_bits_in_first_byte()) % 8;
        // how many times to shift the number right.
        // NOTE if negative shift left.
        // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        #[allow(clippy::cast_possible_truncation)]
        let mut right_shift: i8 =
            ((qi.amount_of_bits() % 8) as i8) - ((qi.available_bits_in_first_byte() % 8) as i8);
        if right_shift < 0 {
            right_shift += 8;
        }
        // because we are applying bits in place we need masks in insure we don't effect other fields
        // data. we need one for the first byte and the last byte.
        let first_bit_mask = get_right_and_mask(qi.available_bits_in_first_byte());
        let last_bit_mask = if bits_in_last_byte == 0 {
            get_left_and_mask(8)
        } else {
            get_left_and_mask(bits_in_last_byte)
        };
        Self {
            right_shift,
            first_bit_mask,
            last_bit_mask,
            bits_in_last_byte,
        }
    }
}
pub struct LittleQuoteInfo {
    pub right_shift: i8,
    pub first_bit_mask: u8,
    pub last_bit_mask: u8,
}
impl From<&QuoteInfo> for LittleQuoteInfo {
    fn from(qi: &QuoteInfo) -> Self {
        let bits_in_last_byte = (qi.amount_of_bits() - qi.available_bits_in_first_byte()) % 8;
        // how many times to shift the number right.
        // NOTE if negative shift left.
        // NOTE if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        let mut bits_needed_in_msb = qi.amount_of_bits() % 8;
        if bits_needed_in_msb == 0 {
            bits_needed_in_msb = 8;
        }
        #[allow(clippy::cast_possible_truncation)]
        let mut right_shift: i8 =
            (bits_needed_in_msb as i8) - ((qi.available_bits_in_first_byte() % 8) as i8);
        if right_shift == 8 {
            right_shift = 0;
        }
        // because we are applying bits in place we need masks in insure we don't effect other fields
        // data. we need one for the first byte and the last byte.
        let first_bit_mask = get_right_and_mask(qi.available_bits_in_first_byte());
        let last_bit_mask = if bits_in_last_byte == 0 {
            get_left_and_mask(8)
        } else {
            get_left_and_mask(bits_in_last_byte)
        };
        Self {
            right_shift,
            first_bit_mask,
            last_bit_mask,
        }
    }
}
pub struct NoneQuoteInfo {
    pub right_shift: i8,
}
impl From<&QuoteInfo> for NoneQuoteInfo {
    fn from(quote_info: &QuoteInfo) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let right_shift: i8 = 8_i8 - ((quote_info.available_bits_in_first_byte() % 8) as i8);
        Self { right_shift }
    }
}
pub struct QuoteInfo {
    /// Amount of bits the field uses in bit form.
    amount_of_bits: usize,
    /// Amount of bits in the first byte this field has bits in that are not used by this field.
    zeros_on_left: usize,
    /// Amount of bits in the first byte this field has bits in that are used by this field.
    available_bits_in_first_byte: usize,
    /// the first byte this field is stored in
    starting_inject_byte: usize,
    /// the name of the buffer we will use to store the data for the fields value.
    field_buffer_name: Ident,
    /// if the structure is flipped. (reverse the bytes order)
    flip: Option<usize>,
}
impl QuoteInfo {
    pub fn new(field_info: &FieldInfo, struct_info: &StructInfo) -> syn::Result<Self> {
        Self::new_inner(field_info, struct_info.get_flip())
    }
    /// TODO im not sure `new_no_flip` should exist. no flip might not be needed.
    // pub fn new_no_flip(field_info: &FieldInfo) -> syn::Result<Self> {
    //     Self::new_inner(field_info, None)
    // }
    fn new_inner(field_info: &FieldInfo, flip: Option<usize>) -> syn::Result<Self> {
        // get the total number of bits the field uses.
        let amount_of_bits = field_info.attrs.bit_length();
        // amount of zeros to have for the right mask. (right mask meaning a mask to keep data on the
        // left)
        let zeros_on_left = field_info.attrs.bit_range.start % 8;
        // NOTE endianness is only for determining how to get the bytes we will apply to the output.
        // calculate how many of the bits will be inside the most significant byte we are adding to.
        if 7 < zeros_on_left {
            return Err(syn::Error::new(
                field_info.ident.span(),
                "ne 8 - zeros_on_left = underflow",
            ));
        }
        let available_bits_in_first_byte = 8 - zeros_on_left;
        // calculate the starting byte index in the outgoing buffer
        let mut starting_inject_byte: usize = field_info.attrs.bit_range.start / 8;
        if let Some(flip) = &flip {
            starting_inject_byte = *flip - starting_inject_byte;
            Some(flip)
        } else {
            None
        };
        // make a name for the buffer that we will store the number in byte form
        let field_buffer_name = format_ident!("{}_bytes", field_info.ident().ident());

        Ok(Self {
            amount_of_bits,
            zeros_on_left,
            available_bits_in_first_byte,
            starting_inject_byte,
            field_buffer_name,
            flip,
        })
    }
    /// Amount of bits the field uses in bit form.
    pub fn amount_of_bits(&self) -> usize {
        self.amount_of_bits
    }
    pub fn starting_inject_byte(&self) -> usize {
        self.starting_inject_byte
    }
    pub fn available_bits_in_first_byte(&self) -> usize {
        self.available_bits_in_first_byte
    }
    pub fn zeros_on_left(&self) -> usize {
        self.zeros_on_left
    }
    pub fn field_buffer_name(&self) -> &Ident {
        &self.field_buffer_name
    }
    /// Returns the next byte index in sequence based of the given `index` and whether or not the Structure in has a reverse bytes order.
    pub fn next_index(&self, index: usize) -> usize {
        if self.flip.is_some() {
            index - 1
        } else {
            index + 1
        }
    }
    /// Returns the `starting_inject_byte` plus or minus `offset` depending on if the bytes order is reversed.
    pub fn offset_starting_inject_byte(&self, offset: usize) -> usize {
        if self.flip.is_some() {
            self.starting_inject_byte - offset
        } else {
            self.starting_inject_byte + offset
        }
    }
    pub fn fields_last_bits_index(&self) -> usize {
        self.amount_of_bits.div_ceil(8) - 1
    }
    pub fn flip(&self) -> Option<usize> {
        self.flip
    }
}
impl TryFrom<(&FieldInfo, &StructInfo)> for QuoteInfo {
    type Error = syn::Error;
    fn try_from((fi, si): (&FieldInfo, &StructInfo)) -> Result<Self, Self::Error> {
        QuoteInfo::new(fi, si)
    }
}

impl FieldInfo {
    /// This will return a [`FieldQuotes`] which contains the code that goes into functions like:
    /// - `read_field`
    /// - `write_field`
    /// - `write_slice_field`
    /// - `StructChecked::read_field`
    ///
    /// More code, and the functions themselves, will be wrapped around this to insure it is safe.
    pub fn get_quotes(&self, struct_info: &StructInfo) -> syn::Result<GeneratedQuotes> {
        match self.attrs.endianness.mode() {
            EndiannessMode::Alternative => self.get_le_quotes(struct_info),
            EndiannessMode::Standard => self.get_be_quotes(struct_info),
            EndiannessMode::Nested => self.get_ne_quotes(struct_info),
        }
    }
    fn get_le_quotes(&self, struct_info: &StructInfo) -> Result<GeneratedQuotes, syn::Error> {
        let (read, write, clear) = {
            let read = self.get_read_quote(struct_info, FieldInfo::get_read_le_quote)?;
            let (write, clear) =
                self.get_write_quote(struct_info, FieldInfo::get_write_le_quote, false)?;
            (read, write, clear)
        };
        Ok(GeneratedQuotes {
            read,
            write,
            zero: clear,
        })
    }
    fn get_ne_quotes(&self, struct_info: &StructInfo) -> Result<GeneratedQuotes, syn::Error> {
        let (read, write, clear) = {
            // generate
            let read = self.get_read_quote(struct_info, FieldInfo::get_read_ne_quote)?;
            let (write, clear) =
                self.get_write_quote(struct_info, FieldInfo::get_write_ne_quote, false)?;
            (read, write, clear)
        };
        Ok(GeneratedQuotes {
            read,
            write,
            zero: clear,
        })
    }
    fn get_be_quotes(&self, struct_info: &StructInfo) -> Result<GeneratedQuotes, syn::Error> {
        let (read, write, clear) = {
            // generate
            let read = self.get_read_quote(struct_info, FieldInfo::get_read_be_quote)?;
            let (write, clear) =
                self.get_write_quote(struct_info, FieldInfo::get_write_be_quote, false)?;
            (read, write, clear)
        };
        Ok(GeneratedQuotes {
            read,
            write,
            zero: clear,
        })
    }
}

#[cfg(feature = "dyn_fns")]
/// generates the `check_slice` fn. please do not use, use `CheckedSliceGen`.
/// returns (fn, `fn_name`).
///
/// `name` is the name of the structure or variant
/// `check_size` is the total byte size of the struct or variant
/// `enum_name` if we are generating code for a variant (not a structure) then a
///     Some value containing the prefixed name shall be provided.
///     ex. enum and variant -> `Test::One` = "`test_one`" <- prefixed name
fn get_check_mut_slice_fn(
    name: &Ident,
    // total_bytes
    check_size: usize,
    enum_name: Option<&Ident>,
) -> (TokenStream, Ident) {
    use convert_case::{Case, Casing};

    let (checked_ident_mut, fn_name) = if let Some(ename) = enum_name {
        (
            format_ident!("{ename}CheckedMut"),
            format_ident!("check_slice_mut_{}", name.to_string().to_case(Case::Snake)),
        )
    } else {
        (
            format_ident!("{name}CheckedMut"),
            format_ident!("check_slice_mut"),
        )
    };
    let comment_mut = format!(
        "Returns a [{checked_ident_mut}] which allows you to read/write any field for a `{}` from/to provided mutable slice.",
        if let Some(ename) = enum_name {
            format!("{ename}::{name}")
        }else{
            name.to_string()
        }
    );
    (
        quote! {
            #[doc = #comment_mut]
            pub fn #fn_name(buffer: &mut [u8]) -> Result<#checked_ident_mut, bondrewd::BitfieldLengthError> {
                let buf_len = buffer.len();
                if buf_len >= #check_size {
                    Ok(#checked_ident_mut {
                        buffer
                    })
                }else{
                    Err(bondrewd::BitfieldLengthError(buf_len, #check_size))
                }
            }
        },
        fn_name,
    )
}
#[cfg(feature = "dyn_fns")]
/// generates the `check_slice` fn. please do not use, use `CheckedSliceGen`.
/// returns (fn, `fn_name`).
///
/// `name` is the name of the structure or variant
/// `check_size` is the total byte size of the struct or variant
/// `enum_name` if we are generating code for a variant (not a structure) then a
///     Some value containing the prefixed name shall be provided.
///     ex. enum and variant -> `Test::One` = "`test_one`" <- prefixed name
fn get_check_slice_fn(
    name: &Ident,
    // total_bytes
    check_size: usize,
    enum_name: Option<&Ident>,
) -> (TokenStream, Ident) {
    use convert_case::{Case, Casing};

    let (checked_ident, fn_name) = if let Some(ename) = enum_name {
        (
            format_ident!("{ename}Checked"),
            format_ident!("check_slice_{}", name.to_string().to_case(Case::Snake)),
        )
    } else {
        (format_ident!("{name}Checked"), format_ident!("check_slice"))
    };
    let comment = format!(
        "Returns a [{checked_ident}] which allows you to read any field for a `{}` from provided slice.",
        if let Some(ename) = enum_name {
            format!("{ename}::{name}")
        }else{
            name.to_string()
        }
    );
    (
        quote! {
            #[doc = #comment]
            pub fn #fn_name(buffer: &[u8]) -> Result<#checked_ident, bondrewd::BitfieldLengthError> {
                let buf_len = buffer.len();
                if buf_len >= #check_size {
                    Ok(#checked_ident {
                        buffer
                    })
                }else{
                    Err(bondrewd::BitfieldLengthError(buf_len, #check_size))
                }
            }
        },
        fn_name,
    )
}
#[cfg(feature = "dyn_fns")]
pub(crate) struct CheckedSliceGen {
    pub fn_gen: TokenStream,
    pub mut_fn_gen: TokenStream,
    pub fn_name: Ident,
    pub mut_fn_name: Ident,
}
#[cfg(feature = "dyn_fns")]
impl CheckedSliceGen {
    pub fn new(
        name: &Ident,
        // total_bytes
        check_size: usize,
        enum_name: Option<&Ident>,
    ) -> Self {
        let (fn_gen, fn_name) = get_check_slice_fn(name, check_size, enum_name);
        let (mut_fn_gen, mut_fn_name) = get_check_mut_slice_fn(name, check_size, enum_name);
        Self {
            fn_gen,
            mut_fn_gen,
            fn_name,
            mut_fn_name,
        }
    }
}
/// Generates a `read_field_name()` function.
pub(crate) fn generate_read_field_fn(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    field_name: &Ident,
) -> TokenStream {
    let fn_field_name = format_ident!("read_{field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let struct_size = &info.total_bytes();
    let comment_bits = if bit_range.end - bit_range.start > 1 {
        format!("bits {} through {}", bit_range.start, bit_range.end - 1)
    } else {
        format!("bit {}", bit_range.start)
    };
    let comment = format!("Reads {comment_bits} within `input_byte_buffer`, getting the `{field_name}` field of a `{struct_name}` in bitfield form.");
    quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(input_byte_buffer: &[u8;#struct_size]) -> #type_ident {
            #field_quote
        }
    }
}
/// Generates a `read_slice_field_name()` function for a slice.
#[cfg(feature = "dyn_fns")]
pub(crate) fn generate_read_slice_field_fn(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    field_name: &Ident,
) -> TokenStream {
    let fn_field_name = format_ident!("read_slice_{field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let min_length = if info.attrs.default_endianess.is_byte_order_reversed() {
        (info.total_bits() - field.attrs.bit_range.start).div_ceil(8)
    } else {
        // TODO check this is correct in generated code.
        field.attrs.bit_range.end.div_ceil(8)
    };
    let comment = format!("Returns the value for the `{field_name}` field of a `{struct_name}` in bitfield form by reading  bits {} through {} in `input_byte_buffer`. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned if not enough bytes are present.", bit_range.start, bit_range.end - 1);
    quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(input_byte_buffer: &[u8]) -> Result<#type_ident, bondrewd::BitfieldLengthError> {
            let slice_length = input_byte_buffer.len();
            if slice_length < #min_length {
                Err(bondrewd::BitfieldLengthError(slice_length, #min_length))
            } else {
                Ok(
                    #field_quote
                )
            }
        }
    }
}
/// For use on generated Checked Slice Structures.
///
/// Generates a `read_field_name()` function for a slice.
///
/// # Warning
/// generated code does NOT check if the slice is large enough to be read from, Checked Slice Structures
/// are nothing but a slice ref that has been checked to contain enough bytes for any
/// `read_slice_field_name` functions.
#[cfg(feature = "dyn_fns")]
pub(crate) fn generate_read_slice_field_fn_unchecked(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    field_name: &Ident,
) -> TokenStream {
    let fn_field_name = format_ident!("read_{field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let comment_bits = if bit_range.end - bit_range.start > 1 {
        format!("bits {} through {}", bit_range.start, bit_range.end - 1)
    } else {
        format!("bit {}", bit_range.start)
    };
    let comment = format!(
        "Reads {comment_bits} in pre-checked slice, getting the `{field_name}` field of a [{struct_name}] in bitfield form."
    );
    quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(&self) -> #type_ident {
            let input_byte_buffer: &[u8] = self.buffer;
            #field_quote
        }
    }
}

/// Generates a `write_field_name()` function.
pub(crate) fn generate_write_field_fn(
    field_quote: &TokenStream,
    clear_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    field_name: &Ident,
) -> TokenStream {
    let field_name_short = field.ident().ident();
    let struct_size = info.total_bytes();
    let bit_range = &field.attrs.bit_range;
    let fn_field_name = format_ident!("write_{field_name}");
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let comment_bits = if bit_range.end - bit_range.start > 1 {
        format!("bits {} through {}", bit_range.start, bit_range.end - 1)
    } else {
        format!("bit {}", bit_range.start)
    };
    let comment = format!("Writes to {comment_bits} within `output_byte_buffer`, setting the `{field_name}` field of a `{struct_name}` in bitfield form.");
    quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(output_byte_buffer: &mut [u8;#struct_size], mut #field_name_short: #type_ident) {
            #clear_quote
            #field_quote
        }
    }
}
/// Generates a `write_slice_field_name()` function for a slice.
#[cfg(feature = "dyn_fns")]
pub(crate) fn generate_write_slice_field_fn(
    field_quote: &TokenStream,
    clear_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    prefixed_field_name: &Ident,
) -> TokenStream {
    let field_name = field.ident().name();
    let fn_field_name = format_ident!("write_slice_{prefixed_field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let min_length = if info.attrs.default_endianess.is_byte_order_reversed() {
        (info.total_bits() - field.attrs.bit_range.start).div_ceil(8)
    } else {
        field.attrs.bit_range.end.div_ceil(8)
    };
    let comment_bits = if bit_range.end - bit_range.start > 1 {
        format!("bits {} through {}", bit_range.start, bit_range.end - 1)
    } else {
        format!("bit {}", bit_range.start)
    };
    let comment = format!("Writes to {comment_bits} in `input_byte_buffer` if enough bytes are present in slice, setting the `{field_name}` field of a `{struct_name}` in bitfield form. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned");
    quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(output_byte_buffer: &mut [u8], #field_name: #type_ident) -> Result<(), bondrewd::BitfieldLengthError> {
            let slice_length = output_byte_buffer.len();
            if slice_length < #min_length {
                Err(bondrewd::BitfieldLengthError(slice_length, #min_length))
            } else {
                #clear_quote
                #field_quote
                Ok(())
            }
        }
    }
}
/// For use on generated Checked Slice Structures.
///
/// Generates a `write_field_name()` function for a slice.
///
/// # Warning
/// generated code does NOT check if the slice can be written to, Checked Slice Structures are nothing
/// but a slice ref that has been checked to contain enough bytes for any `write_slice_field_name`
/// functions.
#[cfg(feature = "dyn_fns")]
pub(crate) fn generate_write_slice_field_fn_unchecked(
    field_quote: &TokenStream,
    clear_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
) -> TokenStream {
    let field_name = field.ident().name();
    let fn_field_name = format_ident!("write_{field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let comment_bits = if bit_range.end - bit_range.start > 1 {
        format!("bits {} through {}", bit_range.start, bit_range.end - 1)
    } else {
        format!("bit {}", bit_range.start)
    };
    let comment = format!(
        "Writes to {comment_bits} in pre-checked mutable slice, setting the `{field_name}` field of a [{struct_name}] in bitfield form.",
    );
    quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(&mut self, #field_name: #type_ident) {
            let output_byte_buffer: &mut [u8] = self.buffer;
            #clear_quote
            #field_quote
        }
    }
}
