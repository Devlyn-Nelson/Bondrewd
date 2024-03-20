//! This file is an effort to merge from and into bytes, which is being delayed for now.
mod from;
mod into;
use proc_macro2::{Ident, TokenStream};
use quote::format_ident;

use crate::parse::common::{
    get_left_and_mask, get_right_and_mask, Endianness, FieldInfo, StructInfo,
};
pub struct FieldQuotes {
    read: proc_macro2::TokenStream,
    write: proc_macro2::TokenStream,
    zero: proc_macro2::TokenStream,
}
impl FieldQuotes {
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
        let flip = if let Some(flip) = flip {
            starting_inject_byte = flip - starting_inject_byte;
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
    pub fn get_quotes(&self, struct_info: &StructInfo) -> syn::Result<FieldQuotes> {
        match *self.attrs.endianness {
            Endianness::Little => self.get_le_quotes(struct_info),
            Endianness::Big => self.get_be_quotes(struct_info),
            Endianness::None => self.get_ne_quotes(struct_info),
        }
    }
    fn get_le_quotes(&self, struct_info: &StructInfo) -> Result<FieldQuotes, syn::Error> {
        let (read, write, clear) = {
            let read = self.get_read_quote(struct_info, FieldInfo::get_read_le_quote)?;
            let (write, clear) =
                self.get_write_quote(struct_info, FieldInfo::get_write_le_quote, false)?;
            (read, write, clear)
        };
        Ok(FieldQuotes {
            read,
            write,
            zero: clear,
        })
    }
    fn get_ne_quotes(&self, struct_info: &StructInfo) -> Result<FieldQuotes, syn::Error> {
        let (read, write, clear) = {
            // generate
            let read = self.get_read_quote(struct_info, FieldInfo::get_read_ne_quote)?;
            let (write, clear) =
                self.get_write_quote(struct_info, FieldInfo::get_write_ne_quote, false)?;
            (read, write, clear)
        };
        Ok(FieldQuotes {
            read,
            write,
            zero: clear,
        })
    }
    fn get_be_quotes(&self, struct_info: &StructInfo) -> Result<FieldQuotes, syn::Error> {
        let (read, write, clear) = {
            // generate
            let read = self.get_read_quote(struct_info, FieldInfo::get_read_be_quote)?;
            let (write, clear) =
                self.get_write_quote(struct_info, FieldInfo::get_write_be_quote, false)?;
            (read, write, clear)
        };
        Ok(FieldQuotes {
            read,
            write,
            zero: clear,
        })
    }
}
