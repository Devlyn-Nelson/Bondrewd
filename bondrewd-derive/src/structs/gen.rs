//! This file is an effort to merge from and into bytes, which is being delayed for now.
use std::{cmp::Ordering, collections::VecDeque};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma};

use crate::structs::common::{get_left_and_mask, get_right_and_mask};

use super::common::{Endianness, FieldDataType, FieldInfo, NumberSignage, StructInfo};

pub struct GeneratedFunctions {
    /// Functions that belong in `Bitfields` impl for object.
    pub bitfield_trait_impl_fns: proc_macro2::TokenStream,
    /// Functions that belong in impl for object.
    pub impl_fns: proc_macro2::TokenStream,
    /// Functions that belong in impl for generated checked slice object.
    pub checked_struct_impl_fns: proc_macro2::TokenStream,
    /// Functions that belong in `BitfieldsDyn` impl for object.
    #[cfg(feature = "dyn_fns")]
    pub bitfield_dyn_trait_impl_fns: proc_macro2::TokenStream,
}

// pub fn generate_functions_enum(info: &EnumInfo) -> Result<GeneratedFunctions, syn::Error> {
//     // function for getting the id of an enum.
//     let mut id_fn = quote! {};
//     let mut bitfield_trait_impl_fns = quote! {};
//     let mut impl_fns = quote! {};
//     #[cfg(feature = "dyn_fns")]
//     let mut bitfield_dyn_trait_impl_fns = quote! {};

//     let from = {
//         let field = info.generate_id_field()?;
//         let flip = false;
//         let field_extractor = get_field_quote(
//             &field,
//             if flip {
//                 // condition use to be `info.attrs.flip` i think this only applies to the variants
//                 // and id_position is what is used here. but it should be done none the less.
//                 Some(info.total_bytes() - 1)
//             } else {
//                 None
//             },
//         )?;
//         let attrs = info.attrs.attrs.clone();
//         let mut fields = vec![field.clone()];
//         fields[0].attrs.bit_range = 0..info.total_bits();
//         let temp_struct_info = StructInfo {
//             name: info.name.clone(),
//             attrs,
//             fields,
//             vis: syn::Visibility::Public(Pub::default()),
//             tuple: false,
//         };
//         let id_field = generate_read_field_fn(&field_extractor, &field, &temp_struct_info, &None);
//         #[cfg(feature = "dyn_fns")]
//         {
//             let id_slice_peek =
//                 generate_read_slice_field_fn(&field_extractor, &field, &temp_struct_info, &None);
//             quote! {
//                 #id_field
//                 #id_slice_peek
//             }
//         }
//         #[cfg(not(feature = "dyn_fns"))]
//         {
//             quote! {
//                 #id_field
//             }
//         }
//     };

//     let into = {
//         let (field_setter, clear_quote) = get_field_quote(
//             &field,
//             if flip {
//                 // condition use to be `info.attrs.flip` i think this only applies to the variants
//                 // and id_position is what is used here. but it should be done none the less.
//                 Some(info.total_bytes() - 1)
//             } else {
//                 None
//             },
//             false,
//         )?;
//         let id_field = generate_write_field_fn(
//             &field_setter,
//             &field,
//             &StructInfo {
//                 name: info.name.clone(),
//                 attrs,
//                 fields,
//                 vis: syn::Visibility::Public(Pub::default()),
//                 tuple: false,
//             },
//             &clear_quote,
//             &None,
//         );
//         let out = quote! {
//             #id_field
//         };
//         let out = {
//             let q = make_checked_mut_func(&info.name, info.total_bytes());
//             quote! {
//                 #out
//                 #q
//             }
//         };
//         out
//     };

