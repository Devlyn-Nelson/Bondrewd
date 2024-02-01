//! This file is an effort to merge from and into bytes, which is being delayed for now.
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::structs::common::{get_left_and_mask, get_right_and_mask};

use super::common::{Endianness, FieldInfo, StructInfo};

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
    pub fn available_bits_in_first_byte(&self) -> usize {
        self.available_bits_in_first_byte
    }
    pub fn field_buffer_name(&self) -> usize {
        self.field_buffer_name
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

impl FieldInfo {
    pub fn get_quotes(&self, struct_info: &StructInfo) -> syn::Result<FieldQuotes> {
        let qi = QuoteInfo::new(self, struct_info)?;
        match *self.attrs.endianness {
            Endianness::Little => self.get_le_quotes(qi),
            Endianness::Big => self.get_be_quotes(qi),
            Endianness::None => self.get_ne_quotes(qi),
        }
    }
    fn get_le_quotes(&self, quote_info: QuoteInfo) -> Result<FieldQuotes, syn::Error> {
        if quote_info.amount_of_bits() > quote_info.available_bits_in_first_byte() {
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
        } else {
            // single bytes logic
        }
        todo!("merged Little Endianness from_bytes/into_bytes Generation code here")
    }
    fn get_le_multibyte_quote(
        &self,
        quote_info: QuoteInfo,
        right_shift: u8,
    ) -> syn::Result<FieldQuotes> {
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
        let fields_last_bits_index = quote_info.fields_last_bits_index();
        // ___________________START HERE____________________________
        let current_bit_mask = get_right_and_mask(quote_info.available_bits_in_first_byte());
        #[allow(clippy::cast_possible_truncation)]
        let mid_shift: u32 = 8 - quote_info.available_bits_in_first_byte() as u32;
        let next_bit_mask = get_left_and_mask(mid_shift as usize);
        let mut i = 0;
        while i != fields_last_bits_index {
            let start = quote_info.offset_starting_inject_byte(i);
            if available_bits_in_first_byte == 0 && right_shift == 0 {
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
                if available_bits_in_first_byte + (8 * i) < amount_of_bits && next_bit_mask != 0 {
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] |= input_byte_buffer[#start #operator 1] & #next_bit_mask;
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
        let used_bits = available_bits_in_first_byte + (8 * i);
        if right_shift > 0 {
            let start = if flip.is_none() {
                starting_inject_byte + i
            } else {
                starting_inject_byte - i
            };
            let right_shift: u32 = u32::from(right_shift.unsigned_abs());
            if used_bits < amount_of_bits {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#start] & #current_bit_mask;
                    #field_buffer_name[#i] |= input_byte_buffer[#start + 1] & #last_bit_mask;
                };
            } else {
                let mut last_mask = first_bit_mask;
                if amount_of_bits < used_bits {
                    last_mask &= !get_right_and_mask(used_bits - amount_of_bits);
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
            let start = if flip.is_none() {
                starting_inject_byte + i
            } else {
                starting_inject_byte - i
            };
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

        let output = match field.ty {
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
                    return Err(syn::Error::new(field.ident.span(), "unsupported floating type"))
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
            FieldDataType::Boolean => return Err(syn::Error::new(field.ident.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(field.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad."))
        };

        Ok(output)
    }
    fn get_ne_quotes(&self, _quote_info: QuoteInfo) -> Result<FieldQuotes, syn::Error> {
        todo!("merged None Endianness from_bytes/into_bytes Generation code here")
    }
    fn get_be_quotes(&self, _quote_info: QuoteInfo) -> Result<FieldQuotes, syn::Error> {
        todo!("merged Big Endianness from_bytes/into_bytes Generation code here")
    }
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
