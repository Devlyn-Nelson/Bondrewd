use std::{cmp::Ordering, collections::VecDeque};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma};

use crate::{
    build::field::NumberType,
    solved::field::{
        Resolver, ResolverArrayType, ResolverData, ResolverPrimitiveStrategy, ResolverSubType, ResolverType
    },
};

use super::{get_be_starting_index, get_left_and_mask, get_right_and_mask, ResolverDataBigAdditive, ResolverDataNestedAdditive};

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
fn rotate_primitive_vec(prim: Vec<u8>, right_shift: i8, span: Span) -> syn::Result<Vec<u8>> {
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
            return Err(syn::Error::new(span, "invalid primitive size"));
        }
    };
    Ok(output)
}

fn add_sign_fix_quote(
    field: &Resolver,
    amount_of_bits: usize,
    right_shift: i8,
) -> syn::Result<Option<TokenStream>> {
    if let ResolverType::Primitive {
        number_ty,
        resolver_strategy,
        rust_size,
    } = field.ty.as_ref()
    {
        let size = rust_size.bytes();
        if amount_of_bits != size {
            if let NumberType::Signed = number_ty {
                let (bit_to_isolate, sign_index) = match resolver_strategy {
                    ResolverPrimitiveStrategy::Standard => (
                        field.data.bit_range_start() % 8,
                        field.data.bit_range_start() / 8,
                    ),
                    ResolverPrimitiveStrategy::Alternate => {
                        let skip_bytes = (amount_of_bits / 8) * 8;
                        let sign_bit_index = field.data.bit_range_start() + skip_bytes;
                        // TODO fix bit isolators to fix signed numbers.
                        (sign_bit_index % 8, sign_bit_index / 8)
                    }
                };
                let sign_mask = isolate_bit_index_mask(bit_to_isolate);
                let sign_bit = quote! {
                    (input_byte_buffer[#sign_index] & #sign_mask)
                };
                let mut unused_bits = size - amount_of_bits;
                let mut buffer: VecDeque<u8> = VecDeque::default();
                for _i in 0..size {
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
                match resolver_strategy {
                    ResolverPrimitiveStrategy::Standard => {
                        buffer = VecDeque::from(rotate_primitive_vec(
                            buffer.into(),
                            right_shift,
                            field.ident().span(),
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
                    ResolverPrimitiveStrategy::Alternate => {
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
    field: &Resolver,
    amount_of_bits: usize,
    byte_index: usize,
) -> TokenStream {
    if let ResolverType::Primitive {
        number_ty,
        resolver_strategy,
        rust_size,
    } = field.ty.as_ref()
    {
        if amount_of_bits != rust_size.bytes() {
            if let NumberType::Signed = number_ty {
                let bit_to_isolate = field.data.bit_range_start() % 8;
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

impl Resolver {
    pub(crate) fn get_read_quote(
        &self,
        gen_read_fn: fn(&ResolverData, &TokenStream) -> syn::Result<TokenStream>,
    ) -> syn::Result<TokenStream> {
        let value_retrieval = match self.ty.as_ref() {
            ResolverType::Array {
                sub_ty,
                array_ty,
                sizings,
            } => match array_ty {
                ResolverArrayType::Element { .. } => {
                    let mut buffer = quote! {};
                    let Some(sub) =
                        ElementArrayIter::from_values(&self.data, sub_ty, array_ty, sizings)
                    else {
                        let ident = self.data.field_name.ident();
                        return Err(Error::new(
                            ident.span(),
                            format!("Failed to construct valid ElementArrayIter for `{ident}`"),
                        ));
                    };
                    // let sub = self.get_element_iter()?;
                    for sub_field in sub {
                        let sub_field_quote = Self::get_read_quote(&sub_field, gen_read_fn)?;
                        buffer = quote! {
                            #buffer
                            {#sub_field_quote},
                        };
                    }
                    let buffer = quote! { [#buffer] };
                    buffer
                }
                ResolverArrayType::Block { .. } => {
                    let mut buffer = quote! {};
                    let Some(sub) =
                        BlockArrayIter::from_values(&self.data, sub_ty, array_ty, sizings)
                    else {
                        let ident = self.data.field_name.ident();
                        return Err(Error::new(
                            ident.span(),
                            format!("Failed to construct valid ElementArrayIter for `{ident}`"),
                        ));
                    };
                    // let sub = self.get_block_iter()?;
                    for sub_field in sub {
                        let sub_field_quote = Self::get_read_quote(&sub_field, gen_read_fn)?;
                        buffer = quote! {
                            #buffer
                            {#sub_field_quote},
                        };
                    }
                    let buffer = quote! { [#buffer] };
                    buffer
                }
            },
            _ => {
                let quote_info: QuoteInfo = (self, struct_info).try_into()?;
                gen_read_fn(&self.data, &quote_info)?
            }
        };

        let output = match self.ty {
            DataType::Float { ref type_quote, .. } => {
                quote! {#type_quote::from_bits(#value_retrieval)}
            }
            DataType::Char { .. } => {
                quote! {
                    if let Some(c) = char::from_u32({
                        #value_retrieval
                    }) {
                        c
                    }else{
                        '�'
                    }
                }
            }
            DataType::Enum { ref type_quote, .. } => {
                quote! {#type_quote::from_primitive(#value_retrieval)}
            }
            DataType::Struct { ref type_quote, .. } => {
                quote! {#type_quote::from_bytes({#value_retrieval})}
            }
            _ => {
                quote! {#value_retrieval}
            }
        };
        Ok(output)
    }
    pub(crate) fn get_read_le_quote(
        &self,
        sub_ty: ResolverSubType,
        field_access_quote: &TokenStream,
    ) -> syn::Result<TokenStream> {
        if quote_info.amount_of_bits() > quote_info.available_bits_in_first_byte() {
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
            self.get_read_le_multi_byte_quote(quote_info)
        } else {
            self.get_read_le_single_byte_quote(quote_info)
        }
    }
    pub(crate) fn get_read_le_single_byte_quote(
        &self,
        quote_info: &QuoteInfo,
    ) -> syn::Result<TokenStream> {
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (quote_info.zeros_on_left() + quote_info.amount_of_bits()) {
            return Err(syn::Error::new(
                self.ident().span(),
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
        if 8 - quote_info.amount_of_bits() < self.data.bit_range_start() % 8 {
            return Err(syn::Error::new(
                self.ident().span(),
                format!(
                    "calculating be left_shift failed {} , {}",
                    quote_info.amount_of_bits(),
                    self.data.bit_range_start() % 8
                ),
            ));
        }
        let shift_left = (8 - quote_info.amount_of_bits()) - (self.data.bit_range_start() % 8);
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
        let output_quote = match self.ty.as_ref() {

            ResolverType::Array {
                sub_ty,
                array_ty,
                sizings,
            } => {
                return Err(syn::Error::new(
                    self.ident().span(),
                    "an array got passed into apply_be_math_to_field_access_quote, which is bad.",
                ));
            }
            ResolverType::Nested {
                ty_ident: _,
                rust_size: _,
            } => {
                return Err(syn::Error::new(self.ident().span(), "Struct was given Endianness which should be described by the struct implementing Bitfield"));
            }
            ResolverType::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            } => match number_ty {
                NumberType::Float => {
                    return Err(syn::Error::new(
                        self.ident().span(),
                        "Float not supported for single byte insert logic",
                    ))
                }
                NumberType::Unsigned => {
                    let mut field_value = quote! {((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left)};
                    quote! {
                        #field_value as #type_quote
                    }
                }
                NumberType::Signed => {
                    let mut field_value = quote! {((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left)};
                    field_value = add_sign_fix_quote_single_bit(
                        field_value,
                        self,
                        quote_info.amount_of_bits(),
                        starting_inject_byte,
                    );
                    let mut value = quote! {
                        let mut #field_buffer_name = #field_value;
                    };
                    value = quote! {
                        {
                            #value
                            #field_buffer_name as #type_quote
                        }
                    };
                    value
                }
                NumberType::Char => {
                    quote! {((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left) as u32}
                }
                NumberType::Bool => {
                    quote! {(input_byte_buffer[#starting_inject_byte] & #mask) != 0}
                }
            },
        };
        Ok(output_quote)
    }
    pub(crate) fn get_read_le_multi_byte_quote(
        &self,
        quote_info: &QuoteInfo,
    ) -> syn::Result<TokenStream> {
        // calculate how many of the bits will be inside the least significant byte we are adding to.
        // this will also be the number used for shifting to the right >> because that will line up
        // our bytes for the buffer.
        if quote_info.amount_of_bits() < quote_info.available_bits_in_first_byte() {
            return Err(syn::Error::new(
                self.ident().span(),
                "calculating le `bits_in_last_bytes` failed, amount of bit is less than available bits in first byte",
            ));
        }
        // calculate how many of the bits will be inside the least significant byte we are adding to.
        // this will also be the number used for shifting to the right >> because that will line up
        // our bytes for the buffer.
        let (right_shift, first_bit_mask, last_bit_mask): (i8, u8, u8) = {
            let thing: LittleQuoteInfo = quote_info.into();
            (thing.right_shift, thing.first_bit_mask, thing.last_bit_mask)
        };
        let rust_type_size = self.ty.rust_size();
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
            DataType::Number{ref type_quote, ..} |
            DataType::Enum{ref type_quote, ..} => {
                let apply_field_to_buffer = quote! {
                    #type_quote::from_le_bytes({
                        #full_quote
                    })
                };
                apply_field_to_buffer
            }
            DataType::Float{..} => {
                let alt_type_quote = if rust_type_size == 4 {
                    quote!{u32}
                }else if rust_type_size == 8 {
                    quote!{u64}
                }else{
                    return Err(syn::Error::new(self.ident().span(), "unsupported floating type"))
                };
                let apply_field_to_buffer = quote! {
                    #alt_type_quote::from_le_bytes({
                        #full_quote
                    })
                };
                apply_field_to_buffer
            }
            DataType::Char{..} => {
                let apply_field_to_buffer = quote! {
                    u32::from_le_bytes({
                        #full_quote
                    })
                };
                apply_field_to_buffer
            }
            DataType::Boolean => return Err(syn::Error::new(self.ident().span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            DataType::Struct{..} => return Err(syn::Error::new(self.ident().span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            DataType::ElementArray{..} | DataType::BlockArray{..} => return Err(syn::Error::new(self.ident().span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad."))
        };

        Ok(output)
    }
    pub(crate) fn get_read_ne_quote(&self, quote_info: &QuoteInfo) -> syn::Result<TokenStream> {
        if quote_info.amount_of_bits > quote_info.available_bits_in_first_byte {
            // how many times to shift the number right.
            // NOTE if negative shift left.
            // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
            // for a f32) then use the last byte in the fields byte array after shifting for the first
            // used byte in the buffer.
            if 8 < quote_info.available_bits_in_first_byte() % 8 {
                return Err(syn::Error::new(
                    self.ident().span(),
                    "calculating ne right_shift failed",
                ));
            }
            self.get_read_ne_multi_byte_quote(quote_info)
        } else {
            self.get_read_ne_single_byte_quote(quote_info)
        }
    }
    pub(crate) fn get_read_ne_single_byte_quote(
        &self,
        quote_info: &QuoteInfo,
    ) -> syn::Result<TokenStream> {
        // how many times to shift the number right.
        // NOTE if negative shift left.
        // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        if 8 < quote_info.available_bits_in_first_byte() % 8 {
            return Err(syn::Error::new(
                self.ident().span(),
                "calculating ne right_shift failed",
            ));
        }
        if 8 < (quote_info.zeros_on_left() + quote_info.amount_of_bits()) {
            return Err(syn::Error::new(
                self.ident().span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (quote_info.zeros_on_left() + quote_info.amount_of_bits());
        let mask = get_right_and_mask(quote_info.available_bits_in_first_byte())
            & get_left_and_mask(8 - zeros_on_right);
        if 8 < quote_info.amount_of_bits()
            || 8 - quote_info.amount_of_bits() < self.data.bit_range_start() % 8
        {
            return Err(syn::Error::new(
                self.ident().span(),
                "calculating ne shift_left failed",
            ));
        }
        let starting_inject_byte = quote_info.starting_inject_byte();
        let output = match self.ty {
            DataType::Number{..} => self.get_read_be_single_byte_quote(quote_info)?,
            DataType::Boolean => {
                quote!{(((input_byte_buffer[#starting_inject_byte] & #mask)) != 0)}
            }
            DataType::Char{..} => return Err(syn::Error::new(self.ident().span(), "Char not supported for single byte insert logic")),
            DataType::Enum{..} => return Err(syn::Error::new(self.ident().span(), "Enum was given Endianness which should be described by the struct implementing Bitfield")),
            DataType::Struct{..} => {
                let used_bits_in_byte = 8 - quote_info.available_bits_in_first_byte();
                quote!{([((input_byte_buffer[#starting_inject_byte] & #mask)) << #used_bits_in_byte])}
            }
            DataType::Float{..} => return Err(syn::Error::new(self.ident().span(), "Float not supported for single byte insert logic")),
            DataType::ElementArray{..} | DataType::BlockArray{..} => return Err(syn::Error::new(self.ident().span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };
        Ok(output)
    }
    pub(crate) fn get_read_ne_multi_byte_quote(
        &self,
        quote_info: &QuoteInfo,
    ) -> syn::Result<TokenStream> {
        // how many times to shift the number right.
        // NOTE if negative shift left.
        // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        if 8 < quote_info.available_bits_in_first_byte() % 8 {
            return Err(syn::Error::new(
                self.ident().span(),
                "calculating ne right_shift failed",
            ));
        }
        let right_shift: i8 = {
            let thing: ResolverDataNestedAdditive = self.data.as_ref().into();
            thing.right_shift
        };
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let full_quote = match self.ty.as_ref() {
            ResolverType::Primitive { .. } => return Err(syn::Error::new(self.ident().span(), "Primitive was not given Endianness, please report this.")),
            ResolverType::Nested{rust_size: size, ..} => {
                let buffer_ident = format_ident!("{}_buffer", self.ident());
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
                            self.ident().span(),
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
            ResolverType::Array { .. } => return Err(syn::Error::new(self.ident().span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };

        Ok(full_quote)
    }

    pub(crate) fn get_read_be_quote(&self, sub_ty: ResolverSubType) -> syn::Result<TokenStream> {
        if self.bit_length() > self.available_bits_in_first_byte() {
            // calculate how many of the bits will be inside the least significant byte we are adding to.
            // this will also be the number used for shifting to the right >> because that will line up
            // our bytes for the buffer.
            if self.bit_length() < self.available_bits_in_first_byte() {
                return Err(syn::Error::new(
                    self.ident().span(),
                    "calculating be bits_in_last_bytes failed",
                ));
            }
            self.get_read_be_multi_byte_quote(sub_ty)
        } else {
            self.get_read_be_single_byte_quote(sub_ty)
        }
    }
    pub(crate) fn get_read_be_single_byte_quote(
        &self,
        sub_ty: ResolverSubType,
    ) -> syn::Result<TokenStream> {
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (self.zeros_on_left() + self.bit_length()) {
            return Err(syn::Error::new(
                self.ident().span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (self.zeros_on_left() + self.bit_length());
        // combining the left and right masks will give us a mask that keeps the amount og bytes we
        // have in the position we need them to be in for this byte. we use available_bytes for
        // right mask because param is amount of 1's on the side specified (right), and
        // available_bytes is (8 - zeros_on_left) which is equal to ones_on_right.
        let mask = get_right_and_mask(self.available_bits_in_first_byte())
            & get_left_and_mask(8 - zeros_on_right);
        // calculate how many left shifts need to occur to the number in order to position the bytes
        // we want to keep in the position we want.
        if 8 - self.bit_length() < self.data.bit_range_start() % 8 {
            return Err(syn::Error::new(
                self.ident().span(),
                format!(
                    "calculating be left_shift failed {} , {}",
                    self.bit_length(),
                    self.data.bit_range_start() % 8
                ),
            ));
        }
        let shift_left = (8 - self.bit_length()) - (self.data.bit_range_start() % 8);
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
        let field_buffer_name = self.field_buffer_name();
        let starting_inject_byte = self.starting_inject_byte();
        let type_quote = self.ty.get_type_ident();
        let output_quote = match self.ty.as_ref() {
            ResolverType::Array {
                sub_ty,
                array_ty,
                sizings,
            } => {
                return Err(syn::Error::new(
                    self.ident().span(),
                    "an array got passed into apply_be_math_to_field_access_quote, which is bad.",
                ));
            }
            ResolverType::Nested {
                ty_ident,
                rust_size,
            } => {
                return Err(syn::Error::new(self.ident().span(), "Struct was given Endianness which should be described by the struct implementing Bitfield"));
            }
            ResolverType::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            } => match number_ty {
                NumberType::Float => {
                    return Err(syn::Error::new(
                        self.ident().span(),
                        "Float not supported for single byte insert logic",
                    ))
                }
                NumberType::Unsigned => {
                    let field_value = quote! {((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left)};
                    quote! {#field_value as #type_quote}
                }
                NumberType::Signed => {
                    let mut field_value = quote! {((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left)};
                    field_value = add_sign_fix_quote_single_bit(
                        field_value,
                        self,
                        self.bit_length(),
                        self.starting_inject_byte(),
                    );
                    let mut value = quote! {
                        let mut #field_buffer_name = #field_value;
                    };
                    value = quote! {
                        {
                            #value
                            #field_buffer_name as #type_quote
                        }
                    };
                    value
                }
                NumberType::Bool => {
                    quote! {(input_byte_buffer[#starting_inject_byte] & #mask) != 0}
                }
                NumberType::Char => {
                    quote! {((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left) as u32}
                }
            },
        };
        Ok(output_quote)
    }
    pub(crate) fn get_read_be_multi_byte_quote(
        &self,
        sub_ty: ResolverSubType,
        // field_access_quote: &TokenStream,
    ) -> syn::Result<TokenStream> {
        let (right_shift, first_bit_mask, last_bit_mask, bits_in_last_byte): (i8, u8, u8, usize) = {
            let thing: ResolverDataBigAdditive = self.data.as_ref().into();
            (
                thing.right_shift,
                thing.first_bit_mask,
                thing.last_bit_mask,
                thing.bits_in_last_byte,
            )
        };
        // create a quote that holds the bit shifting operator and shift value and the field name.
        // first_bits_index is the index to use in the fields byte array after shift for the
        // starting byte in the byte buffer. when left shifts happen on full sized numbers the last
        // index of the fields byte array will be used.
        let (shift, first_bits_index) = if right_shift < 0 {
            // convert to left shift using absolute value
            let left_shift: u32 = u32::from(right_shift.unsigned_abs());
            // shift left code
            (
                quote! { .rotate_right(#left_shift) },
                // if the size of the field type is the same as the bit size going into the
                // bit_buffer then we use the last byte for applying to the buffers first effected
                // byte.
                if self.ty.rust_size() * 8 == self.bit_length() {
                    self.ty.rust_size() - 1
                } else {
                    match get_be_starting_index(
                        self.bit_length(),
                        right_shift,
                        self.ty.rust_size(),
                    ) {
                        Ok(good) => good,
                        Err(err) => {
                            return Err(syn::Error::new(
                                self.ident().span(),
                                format!("{err} (from 1)"),
                            ))
                        }
                    }
                },
            )
        } else {
            (
                if right_shift == 0 {
                    // no shift no code, just the
                    quote! {}
                } else {
                    // shift right code
                    let right_shift_usize: u32 = u32::from(right_shift.unsigned_abs());
                    quote! { .rotate_left(#right_shift_usize) }
                },
                match get_be_starting_index(
                    self.bit_length(),
                    right_shift,
                    self.ty.rust_size(),
                ) {
                    Ok(good) => good,
                    Err(err) => {
                        return Err(syn::Error::new(
                            self.ident().span(),
                            format!("{err} (from 2)"),
                        ))
                    }
                },
            )
        };
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let ty_ident = self.ty.get_type_ident();
        let output = match self.ty.as_ref() {
            ResolverType::Primitive { number_ty, resolver_strategy, rust_size } => {
                match number_ty {
                    NumberType::Float =>{
                        // let info = BuildNumberQuotePackage { amount_of_bits: quote_info.amount_of_bits(), bits_in_last_byte, field_buffer_name: quote_info.field_buffer_name(), rust_size, first_bits_index, starting_inject_byte: quote_info.starting_inject_byte(), first_bit_mask, last_bit_mask, right_shift, available_bits_in_first_byte: quote_info.available_bits_in_first_byte(), flip: quote_info.flip()};
                        let full_quote = build_be_number_quote(self, first_bits_index)?;
                        let apply_field_to_buffer = quote! {
                            #ty_ident::from_be_bytes({
                                #full_quote
                            })#shift
                        };
                        apply_field_to_buffer
                    }
                    NumberType::Unsigned |
                    NumberType::Signed => {
                        // let info = BuildNumberQuotePackage { amount_of_bits: quote_info.amount_of_bits(), bits_in_last_byte, field_buffer_name: quote_info.field_buffer_name(), rust_size, first_bits_index, starting_inject_byte: quote_info.starting_inject_byte(), first_bit_mask, last_bit_mask, right_shift, available_bits_in_first_byte: quote_info.available_bits_in_first_byte(), flip: quote_info.flip()};
                        let full_quote = build_be_number_quote(self, first_bits_index)?;
                        let apply_field_to_buffer = quote! {
                            #ty_ident::from_be_bytes({
                                #full_quote
                            })#shift
                        };
                        apply_field_to_buffer
                    }
                    NumberType::Char => {
                        // let info = BuildNumberQuotePackage { amount_of_bits: quote_info.amount_of_bits(), bits_in_last_byte, field_buffer_name: quote_info.field_buffer_name(), rust_size, first_bits_index, starting_inject_byte: quote_info.starting_inject_byte(), first_bit_mask, last_bit_mask, right_shift, available_bits_in_first_byte: quote_info.available_bits_in_first_byte(), flip: quote_info.flip()};
                        let full_quote = build_be_number_quote(self, first_bits_index)?;
                        let apply_field_to_buffer = quote! {
                            u32::from_be_bytes({
                                #full_quote
                            })#shift
                        };
                        apply_field_to_buffer
                    }
                    NumberType::Bool => return Err(syn::Error::new(self.ident().span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
                }
            }
            ResolverType::Array { sub_ty, array_ty, sizings } => {
                return Err(syn::Error::new(self.ident().span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad."));
            }
            ResolverType::Nested { ty_ident, rust_size } => {
                return Err(syn::Error::new(self.ident().span(), "Struct was given Endianness which should be described by the struct implementing Bitfield"));
            }
        };

        Ok(output)
    }
}

fn build_be_number_quote(
    field: &Resolver,
    first_bits_index: usize,
) -> syn::Result<TokenStream> {

    let stuff = ResolverDataBigAdditive::from(field.data.as_ref())
    let amount_of_bits = field.bit_length();
    let bits_in_last_byte = stuff.bits_in_last_byte;
    let field_buffer_name = field.field_buffer_name();
    let size = field.ty.rust_size();
    let starting_inject_byte = field.starting_inject_byte();
    let first_bit_mask = stuff.first_bit_mask;
    let last_bit_mask = stuff.last_bit_mask;
    let right_shift = stuff.right_shift;
    let available_bits_in_first_byte = field.available_bits_in_first_byte();
    let flip = field.data.flip();
    let new_array_quote = if let Some(a) = add_sign_fix_quote(field, amount_of_bits, right_shift)? {
        a
    } else {
        quote! {[0u8;#size]}
    };
    let mut full_quote = if first_bit_mask == u8::MAX {
        quote! {
            let mut #field_buffer_name: [u8;#size] = #new_array_quote;
            #field_buffer_name[#first_bits_index] |= input_byte_buffer[#starting_inject_byte];
        }
    } else {
        quote! {
            let mut #field_buffer_name: [u8;#size] = #new_array_quote;
            #field_buffer_name[#first_bits_index] |= input_byte_buffer[#starting_inject_byte] & #first_bit_mask;
        }
    };
    // fill in the rest of the bits
    let mut current_byte_index_in_buffer: usize = if flip.is_none() {
        starting_inject_byte + 1
    } else {
        starting_inject_byte - 1
    };
    if right_shift > 0 {
        // right shift (this means that the last bits are in the first byte)
        if available_bits_in_first_byte + bits_in_last_byte != amount_of_bits {
            for i in first_bits_index + 1usize..size {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#current_byte_index_in_buffer] ;
                };
                current_byte_index_in_buffer = if flip.is_none() {
                    current_byte_index_in_buffer + 1
                } else {
                    current_byte_index_in_buffer - 1
                };
            }
        }
        full_quote = quote! {
            #full_quote
            #field_buffer_name[0] |= input_byte_buffer[#current_byte_index_in_buffer] & #last_bit_mask;
            #field_buffer_name
        };
    } else {
        // no shift or left shift (this means the last byte contains the last bits)
        if available_bits_in_first_byte + bits_in_last_byte != amount_of_bits {
            for i in first_bits_index + 1..size - 1 {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#current_byte_index_in_buffer];
                };
                current_byte_index_in_buffer = if flip.is_none() {
                    current_byte_index_in_buffer + 1
                } else {
                    current_byte_index_in_buffer - 1
                };
            }
        }
        // this should give us the last index of the field
        let final_index = size - 1;
        if last_bit_mask == u8::MAX {
            full_quote = quote! {
                #full_quote
                #field_buffer_name[#final_index] |= input_byte_buffer[#current_byte_index_in_buffer];
                #field_buffer_name
            };
        } else {
            full_quote = quote! {
                #full_quote
                #field_buffer_name[#final_index] |= input_byte_buffer[#current_byte_index_in_buffer] & #last_bit_mask;
                #field_buffer_name
            };
        }
    }
    Ok(full_quote)
}