//     todo!("finish merged (from AND into) generate functions");
// }
/// the flip value must be the total amount of bytes the result of `into_bytes` should have MINUS ONE,
/// the number is used to invert indices
// fn get_field_quotes(
//     field: &FieldInfo,
//     flip: Option<usize>,
//     with_self: bool,
// ) -> syn::Result<FieldQuotes> {
//     let field_name = field.ident().name();
//     let quote_field_name = match field.ty {
//         FieldDataType::Float(_, _) => {
//             if with_self {
//                 quote! {self.#field_name.to_bits()}
//             } else {
//                 quote! {#field_name.to_bits()}
//             }
//         }
//         FieldDataType::Char(_, _) => {
//             if with_self {
//                 quote! {(self.#field_name as u32)}
//             } else {
//                 quote! {(#field_name as u32)}
//             }
//         }
//         FieldDataType::Enum(_, _, _) => {
//             if with_self {
//                 quote! {((self.#field_name).into_primitive())}
//             } else {
//                 quote! {((#field_name).into_primitive())}
//             }
//         }
//         FieldDataType::ElementArray(_, _, _) => {
//             let mut clear_buffer = quote! {};
//             let mut buffer = quote! {};
//             let mut de_refs: Punctuated<IdentSyn, Comma> = Punctuated::default();
//             let outer_field_name = &field.ident().ident();
//             let sub = field.get_element_iter()?;
//             for sub_field in sub {
//                 let field_name = &sub_field.ident().name();
//                 let (sub_field_quote, clear) = get_field_quote(&sub_field, flip, with_self)?;
//                 buffer = quote! {
//                     #buffer
//                     #sub_field_quote
//                 };
//                 clear_buffer = quote! {
//                     #clear_buffer
//                     #clear
//                 };
//                 de_refs.push(format_ident!("{}", field_name));
//             }
//             buffer = quote! {
//                 let [#de_refs] = #outer_field_name;
//                 #buffer
//             };
//             return Ok((buffer, clear_buffer));
//         }
//         FieldDataType::BlockArray(_, _, _) => {
//             let mut buffer = quote! {};
//             let mut clear_buffer = quote! {};
//             let mut de_refs: Punctuated<IdentSyn, Comma> = Punctuated::default();
//             let outer_field_name = &field.ident().ident();
//             let sub = field.get_block_iter()?;
//             for sub_field in sub {
//                 let field_name = &sub_field.ident().name();
//                 let (sub_field_quote, clear) = get_field_quote(&sub_field, flip, with_self)?;
//                 buffer = quote! {
//                     #buffer
//                     #sub_field_quote
//                 };
//                 clear_buffer = quote! {
//                     #clear_buffer
//                     #clear
//                 };
//                 de_refs.push(format_ident!("{}", field_name));
//             }
//             buffer = quote! {
//                 let [#de_refs] = #outer_field_name;
//                 #buffer
//             };
//             return Ok((buffer, clear_buffer));
//         }
//         _ => {
//             if with_self {
//                 quote! {self.#field_name}
//             } else {
//                 quote! {#field_name}
//             }
//         }
//     };
//     match field.attrs.endianness.as_ref() {
//         Endianness::Big => apply_be_math_to_field_access_quote(field, quote_field_name, flip),
//         Endianness::Little => apply_le_math_to_field_access_quote(field, quote_field_name, flip),
//         Endianness::None => apply_ne_math_to_field_access_quote(field, &quote_field_name, flip),
//     }
// }
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
struct QuoteInfo {
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
        let flip = if let Some(flip) = struct_info.get_flip() {
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
    /// Returns the starting_inject_byte plus or minus `offset` depending on if the bytes order is reversed.
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
}

enum GenerateWriteQuoteFn {
    Single(fn(&FieldInfo, &QuoteInfo, TokenStream) -> syn::Result<(TokenStream, TokenStream)>),
    Multi {
        right_shift: i8,
        first_bit_mask: u8,
        last_bit_mask: u8,
        gen_fn: fn(
            &FieldInfo,
            &QuoteInfo,
            i8,
            u8,
            u8,
            TokenStream,
        ) -> syn::Result<(TokenStream, TokenStream)>,
    },
    MultiNoEndianness {
        right_shift: i8,
        gen_fn: fn(
            &FieldInfo,
            &QuoteInfo,
            i8,
            TokenStream,
        ) -> syn::Result<(TokenStream, TokenStream)>,
    },
}

impl GenerateWriteQuoteFn {
    pub fn le_multi_byte(right_shift: i8, first_bit_mask: u8, last_bit_mask: u8) -> Self {
        Self::Multi {
            right_shift,
            first_bit_mask,
            last_bit_mask,
            gen_fn: FieldInfo::get_write_le_multi_byte_quote,
        }
    }
    pub fn le_single_byte() -> Self {
        Self::Single(FieldInfo::get_write_le_single_byte_quote)
    }
    pub fn ne_multi_byte(right_shift: i8) -> Self {
        Self::MultiNoEndianness {
            right_shift,
            gen_fn: FieldInfo::get_write_ne_multi_byte_quote,
        }
    }
    pub fn ne_single_byte() -> Self {
        Self::Single(FieldInfo::get_write_ne_single_byte_quote)
    }
    pub fn be_multi_byte(right_shift: i8, first_bit_mask: u8, last_bit_mask: u8) -> Self {
        Self::Multi {
            right_shift,
            first_bit_mask,
            last_bit_mask,
            gen_fn: FieldInfo::get_write_be_multi_byte_quote,
        }
    }
    pub fn be_single_byte() -> Self {
        Self::Single(FieldInfo::get_write_be_single_byte_quote)
    }
    pub fn run(&self, field_info: &FieldInfo, quote_info: &QuoteInfo, field_access: TokenStream) -> syn::Result<(TokenStream,TokenStream)> {
        match self {
            GenerateWriteQuoteFn::Single(gen_fn) => gen_fn(field_info, quote_info, field_access),
            GenerateWriteQuoteFn::Multi {
                right_shift,
                first_bit_mask,
                last_bit_mask,
                gen_fn,
            } => gen_fn(
                field_info,
                quote_info,
                *right_shift,
                *first_bit_mask,
                *last_bit_mask,
                field_access,
            ),
            GenerateWriteQuoteFn::MultiNoEndianness {
                right_shift,
                gen_fn,
            } => gen_fn(
                field_info,
                quote_info,
                *right_shift,
                field_access,
            ),
        }
    }
}

impl FieldInfo {
    /// This function is kind of funny. it is essentially a function that gets called by either
    /// `get_le_quotes`, `get_be_quotes`, `get_ne_quotes` with the end code generation function given
    /// as a parameter `gen_write_fn`. and example of a function that can be used as `gen_write_fn` would
    /// be `get_write_le_multibyte_quote`;
    fn get_write_quote(
        &self,
        quote_info: &QuoteInfo,
        gen_write_fn: &GenerateWriteQuoteFn,
        with_self: bool,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        let field_name = self.ident().name();
        let field_access = match self.ty {
            FieldDataType::Float(_, _) => {
                if with_self {
                    quote! {self.#field_name.to_bits()}
                } else {
                    quote! {#field_name.to_bits()}
                }
            }
            FieldDataType::Char(_, _) => {
                if with_self {
                    quote! {(self.#field_name as u32)}
                } else {
                    quote! {(#field_name as u32)}
                }
            }
            FieldDataType::Enum(_, _, _) => {
                if with_self {
                    quote! {((self.#field_name).into_primitive())}
                } else {
                    quote! {((#field_name).into_primitive())}
                }
            }
            // Array types need to recurse which is the reason this in-between function exists.
            FieldDataType::ElementArray(_, _, _) => {
                let mut clear_buffer = quote! {};
                let mut buffer = quote! {};
                let mut de_refs: Punctuated<syn::Ident, Comma> = Punctuated::default();
                let outer_field_name = &self.ident().ident();
                let sub = self.get_element_iter()?;
                for sub_field in sub {
                    let field_name = &sub_field.ident().name();
                    let (sub_field_quote, clear) =
                        self.get_write_quote(quote_info, gen_write_fn, with_self)?;
                    buffer = quote! {
                        #buffer
                        #sub_field_quote
                    };
                    clear_buffer = quote! {
                        #clear_buffer
                        #clear
                    };
                    de_refs.push(format_ident!("{}", field_name));
                }
                buffer = quote! {
                    let [#de_refs] = #outer_field_name;
                    #buffer
                };
                return Ok((buffer, clear_buffer));
            }
            FieldDataType::BlockArray(_, _, _) => {
                let mut buffer = quote! {};
                let mut clear_buffer = quote! {};
                let mut de_refs: Punctuated<syn::Ident, Comma> = Punctuated::default();
                let outer_field_name = &self.ident().ident();
                let sub = self.get_block_iter()?;
                for sub_field in sub {
                    let field_name = &sub_field.ident().name();
                    let (sub_field_quote, clear) =
                        self.get_write_quote(quote_info, gen_write_fn, with_self)?;
                    buffer = quote! {
                        #buffer
                        #sub_field_quote
                    };
                    clear_buffer = quote! {
                        #clear_buffer
                        #clear
                    };
                    de_refs.push(format_ident!("{}", field_name));
                }
                buffer = quote! {
                    let [#de_refs] = #outer_field_name;
                    #buffer
                };
                return Ok((buffer, clear_buffer));
            }
            _ => {
                if with_self {
                    quote! {self.#field_name}
                } else {
                    quote! {#field_name}
                }
            }
        };
        gen_write_fn.run(self, quote_info, field_access)
    }
    pub fn get_quotes(&self, struct_info: &StructInfo) -> syn::Result<FieldQuotes> {
        let qi = QuoteInfo::new(self, struct_info)?;
        match *self.attrs.endianness {
            Endianness::Little => self.get_le_quotes(qi),
            Endianness::Big => self.get_be_quotes(qi),
            Endianness::None => self.get_ne_quotes(qi),
        }
    }
    fn get_le_quotes(&self, quote_info: QuoteInfo) -> Result<FieldQuotes, syn::Error> {
        let (read,write,clear) = if quote_info.amount_of_bits() > quote_info.available_bits_in_first_byte() {
            // calculate how many of the bits will be inside the least significant byte we are adding to.
            // this will also be the number used for shifting to the right >> because that will line up
            // our bytes for the buffer.
            if quote_info.amount_of_bits() < quote_info.available_bits_in_first_byte() {
                return Err(syn::Error::new(
                    self.ident.span(),
                    "calculating le `bits_in_last_bytes` failed",
                ));
            }
            let bits_in_last_byte =
                (quote_info.amount_of_bits() - quote_info.available_bits_in_first_byte()) % 8;
            // how many times to shift the number right.
            // NOTE if negative shift left.
            // NOTE if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
            // for a f32) then use the last byte in the fields byte array after shifting for the first
            // used byte in the buffer.
            let mut bits_needed_in_msb = quote_info.amount_of_bits() % 8;
            if bits_needed_in_msb == 0 {
                bits_needed_in_msb = 8;
            }
            #[allow(clippy::cast_possible_truncation)]
            let mut right_shift: i8 = (bits_needed_in_msb as i8)
                - ((quote_info.available_bits_in_first_byte() % 8) as i8);
            if right_shift == 8 {
                right_shift = 0;
            }
            // because we are applying bits in place we need masks in insure we don't effect other fields
            // data. we need one for the first byte and the last byte.
            let first_bit_mask = get_right_and_mask(quote_info.available_bits_in_first_byte());
            let last_bit_mask = if bits_in_last_byte == 0 {
                get_left_and_mask(8)
            } else {
                get_left_and_mask(bits_in_last_byte)
            };
            // create a quote that holds the bit shifting operator and shift value and the field name.
            // first_bits_index is the index to use in the fields byte array after shift for the
            // starting byte in the byte buffer. when left shifts happen on full sized numbers the last
            // index of the fields byte array will be used.
            //
            // let shift = if right_shift < 0 {
            //     // convert to left shift using absolute value
            //     let left_shift: u32 = right_shift.clone().abs() as u32;
            //     // shift left code
            //     quote! { (#field_access_quote.rotate_left(#left_shift)) }
            // } else {
            //     if right_shift == 0 {
            //         // no shift no code, just the
            //         quote! { #field_access_quote }
            //     } else {
            //         // shift right code
            //         let right_shift_usize: u32 = right_shift.clone() as u32;
            //         quote! { (#field_access_quote.rotate_right(#right_shift_usize)) }
            //     }
            // };
            let read = self.get_read_le_multi_byte_quote(
                &quote_info,
                right_shift,
                first_bit_mask,
                last_bit_mask,
            )?;
            let gen_write_fn =
                GenerateWriteQuoteFn::le_multi_byte(right_shift, first_bit_mask, last_bit_mask);
            let (write, clear) = self.get_write_quote(
                &quote_info,
                &gen_write_fn,
                false
            )?;
            (read,write,clear)
        } else {
            // single bytes logic
            let read = self.get_read_le_single_byte_quote(&quote_info)?;
            let gen_write_fn =
                GenerateWriteQuoteFn::le_single_byte();
            let (write, clear) = self.get_write_quote(&quote_info, &gen_write_fn, false)?;
            (read,write,clear)
        };
        Ok(FieldQuotes { read, write, zero: clear })
    }
    fn get_ne_quotes(&self, quote_info: QuoteInfo) -> Result<FieldQuotes, syn::Error> {
        let (read,write,clear) = if quote_info.amount_of_bits > quote_info.available_bits_in_first_byte {
            // how many times to shift the number right.
            // NOTE if negative shift left.
            // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
            // for a f32) then use the last byte in the fields byte array after shifting for the first
            // used byte in the buffer.
            if 8 < quote_info.available_bits_in_first_byte() % 8 {
                return Err(syn::Error::new(
                    self.ident.span(),
                    "calculating ne right_shift failed",
                ));
            }
            #[allow(clippy::cast_possible_truncation)]
            let right_shift: i8 = 8_i8 - ((quote_info.available_bits_in_first_byte() % 8) as i8);
            // generate
            let read = self.get_read_ne_multi_byte_quote(
                &quote_info,
                right_shift,
            )?;
            let gen_write_fn =
                GenerateWriteQuoteFn::ne_multi_byte(right_shift);
            let (write, clear) = self.get_write_quote(
                &quote_info,
                &gen_write_fn,
                false
            )?;
            (read,write,clear)
        }else{
            // single bytes logic
            let read = self.get_read_ne_single_byte_quote(&quote_info)?;
            let gen_write_fn =
                GenerateWriteQuoteFn::ne_single_byte();
            let (write, clear) = self.get_write_quote(&quote_info, &gen_write_fn, false)?;
            (read,write,clear)
        };
        Ok(FieldQuotes { read, write, zero: clear })
    }
    fn get_be_quotes(&self, _quote_info: QuoteInfo) -> Result<FieldQuotes, syn::Error> {
        todo!("merged Big Endianness from_bytes/into_bytes Generation code here")
    }
    fn get_read_le_single_byte_quote(
        &self,
        quote_info: &QuoteInfo,
    ) -> syn::Result<TokenStream> {
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (quote_info.zeros_on_left() + quote_info.amount_of_bits()) {
            return Err(syn::Error::new(
                self.ident.span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (quote_info.zeros_on_left() + quote_info.amount_of_bits());
        // combining the left and right masks will give us a mask that keeps the amount og bytes we
        // have in the position we need them to be in for this byte. we use available_bytes for
        // right mask because param is amount of 1's on the side specified (right), and
        // available_bytes is (8 - zeros_on_left) which is equal to ones_on_right.
        let mask = get_right_and_mask(quote_info.available_bits_in_first_byte())
            & get_left_and_mask(8 - zeros_on_right);
        // calculate how many left shifts need to occur to the number in order to position the bytes
        // we want to keep in the position we want.
        if 8 - quote_info.amount_of_bits() < self.attrs.bit_range.start % 8 {
            return Err(syn::Error::new(
                self.ident.span(),
                format!(
                    "calculating be left_shift failed {} , {}",
                    quote_info.amount_of_bits(),
                    self.attrs.bit_range.start % 8
                ),
            ));
        }
        let shift_left = (8 - quote_info.amount_of_bits()) - (self.attrs.bit_range.start % 8);
        // a quote that puts the field into a byte buffer we assume exists (because this is a
        // fragment).
        // NOTE the mask used here is only needed if we can NOT guarantee the field is only using the
        // bits the size attribute says it can. for example if our field is a u8 but the bit_length
        // attribute say to only use 2 bits, then the possible values are 0-3. so if the u8 (0-255)
        // is set to 4 then the extra bit being used will be dropped by the mask making the value 0.
        // FEATURE remove the "#mask & " from this quote to make it faster. but that means the
        // numbers have to be correct. if you want the no-mask feature then suggested enforcement of
        // the number would be:
        //      - generate setters that make a mask that drops bits not desired. (fast)
        //      - generate setters that check if its above max_value for the bit_length and set it
        //          to the max_value if its larger. (prevents situations like the 2bit u8 example
        //          in the note above)
        // both of these could benefit from a return of the number that actually got set.
        let starting_inject_byte = quote_info.starting_inject_byte();
        let field_buffer_name = quote_info.field_buffer_name();
        let output_quote = match self.ty {
            FieldDataType::Number(_, ref sign, ref ident) => {
                let mut field_value = quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left)};
                if let NumberSignage::Signed = sign {
                    field_value = add_sign_fix_quote_single_bit(field_value, self, quote_info.amount_of_bits(), starting_inject_byte);
                    let mut value = quote!{
                        let mut #field_buffer_name = #field_value;
                    };
                    value = quote!{
                        {
                            #value
                            #field_buffer_name as #ident
                        }
                    };
                    value
                }else{
                    quote!{
                        #field_value as #ident
                    }
                }
            }
            FieldDataType::Boolean => {
                quote!{(input_byte_buffer[#starting_inject_byte] & #mask) != 0}
            }
            FieldDataType::Char(_, _) => quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left) as u32},
            FieldDataType::Enum(ref ident, _, _) => quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left) as #ident},
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(self.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Float(_, _) => return Err(syn::Error::new(self.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad.")),
        };
        Ok(output_quote)
    }
    fn get_read_le_multi_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        right_shift: i8,
        first_bit_mask: u8,
        last_bit_mask: u8,
    ) -> syn::Result<TokenStream> {
        let rust_type_size = self.ty.size();
        // Allocate a buffer to put the bits into as we extract them. if signed we need to be able fill
        // the unused bits with zero or 1 depending on if it is negative when read.
        let new_array_quote =
            if let Some(a) = add_sign_fix_quote(self, quote_info.amount_of_bits(), right_shift)? {
                a
            } else {
                quote! {[0u8;#rust_type_size]}
            };
        let field_buffer_name = &quote_info.field_buffer_name();
        // Create the full buffer initiation quotes.
        let mut full_quote = quote! {
            let mut #field_buffer_name: [u8;#rust_type_size] = #new_array_quote;
        };
        // generate the reading code. the while loop will do all except for the last byte.
        let fields_last_bits_index = quote_info.fields_last_bits_index();
        let current_bit_mask = get_right_and_mask(quote_info.available_bits_in_first_byte());
        #[allow(clippy::cast_possible_truncation)]
        let mid_shift: u32 = 8 - quote_info.available_bits_in_first_byte() as u32;
        let next_bit_mask = get_left_and_mask(mid_shift as usize);
        let mut i = 0;
        while i != fields_last_bits_index {
            let start = quote_info.offset_starting_inject_byte(i);
            if quote_info.available_bits_in_first_byte() == 0 && right_shift == 0 {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#start];
                };
            } else {
                if current_bit_mask == u8::MAX {
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] |= input_byte_buffer[#start];
                    };
                } else if current_bit_mask != 0 {
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] |= input_byte_buffer[#start] & #current_bit_mask;
                    };
                }
                if quote_info.available_bits_in_first_byte() + (8 * i) < quote_info.amount_of_bits()
                    && next_bit_mask != 0
                {
                    let next_index = quote_info.next_index(start);
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] |= input_byte_buffer[#next_index] & #next_bit_mask;
                    }
                }
                if mid_shift != 0 {
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] = #field_buffer_name[#i].rotate_left(#mid_shift);
                    };
                }
            }
            i += 1;
        }
        // finish read the last byte's bits.
        let used_bits = quote_info.available_bits_in_first_byte() + (8 * i);
        if right_shift > 0 {
            let start = quote_info.offset_starting_inject_byte(i);
            let right_shift: u32 = u32::from(right_shift.unsigned_abs());
            if used_bits < quote_info.amount_of_bits() {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#start] & #current_bit_mask;
                    #field_buffer_name[#i] |= input_byte_buffer[#start + 1] & #last_bit_mask;
                };
            } else {
                let mut last_mask = first_bit_mask;
                if quote_info.amount_of_bits() < used_bits {
                    last_mask &= !get_right_and_mask(used_bits - quote_info.amount_of_bits());
                }
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#start] & #last_mask;
                };
            }
            full_quote = quote! {
                #full_quote
                #field_buffer_name[#i] = #field_buffer_name[#i].rotate_left(#right_shift);
            };
        } else {
            let start = quote_info.offset_starting_inject_byte(i);
            // this should give us the last index of the field
            let left_shift: u32 = u32::from(right_shift.unsigned_abs());
            let mid_mask = first_bit_mask & last_bit_mask;
            if mid_mask == u8::MAX {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= (input_byte_buffer[#start]);
                    #field_buffer_name[#i] = #field_buffer_name[#i].rotate_right(#left_shift);
                };
            } else {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= (input_byte_buffer[#start] & #mid_mask);
                    #field_buffer_name[#i] = #field_buffer_name[#i].rotate_right(#left_shift);
                };
            }
        }
        full_quote = quote! {
            #full_quote
            #field_buffer_name
        };
        // generate code to transform buffer into rust type.
        let output = match self.ty {
            FieldDataType::Number(_, _, ref type_quote) |
            FieldDataType::Enum(ref type_quote, _, _) => {
                let apply_field_to_buffer = quote! {
                    #type_quote::from_le_bytes({
                        #full_quote
                    })
                };
                apply_field_to_buffer
            }
            FieldDataType::Float(_, _) => {
                let alt_type_quote = if rust_type_size == 4 {
                    quote!{u32}
                }else if rust_type_size == 8 {
                    quote!{u64}
                }else{
                    return Err(syn::Error::new(self.ident.span(), "unsupported floating type"))
                };
                let apply_field_to_buffer = quote! {
                    #alt_type_quote::from_le_bytes({
                        #full_quote
                    })
                };
                apply_field_to_buffer
            }
            FieldDataType::Char(_, _) => {
                let apply_field_to_buffer = quote! {
                    u32::from_le_bytes({
                        #full_quote
                    })
                };
                apply_field_to_buffer
            }
            FieldDataType::Boolean => return Err(syn::Error::new(self.ident.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(self.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad."))
        };

        Ok(output)
    }
    fn get_write_le_single_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        field_access_quote: TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (quote_info.zeros_on_left() + quote_info.amount_of_bits()) {
            return Err(syn::Error::new(
                self.ident.span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (quote_info.zeros_on_left() + quote_info.amount_of_bits());
        // combining the left and right masks will give us a mask that keeps the amount og bytes we
        // have in the position we need them to be in for this byte. we use available_bytes for
        // right mask because param is amount of 1's on the side specified (right), and
        // available_bytes is (8 - zeros_on_left) which is equal to ones_on_right.
        let mask = get_right_and_mask(quote_info.available_bits_in_first_byte())
            & get_left_and_mask(8 - zeros_on_right);
        // calculate how many left shifts need to occur to the number in order to position the bytes
        // we want to keep in the position we want.
        if 8 - quote_info.amount_of_bits() < self.attrs.bit_range.start % 8 {
            return Err(syn::Error::new(
                self.ident.span(),
                format!(
                    "calculating be left_shift failed {} , {}",
                    quote_info.amount_of_bits(),
                    self.attrs.bit_range.start % 8
                ),
            ));
        }
        let shift_left = (8 - quote_info.amount_of_bits()) - (self.attrs.bit_range.start % 8);
        // a quote that puts the field into a byte buffer we assume exists (because this is a
        // fragment).
        // NOTE the mask used here is only needed if we can NOT guarantee the field is only using the
        // bits the size attribute says it can. for example if our field is a u8 but the bit_length
        // attribute say to only use 2 bits, then the possible values are 0-3. so if the u8 (0-255)
        // is set to 4 then the extra bit being used will be dropped by the mask making the value 0.
        // FEATURE remove the "#mask & " from this quote to make it faster. but that means the
        // numbers have to be correct. if you want the no-mask feature then suggested enforcement of
        // the number would be:
        //      - generate setters that make a mask that drops bits not desired. (fast)
        //      - generate setters that check if its above max_value for the bit_length and set it
        //          to the max_value if its larger. (prevents situations like the 2bit u8 example
        //          in the note above)
        // both of these could benefit from a return of the number that actually got set.
        let field_as_u8_quote = match self.ty {
            FieldDataType::Char(_, _) |

            FieldDataType::Number(_, _, _)
            | FieldDataType::Boolean => {
                quote!{(#field_access_quote as u8)}
            }
            FieldDataType::Enum(_, _, _) => field_access_quote,
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(self.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Float(_, _) => return Err(syn::Error::new(self.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad.")),
        };
        let not_mask = !mask;
        let starting_inject_byte = quote_info.starting_inject_byte();
        let clear_quote = quote! {
            output_byte_buffer[#starting_inject_byte] &= #not_mask;
        };
        let mut source = quote! {#field_as_u8_quote};
        if shift_left != 0 {
            source = quote! {(#source << #shift_left)};
        }
        if mask != u8::MAX {
            source = quote! {#source & #mask};
        }
        let apply_field_to_buffer = quote! {
            output_byte_buffer[#starting_inject_byte] |= #source;
        };
        Ok((apply_field_to_buffer, clear_quote))
    }
    fn get_write_le_multi_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        right_shift: i8,
        first_bit_mask: u8,
        last_bit_mask: u8,
        field_access_quote: TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        // make a name for the buffer that we will store the number in byte form
        let field_buffer_name = quote_info.field_buffer_name();
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let mut full_quote = match self.ty {
            FieldDataType::Enum(_, _, _) |
            FieldDataType::Number(_, _, _) |
            FieldDataType::Float(_, _) |
            FieldDataType::Char(_, _) => {
                let field_call = quote!{#field_access_quote.to_le_bytes()};
                let apply_field_to_buffer = quote! {
                    let mut #field_buffer_name = #field_call;
                };
                apply_field_to_buffer
            }
            FieldDataType::Boolean => return Err(syn::Error::new(self.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(self.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad."))
        };
        let fields_last_bits_index = quote_info.amount_of_bits().div_ceil(8) - 1;
        let current_bit_mask = get_right_and_mask(quote_info.available_bits_in_first_byte());
        #[allow(clippy::cast_possible_truncation)]
        let mid_shift: u32 = 8 - quote_info.available_bits_in_first_byte() as u32;
        let next_bit_mask = get_left_and_mask(mid_shift as usize);
        let mut i = 0;
        let mut clear_quote = quote! {};
        while i != fields_last_bits_index {
            let start = quote_info.offset_starting_inject_byte(i);
            let not_current_bit_mask = !current_bit_mask;
            if quote_info.available_bits_in_first_byte() == 0 && right_shift == 0 {
                full_quote = quote! {
                    #full_quote
                    output_byte_buffer[#start] |= #field_buffer_name[#i] & #current_bit_mask;
                };
                clear_quote = quote! {
                    #clear_quote
                    output_byte_buffer[#start] &= #not_current_bit_mask;
                };
            } else {
                clear_quote = quote! {
                    #clear_quote
                    output_byte_buffer[#start] &= #not_current_bit_mask;
                };
                if mid_shift != 0 {
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] = #field_buffer_name[#i].rotate_right(#mid_shift);
                    };
                }
                if quote_info.available_bits_in_first_byte() + (8 * i) < quote_info.amount_of_bits()
                    && current_bit_mask != 0
                {
                    if current_bit_mask == u8::MAX {
                        full_quote = quote! {
                            #full_quote
                            output_byte_buffer[#start] |= #field_buffer_name[#i];
                        };
                    } else {
                        full_quote = quote! {
                            #full_quote
                            output_byte_buffer[#start] |= #field_buffer_name[#i] & #current_bit_mask;
                        };
                    }
                }
                let next_index = quote_info.next_index(start);
                if next_bit_mask == u8::MAX {
                    full_quote = quote! {
                        #full_quote
                        output_byte_buffer[#next_index] |= #field_buffer_name[#i];
                    };
                } else if next_bit_mask != 0 {
                    full_quote = quote! {
                        #full_quote
                        output_byte_buffer[#next_index] |= #field_buffer_name[#i] & #next_bit_mask;
                    };
                }
            }
            i += 1;
        }
        // bits used after applying the first_bit_mask one more time.
        let used_bits = quote_info.available_bits_in_first_byte() + (8 * i);
        let start = quote_info.offset_starting_inject_byte(i);
        if right_shift > 0 {
            let right_shift: u32 = u32::from(right_shift.unsigned_abs());
            // let not_first_bit_mask = !first_bit_mask;
            // let not_last_bit_mask = !last_bit_mask;

            full_quote = quote! {
                #full_quote
                #field_buffer_name[#i] = #field_buffer_name[#i].rotate_right(#right_shift);
            };
            if used_bits < quote_info.amount_of_bits() {
                clear_quote = quote! {
                    #clear_quote
                    output_byte_buffer[#start] &= 0;
                };
                let next_index = quote_info.next_index(start);
                full_quote = quote! {
                    #full_quote
                    output_byte_buffer[#start] |= #field_buffer_name[#i] & #first_bit_mask;
                    output_byte_buffer[#next_index] |= #field_buffer_name[#i] & #last_bit_mask;
                };
            } else {
                let mut last_mask = first_bit_mask;
                if quote_info.amount_of_bits() <= used_bits {
                    last_mask &= !get_right_and_mask(used_bits - quote_info.amount_of_bits());
                }
                let not_last_mask = !last_mask;
                clear_quote = quote! {
                    #clear_quote
                    output_byte_buffer[#start] &= #not_last_mask;
                };
                full_quote = quote! {
                    #full_quote
                    output_byte_buffer[#start] |= #field_buffer_name[#i] & #last_mask;
                };
            }
        } else {
            // this should give us the last index of the field
            let left_shift: u32 = u32::from(right_shift.unsigned_abs());
            let mut last_mask = first_bit_mask;
            if quote_info.amount_of_bits() <= used_bits {
                last_mask &= !get_right_and_mask(used_bits - quote_info.amount_of_bits());
            }
            let not_last_mask = !last_mask;
            clear_quote = quote! {
                #clear_quote
                output_byte_buffer[#start] &= #not_last_mask;
            };
            let mut finalize = quote! {#field_buffer_name[#i]};
            if left_shift != 0 && left_shift != 8 {
                finalize = quote! {(#finalize.rotate_left(#left_shift))};
            }
            if last_mask != u8::MAX {
                finalize = quote! {#finalize & #last_mask};
            }
            if last_mask != 0 {
                full_quote = quote! {
                    #full_quote
                    output_byte_buffer[#start] |= #finalize;
                };
            }
        }

        Ok((full_quote, clear_quote))
    }
    fn get_read_ne_single_byte_quote(
        &self,
        quote_info: &QuoteInfo,
    ) -> syn::Result<TokenStream> {
        if 8 < (quote_info.zeros_on_left() + quote_info.amount_of_bits()) {
            return Err(syn::Error::new(
                self.ident.span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (quote_info.zeros_on_left() + quote_info.amount_of_bits());
        let mask = get_right_and_mask(quote_info.available_bits_in_first_byte())
            & get_left_and_mask(8 - zeros_on_right);
        if 8 < quote_info.amount_of_bits() || 8 - quote_info.amount_of_bits() < self.attrs.bit_range.start % 8 {
            return Err(syn::Error::new(
                self.ident.span(),
                "calculating ne shift_left failed",
            ));
        }
        let starting_inject_byte = quote_info.starting_inject_byte();
        let output = match self.ty {
            FieldDataType::Number(_, _, _) => return Err(syn::Error::new(self.ident.span(), "Number was not given Endianness, please report this")),
            FieldDataType::Boolean => {
                quote!{(((input_byte_buffer[#starting_inject_byte] & #mask)) != 0)}
            }
            FieldDataType::Char(_, _) => return Err(syn::Error::new(self.ident.span(), "Char not supported for single byte insert logic")),
            FieldDataType::Enum(_, _, _) => return Err(syn::Error::new(self.ident.span(), "Enum was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Struct(_, _) => {
                let used_bits_in_byte = 8 - quote_info.available_bits_in_first_byte();
                quote!{([((input_byte_buffer[#starting_inject_byte] & #mask)) << #used_bits_in_byte])}
            }
            FieldDataType::Float(_, _) => return Err(syn::Error::new(self.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };
        Ok(output)
    }
    fn get_read_ne_multi_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        right_shift: i8,
    ) -> syn::Result<TokenStream> {
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let full_quote = match self.ty {
            FieldDataType::Number(_, _,_ ) |
            FieldDataType::Float(_, _) |
            FieldDataType::Char(_, _) => return Err(syn::Error::new(self.ident.span(), "Char was not given Endianness, please report this.")),
            FieldDataType::Boolean => return Err(syn::Error::new(self.ident.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Enum(_, _, _) => return Err(syn::Error::new(self.ident.span(), "Enum was not given Endianness, please report this.")),
            FieldDataType::Struct(ref size, _) => {
                let buffer_ident = format_ident!("{}_buffer", self.ident().ident());
                let mut quote_builder = quote!{let mut #buffer_ident: [u8;#size] = [0u8;#size];};
                match right_shift.cmp(&0) {
                    Ordering::Greater => {
                        // right shift (this means that the last bits are in the first byte)
                        // because we are applying bits in place we need masks in insure we don't effect other fields
                        // data. we need one for the first byte and the last byte.
                        let current_bit_mask = get_right_and_mask(quote_info.available_bits_in_first_byte());
                        let next_bit_mask = get_left_and_mask(8 - quote_info.available_bits_in_first_byte());
                        let right_shift: u32 = u32::from(right_shift.unsigned_abs());
                        for i in 0..*size {
                            let start = quote_info.offset_starting_inject_byte(i);
                            let mut first = if current_bit_mask == u8::MAX {
                                quote!{
                                    #buffer_ident[#i] = input_byte_buffer[#start];
                                }
                            }else{
                                quote!{
                                    #buffer_ident[#i] = input_byte_buffer[#start] & #current_bit_mask;
                                }
                            };
                            if quote_info.available_bits_in_first_byte() + (8 * i) < quote_info.amount_of_bits() && next_bit_mask != 0 {
                                let next_index = quote_info.next_index(start);
                                first = quote!{
                                    #first
                                    #buffer_ident[#i] |= input_byte_buffer[#next_index] & #next_bit_mask;
                                };
                            }
                            quote_builder = quote!{
                                #quote_builder
                                #first
                                #buffer_ident[#i] = #buffer_ident[#i].rotate_left(#right_shift);
                            };
                        }
                    }
                    Ordering::Less => {
                        return Err(syn::Error::new(
                            self.ident.span(),
                            "left shifting struct was removed to see if it would ever happen",
                        ));
                        //TODO this might be impossible for structs
                        // left shift (this means that the last bits are in the first byte)
                        // because we are applying bits in place we need masks in insure we don't effect other fields
                        // data. we need one for the first byte and the last byte.
                        /*let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);
                        let next_bit_mask = get_left_and_mask(8 - available_bits_in_first_byte);
                        let left_shift = right_shift.clone().abs() as u32;
                        for i in 0..*size {
                            let start = if let None = flip {starting_inject_byte + i}else{starting_inject_byte - i};
                            let mut first = quote!{
                                #buffer_ident[#i] = input_byte_buffer[#start] & #current_bit_mask;
                            };
                            if i + 1 <= *size {
                                first = quote!{
                                    #first
                                    #buffer_ident[#i] = input_byte_buffer[#start #operator 1] & #next_bit_mask;
                                };
                            }
                            quote_builder = quote!{
                                #quote_builder
                                #first
                                #buffer_ident[#i] = #buffer_ident[#i].rotate_right(#left_shift);
                            };
                        }*/
                    }
                    Ordering::Equal => {
                        // no shift can be more faster.
                        let current_bit_mask = get_right_and_mask(quote_info.available_bits_in_first_byte());
                        for i in 0..*size {
                            let start = quote_info.offset_starting_inject_byte(i);
                            if i == 0{
                                quote_builder = quote!{
                                    #quote_builder
                                    #buffer_ident[#i] = input_byte_buffer[#start] & #current_bit_mask;
                                };
                            }else{
                                quote_builder = quote!{
                                    #quote_builder
                                    #buffer_ident[#i] = input_byte_buffer[#start];
                                }
                            }
                        }
                    }
                }
                // return the value
                quote_builder = quote!{
                    #quote_builder
                    #buffer_ident
                };
                quote_builder
            }
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };

        Ok(full_quote)
    }
    fn get_write_ne_single_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        field_access_quote: TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (quote_info.zeros_on_left() + quote_info.amount_of_bits()) {
            return Err(syn::Error::new(
                self.ident.span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (quote_info.zeros_on_left() + quote_info.amount_of_bits());
        // combining the left and right masks will give us a mask that keeps the amount og bytes we
        // have in the position we need them to be in for this byte. we use available_bytes for
        // right mask because param is amount of 1's on the side specified (right), and
        // available_bytes is (8 - zeros_on_left) which is equal to ones_on_right.
        let mask = get_right_and_mask(quote_info.available_bits_in_first_byte())
            & get_left_and_mask(8 - zeros_on_right);
        // calculate how many left shifts need to occur to the number in order to position the bytes
        // we want to keep in the position we want.
        if 8 < quote_info.amount_of_bits() || 8 - quote_info.amount_of_bits() < self.attrs.bit_range.start % 8 {
            return Err(syn::Error::new(
                self.ident.span(),
                "calculating ne shift_left failed",
            ));
        }
        let shift_left = (8 - quote_info.amount_of_bits()) - (self.attrs.bit_range.start % 8);
        let starting_inject_byte = quote_info.starting_inject_byte();
        let not_mask = !mask;
        let clear_quote = quote! {
            output_byte_buffer[#starting_inject_byte] &= #not_mask;
        };
        // a quote that puts the field into a byte buffer we assume exists (because this is a
        // fragment).
        // NOTE the mask used here is only needed if we can NOT guarantee the field is only using the
        // bits the size attribute says it can. for example if our field is a u8 but the bit_length
        // attribute say to only use 2 bits, then the possible values are 0-3. so if the u8 (0-255)
        // is set to 4 then the extra bit being used will be dropped by the mask making the value 0.
        // FEATURE remove the "#mask & " from this quote to make it faster. but that means the
        // numbers have to be correct. if you want the no-mask feature then suggested enforcement of
        // the number would be:
        //      - generate setters that make a mask that drops bits not desired. (fast)
        //      - generate setters that check if its above max_value for the bit_length and set it
        //          to the max_value if its larger. (prevents situations like the 2bit u8 example
        //          in the note above)
        // both of these could benefit from a return of the number that actually got set.
        let finished_quote = match self.ty {
            FieldDataType::Number(_, _, _) => return Err(syn::Error::new(self.ident.span(), "Number was not given Endianness, please report this")),
            FieldDataType::Boolean => {
                quote!{output_byte_buffer[#starting_inject_byte] |= ((#field_access_quote as u8) << #shift_left) & #mask;}
            }
            FieldDataType::Char(_, _) => return Err(syn::Error::new(self.ident.span(), "Char not supported for single byte insert logic")),
            FieldDataType::Enum(_, _, _) => return Err(syn::Error::new(self.ident.span(), "Enum was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Struct(_, _) => {
                let used_bits_in_byte = 8 - quote_info.available_bits_in_first_byte();
                quote!{output_byte_buffer[#starting_inject_byte] |= (#field_access_quote.into_bytes()[0]) >> #used_bits_in_byte;}
            }
            FieldDataType::Float(_, _) => return Err(syn::Error::new(self.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };
        Ok((finished_quote, clear_quote))
    }
    fn get_write_ne_multi_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        right_shift: i8,
        field_access_quote: TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        // make a name for the buffer that we will store the number in byte form
        let field_buffer_name = format_ident!("{}_bytes", self.ident().ident());
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let (field_byte_buffer, size) = match self.ty {
            FieldDataType::Number(_, _,_ ) |
            FieldDataType::Float(_, _) |
            FieldDataType::Char(_, _) => return Err(syn::Error::new(self.span(), "Char was not given Endianness, please report this.")),
            FieldDataType::Boolean => return Err(syn::Error::new(self.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Enum(_, _, _) => return Err(syn::Error::new(self.span(), "Enum was not given Endianness, please report this.")),
            FieldDataType::Struct(ref size, _) => {
                let field_call = quote!{#field_access_quote.into_bytes()};
                let apply_field_to_buffer = quote! {
                    let mut #field_buffer_name = #field_call
                };
                (apply_field_to_buffer, *size)
            }
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };
        let mut clear_quote = quote! {};
        let mut full_quote = quote! {
            #field_byte_buffer;
        };
        // fill in the rest of the bits
        match right_shift.cmp(&0) {
            Ordering::Greater => {
                // right shift (this means that the last bits are in the first byte)
                // because we are applying bits in place we need masks in insure we don't effect other fields
                // data. we need one for the first byte and the last byte.
                let current_bit_mask = get_right_and_mask(quote_info.available_bits_in_first_byte());
                let next_bit_mask = get_left_and_mask(8 - quote_info.available_bits_in_first_byte());
                let right_shift: u32 = u32::from(right_shift.unsigned_abs());
                for i in 0usize..size {
                    let start = quote_info.offset_starting_inject_byte(i);
                    let not_current_bit_mask = !current_bit_mask;
                    let not_next_bit_mask = !next_bit_mask;
                    clear_quote = quote! {
                        #clear_quote
                        output_byte_buffer[#start] &= #not_current_bit_mask;
                    };
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] = #field_buffer_name[#i].rotate_right(#right_shift);
                        output_byte_buffer[#start] |= #field_buffer_name[#i] & #current_bit_mask;
                    };
                    let next_index = quote_info.next_index(start);
                    if quote_info.available_bits_in_first_byte() + (8 * i) < quote_info.amount_of_bits() {
                        if not_next_bit_mask != u8::MAX {
                            clear_quote = quote! {
                                #clear_quote
                                output_byte_buffer[#next_index] &= #not_next_bit_mask;
                            };
                        }
                        if next_bit_mask != 0 {
                            full_quote = quote! {
                                #full_quote
                                output_byte_buffer[#next_index] |= #field_buffer_name[#i] & #next_bit_mask;
                            };
                        }
                    }
                }
            }
            Ordering::Less => {
                return Err(syn::Error::new(
                    self.ident.span(),
                    "left shifting struct was removed to see if it would ever happen (please open issue on github)",
                ));
                /* left shift (this means that the last bits are in the first byte)
                // because we are applying bits in place we need masks in insure we don't effect other fields
                // data. we need one for the first byte and the last byte.
                let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);
                let next_bit_mask = get_left_and_mask(8 - available_bits_in_first_byte);
                let left_shift = right_shift.clone().abs() as u32;
                for i in 0usize..size {
                    let start = if let None = flip {starting_inject_byte + i}else{starting_inject_byte - i};
                    full_quote = quote!{
                        #full_quote
                        #field_buffer_name[#i] = #field_buffer_name[#i].rotate_left(#left_shift);
                        output_byte_buffer[#start] |= #field_buffer_name[#i] & #current_bit_mask;
                    };
                    if i + 1 != size {
                        full_quote = quote!{
                            #full_quote
                        }
                    }
                }*/
            }
            Ordering::Equal => {
                // no shift can be more faster.
                let current_bit_mask = get_right_and_mask(quote_info.available_bits_in_first_byte());

                for i in 0usize..size {
                    let start = quote_info.offset_starting_inject_byte(i);
                    let not_current_bit_mask = !current_bit_mask;
                    clear_quote = quote! {
                        #clear_quote
                        output_byte_buffer[#start] &= #not_current_bit_mask;
                    };
                    if i == 0 {
                        if current_bit_mask == u8::MAX {
                            full_quote = quote! {
                                #full_quote
                                output_byte_buffer[#start] |= #field_buffer_name[#i];
                            };
                        } else {
                            full_quote = quote! {
                                #full_quote
                                output_byte_buffer[#start] |= #field_buffer_name[#i] & #current_bit_mask;
                            };
                        }
                    } else {
                        full_quote = quote! {
                            #full_quote
                            output_byte_buffer[#start] |= #field_buffer_name[#i];
                        };
                    }
                }
            }
        }
        Ok((full_quote, clear_quote))
    }
    fn get_read_be_single_byte_quote(
        &self,
        quote_info: &QuoteInfo,
    ) -> syn::Result<TokenStream> {
        todo!("write be generate fns")
    }
    fn get_read_be_multi_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        right_shift: i8,
        first_bit_mask: u8,
        last_bit_mask: u8,
    ) -> syn::Result<TokenStream> {
        todo!("write be generate fns")
    }
    fn get_write_be_single_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        field_access_quote: TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        todo!("write be generate fns")
    }
    fn get_write_be_multi_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        right_shift: i8,
        first_bit_mask: u8,
        last_bit_mask: u8,
        field_access_quote: TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        todo!("write be generate fns")
    }
}

fn isolate_bit_index_mask(bit_index: usize) -> u8 {
    match bit_index {
        1 => 0b0100_0000,
        2 => 0b0010_0000,
        3 => 0b0001_0000,
        4 => 0b0000_1000,
        5 => 0b0000_0100,
        6 => 0b0000_0010,
        7 => 0b0000_0001,
        _ => 0b1000_0000,
    }
}
fn rotate_primitive_vec(prim: Vec<u8>, right_shift: i8, field: &FieldInfo) -> syn::Result<Vec<u8>> {
    // REMEMBER SHIFTS ARE BACKWARD BECAUSE YOU COPIED AND PASTED into_bytes
    if right_shift == 0 {
        return Ok(prim);
    }
    let output = match prim.len() {
        1 => {
            let mut temp = u8::from_be_bytes([prim[0]]);
            match right_shift {
                i8::MIN..=-1 => {
                    let left_shift = -right_shift;
                    temp = temp.rotate_left(u32::from(left_shift.unsigned_abs()));
                }
                0..=i8::MAX => {
                    temp = temp.rotate_right(u32::from(right_shift.unsigned_abs()));
                }
            }
            temp.to_be_bytes().to_vec()
        }
        2 => {
            let mut temp = u16::from_be_bytes([prim[0], prim[1]]);
            match right_shift {
                i8::MIN..=-1 => {
                    let left_shift = -right_shift;
                    temp = temp.rotate_left(u32::from(left_shift.unsigned_abs()));
                }
                0..=i8::MAX => {
                    temp = temp.rotate_right(u32::from(right_shift.unsigned_abs()));
                }
            }
            temp.to_be_bytes().to_vec()
        }
        4 => {
            let mut temp = u32::from_be_bytes([prim[0], prim[1], prim[2], prim[3]]);
            match right_shift {
                i8::MIN..=-1 => {
                    let left_shift = -right_shift;
                    temp = temp.rotate_left(u32::from(left_shift.unsigned_abs()));
                }
                0..=i8::MAX => {
                    temp = temp.rotate_right(u32::from(right_shift.unsigned_abs()));
                }
            }
            temp.to_be_bytes().to_vec()
        }
        8 => {
            let mut temp = u64::from_be_bytes([
                prim[0], prim[1], prim[2], prim[3], prim[4], prim[5], prim[6], prim[7],
            ]);
            match right_shift {
                i8::MIN..=-1 => {
                    let left_shift = -right_shift;
                    temp = temp.rotate_left(u32::from(left_shift.unsigned_abs()));
                }
                0..=i8::MAX => {
                    temp = temp.rotate_right(u32::from(right_shift.unsigned_abs()));
                }
            }
            temp.to_be_bytes().to_vec()
        }
        16 => {
            let mut temp = u128::from_be_bytes([
                prim[0], prim[1], prim[2], prim[3], prim[4], prim[5], prim[6], prim[7], prim[8],
                prim[9], prim[10], prim[11], prim[12], prim[13], prim[14], prim[15],
            ]);
            match right_shift {
                i8::MIN..=-1 => {
                    let left_shift = -right_shift;
                    temp = temp.rotate_left(u32::from(left_shift.unsigned_abs()));
                }
                0..=i8::MAX => {
                    temp = temp.rotate_right(u32::from(right_shift.unsigned_abs()));
                }
            }
            temp.to_be_bytes().to_vec()
        }
        _ => {
            return Err(syn::Error::new(
                field.ident.span(),
                "invalid primitive size",
            ));
        }
    };
    Ok(output)
}

fn add_sign_fix_quote(
    field: &FieldInfo,
    amount_of_bits: usize,
    right_shift: i8,
) -> syn::Result<Option<TokenStream>> {
    if let FieldDataType::Number(ref size, ref sign, _) = field.ty {
        if amount_of_bits != size * 8 {
            if let NumberSignage::Signed = sign {
                let (bit_to_isolate, sign_index) = match field.attrs.endianness.as_ref() {
                    Endianness::Big => (
                        field.attrs.bit_range.start % 8,
                        field.attrs.bit_range.start / 8,
                    ),
                    Endianness::Little => {
                        let skip_bytes = (amount_of_bits / 8) * 8;
                        let sign_bit_index = field.attrs.bit_range.start + skip_bytes;
                        // TODO fix bit isolators to fix signed numbers.
                        (sign_bit_index % 8, sign_bit_index / 8)
                    }
                    Endianness::None => return Ok(None),
                };
                let sign_mask = isolate_bit_index_mask(bit_to_isolate);
                let sign_bit = quote! {
                    (input_byte_buffer[#sign_index] & #sign_mask)
                };
                let mut unused_bits = (size * 8) - amount_of_bits;
                let mut buffer: VecDeque<u8> = VecDeque::default();
                for _i in 0..*size {
                    if unused_bits > 7 {
                        buffer.push_back(get_left_and_mask(8));
                        unused_bits -= 8;
                    } else if unused_bits != 0 {
                        buffer.push_back(get_left_and_mask(unused_bits));
                        unused_bits = 0;
                    } else {
                        buffer.push_back(get_left_and_mask(0));
                    }
                }
                let mut bit_buffer: Punctuated<u8, Comma> = Punctuated::default();
                match field.attrs.endianness.as_ref() {
                    Endianness::Big => {
                        buffer = VecDeque::from(rotate_primitive_vec(
                            buffer.into(),
                            right_shift,
                            field,
                        )?);
                        while {
                            if let Some(c) = buffer.pop_front() {
                                bit_buffer.push(c);
                                true
                            } else {
                                false
                            }
                        } {}
                    }
                    Endianness::Little => {
                        match right_shift.cmp(&0) {
                            Ordering::Greater => {
                                buffer = buffer
                                    .into_iter()
                                    .map(|x| x.rotate_right(u32::from(right_shift.unsigned_abs())))
                                    .collect();
                            }
                            Ordering::Less => {
                                let left_shift = u32::from(right_shift.unsigned_abs());
                                buffer = buffer
                                    .into_iter()
                                    .map(|x| x.rotate_left(left_shift))
                                    .collect();
                            }
                            Ordering::Equal => {}
                        }
                        while {
                            if let Some(c) = buffer.pop_back() {
                                bit_buffer.push(c);
                                true
                            } else {
                                false
                            }
                        } {}
                    }
                    Endianness::None => return Ok(None),
                }
                return Ok(Some(quote! {
                    if #sign_bit == #sign_mask {[#bit_buffer]} else {[0u8;#size]}
                }));
            }
        }
    }
    Ok(None)
}

fn add_sign_fix_quote_single_bit(
    field_access: TokenStream,
    field: &FieldInfo,
    amount_of_bits: usize,
    byte_index: usize,
) -> TokenStream {
    if let FieldDataType::Number(ref size, ref sign, _) = field.ty {
        if amount_of_bits != *size * 8 {
            if let NumberSignage::Signed = sign {
                let bit_to_isolate = field.attrs.bit_range.start % 8;
                let sign_mask = isolate_bit_index_mask(bit_to_isolate);
                let neg_mask = get_left_and_mask(bit_to_isolate + 1);
                let sign_bit = quote! {
                    (input_byte_buffer[#byte_index] & #sign_mask)
                };
                let add_me = quote! {
                    if #sign_bit == #sign_mask {#neg_mask | #field_access} else {0u8 | #field_access}
                };
                return add_me;
            }
        }
    }
    field_access
}
