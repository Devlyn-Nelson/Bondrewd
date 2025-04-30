use std::cmp::Ordering;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma, Error};

use crate::{
    build::field::NumberType,
    solved::{
        array_iter::{BlockArrayIter, ElementArrayIter},
        field::{Resolver, ResolverArrayType, ResolverSubType, ResolverType},
    },
};

use super::{
    get_be_starting_index, get_left_and_mask, get_right_and_mask, ResolverDataBigAdditive,
    ResolverDataLittleAdditive, ResolverDataNestedAdditive,
};

impl Resolver {
    /// This function is kind of funny. it is essentially a function that gets called by either
    /// `get_le_quotes`, `get_be_quotes`, `get_ne_quotes` with the end code generation function given
    /// as a parameter `gen_write_fn`. and example of a function that can be used as `gen_write_fn` would
    /// be `get_write_le_multi_byte_quote`;
    pub(crate) fn get_write_quote(
        &self,
        gen_write_fn: fn(&Resolver, &TokenStream) -> syn::Result<(TokenStream, TokenStream)>,
        with_self: bool,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        let field_name = self.name();
        let field_access = match self.ty.as_ref() {
            ResolverType::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            } => match number_ty {
                NumberType::Float => {
                    if with_self {
                        quote! {self.#field_name.to_bits()}
                    } else {
                        quote! {#field_name.to_bits()}
                    }
                }
                NumberType::Unsigned | NumberType::Signed | NumberType::Bool => {
                    if with_self {
                        quote! {self.#field_name}
                    } else {
                        quote! {#field_name}
                    }
                }
                NumberType::Char => {
                    if with_self {
                        quote! {(self.#field_name as u32)}
                    } else {
                        quote! {(#field_name as u32)}
                    }
                }
            },
            ResolverType::Nested {
                ty_ident,
                rust_size,
            } => {
                if with_self {
                    quote! {self.#field_name}
                } else {
                    quote! {#field_name}
                }
            }
            ResolverType::Array {
                sub_ty,
                array_ty,
                sizings,
            } => match array_ty {
                ResolverArrayType::Element => {
                    let mut clear_buffer = quote! {};
                    let mut buffer = quote! {};
                    let mut de_refs: Punctuated<syn::Ident, Comma> = Punctuated::default();
                    let outer_field_name = &self.ident();
                    let Some(sub) =
                        ElementArrayIter::from_values(&self.data, sub_ty, array_ty, sizings)
                    else {
                        let ident = self.data.field_name.ident();
                        return Err(Error::new(
                            ident.span(),
                            format!("Failed to construct valid ElementArrayIter for `{ident}`"),
                        ));
                    };
                    for sub_field in sub {
                        let field_name = &sub_field.name();
                        let (sub_field_quote, clear) =
                            Self::get_write_quote(&sub_field, gen_write_fn, with_self)?;
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
                ResolverArrayType::Block => {
                    let mut buffer = quote! {};
                    let mut clear_buffer = quote! {};
                    let mut de_refs: Punctuated<syn::Ident, Comma> = Punctuated::default();
                    let outer_field_name = &self.ident();
                    let Some(sub) =
                        BlockArrayIter::from_values(&self.data, sub_ty, array_ty, sizings)
                    else {
                        let ident = self.data.field_name.ident();
                        return Err(Error::new(
                            ident.span(),
                            format!("Failed to construct valid ElementArrayIter for `{ident}`"),
                        ));
                    };
                    for sub_field in sub {
                        let field_name = &sub_field.name();
                        let (sub_field_quote, clear) =
                            Self::get_write_quote(&sub_field, gen_write_fn, with_self)?;
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
            },
        };
        gen_write_fn(&self, &field_access)
    }
    pub(crate) fn get_write_le_quote(
        &self,
        field_access_quote: &TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        if self.bit_length() > self.available_bits_in_first_byte() {
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
            self.get_write_le_multi_byte_quote(field_access_quote)
        } else {
            self.get_write_le_single_byte_quote(field_access_quote)
        }
    }
    pub(crate) fn get_write_le_single_byte_quote(
        &self,
        field_access_quote: &TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        let amount_of_bits = self.bit_length();
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (self.data.zeros_on_left + amount_of_bits) {
            return Err(syn::Error::new(
                self.ident().span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (self.data.zeros_on_left + amount_of_bits);
        // combining the left and right masks will give us a mask that keeps the amount og bytes we
        // have in the position we need them to be in for this byte. we use available_bytes for
        // right mask because param is amount of 1's on the side specified (right), and
        // available_bytes is (8 - zeros_on_left) which is equal to ones_on_right.
        let bit_range_start = self.data.bit_range_start();
        let mask = get_right_and_mask(self.data.available_bits_in_first_byte)
            & get_left_and_mask(8 - zeros_on_right);
        // calculate how many left shifts need to occur to the number in order to position the bytes
        // we want to keep in the position we want.
        if 8 - amount_of_bits < bit_range_start % 8 {
            return Err(syn::Error::new(
                self.ident().span(),
                format!(
                    "calculating be left_shift failed {} , {}",
                    amount_of_bits,
                    bit_range_start % 8
                ),
            ));
        }
        let shift_left = (8 - amount_of_bits) - (bit_range_start % 8);
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
        let field_as_u8_quote = match self.get_resolved_ty() {
            ResolverSubType::Primitive { number_ty, resolver_strategy, rust_size } => match number_ty {
                NumberType::Float => return Err(syn::Error::new(self.ident().span(), "Float not supported for single byte insert logic")),
                NumberType::Unsigned |
                NumberType::Signed |
                NumberType::Char |
                NumberType::Bool => quote!{(#field_access_quote as u8)},
            },
            ResolverSubType::Nested { ty_ident, rust_size } => return Err(syn::Error::new(self.ident().span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
        };
        let not_mask = !mask;
        let starting_inject_byte = self.data.starting_inject_byte;
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
    pub(crate) fn get_write_le_multi_byte_quote(
        &self,
        field_access_quote: &TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        let amount_of_bits = self.data.bit_length();
        let (right_shift, first_bit_mask, last_bit_mask): (i8, u8, u8) = {
            let thing: ResolverDataLittleAdditive = self.data.as_ref().into();
            (thing.right_shift, thing.first_bit_mask, thing.last_bit_mask)
        };
        // make a name for the buffer that we will store the number in byte form
        let field_buffer_name = self.data.field_name.name();
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let mut full_quote = match self.get_resolved_ty() {
            ResolverSubType::Primitive { number_ty, resolver_strategy, rust_size } => match number_ty {
                NumberType::Float |
                NumberType::Unsigned |
                NumberType::Signed |
                NumberType::Char => {
                    let field_call = quote!{#field_access_quote.to_le_bytes()};
                    let apply_field_to_buffer = quote! {
                        let mut #field_buffer_name = #field_call;
                    };
                    apply_field_to_buffer
                }
                NumberType::Bool => return Err(syn::Error::new(self.ident().span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            },
            ResolverSubType::Nested { ty_ident, rust_size } => return Err(syn::Error::new(self.ident().span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
        };
        let fields_last_bits_index = amount_of_bits.div_ceil(8) - 1;
        let current_bit_mask = get_right_and_mask(self.data.available_bits_in_first_byte);
        #[allow(clippy::cast_possible_truncation)]
        let mid_shift: u32 = 8 - self.data.available_bits_in_first_byte as u32;
        let next_bit_mask = get_left_and_mask(mid_shift as usize);
        let mut i = 0;
        let mut clear_quote = quote! {};
        while i != fields_last_bits_index {
            let start = self.data.offset_starting_inject_byte(i);
            let not_current_bit_mask = !current_bit_mask;
            if self.data.available_bits_in_first_byte == 0 && right_shift == 0 {
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
                if self.data.available_bits_in_first_byte + (8 * i) < amount_of_bits
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
                let next_index = self.data.next_index(start);
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
        let used_bits = self.data.available_bits_in_first_byte + (8 * i);
        let start = self.data.offset_starting_inject_byte(i);
        if right_shift > 0 {
            let right_shift: u32 = u32::from(right_shift.unsigned_abs());
            // let not_first_bit_mask = !first_bit_mask;
            // let not_last_bit_mask = !last_bit_mask;

            full_quote = quote! {
                #full_quote
                #field_buffer_name[#i] = #field_buffer_name[#i].rotate_right(#right_shift);
            };
            if used_bits < amount_of_bits {
                clear_quote = quote! {
                    #clear_quote
                    output_byte_buffer[#start] &= 0;
                };
                let next_index = self.data.next_index(start);
                full_quote = quote! {
                    #full_quote
                    output_byte_buffer[#start] |= #field_buffer_name[#i] & #first_bit_mask;
                    output_byte_buffer[#next_index] |= #field_buffer_name[#i] & #last_bit_mask;
                };
            } else {
                let mut last_mask = first_bit_mask;
                if amount_of_bits <= used_bits {
                    last_mask &= !get_right_and_mask(used_bits - amount_of_bits);
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
            if amount_of_bits <= used_bits {
                last_mask &= !get_right_and_mask(used_bits - amount_of_bits);
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
    pub(crate) fn get_write_ne_quote(
        &self,
        field_access_quote: &TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        if self.data.bit_length() > self.data.available_bits_in_first_byte {
            // how many times to shift the number right.
            // NOTE if negative shift left.
            // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
            // for a f32) then use the last byte in the fields byte array after shifting for the first
            // used byte in the buffer.
            if 8 < self.data.available_bits_in_first_byte % 8 {
                return Err(syn::Error::new(
                    self.ident().span(),
                    "calculating ne right_shift failed",
                ));
            }
            self.get_write_ne_multi_byte_quote(field_access_quote)
        } else {
            self.get_write_ne_single_byte_quote(field_access_quote)
        }
    }
    pub(crate) fn get_write_ne_single_byte_quote(
        &self,
        field_access_quote: &TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        let amount_of_bits = self.data.bit_length();
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (self.data.zeros_on_left + amount_of_bits) {
            return Err(syn::Error::new(
                self.ident().span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (self.data.zeros_on_left + amount_of_bits);
        // combining the left and right masks will give us a mask that keeps the amount og bytes we
        // have in the position we need them to be in for this byte. we use available_bytes for
        // right mask because param is amount of 1's on the side specified (right), and
        // available_bytes is (8 - zeros_on_left) which is equal to ones_on_right.
        let mask = get_right_and_mask(self.data.available_bits_in_first_byte)
            & get_left_and_mask(8 - zeros_on_right);
        // calculate how many left shifts need to occur to the number in order to position the bytes
        // we want to keep in the position we want.
        if 8 < amount_of_bits || 8 - amount_of_bits < self.data.bit_range_start() % 8 {
            return Err(syn::Error::new(
                self.ident().span(),
                "calculating ne shift_left failed",
            ));
        }
        let shift_left = (8 - amount_of_bits) - (self.data.bit_range_start() % 8);
        let starting_inject_byte = self.data.starting_inject_byte;
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
        let finished_quote = match self.get_resolved_ty() {
            ResolverSubType::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            } => match number_ty {
                NumberType::Float | NumberType::Unsigned | NumberType::Signed => {
                    return Err(syn::Error::new(
                        self.ident().span(),
                        "Number not supported for no-endian insert logic",
                    ))
                }
                NumberType::Char => {
                    return Err(syn::Error::new(
                        self.ident().span(),
                        "Char not supported for no-endian insert logic",
                    ))
                }
                NumberType::Bool => {
                    quote! {output_byte_buffer[#starting_inject_byte] |= ((#field_access_quote as u8) << #shift_left) & #mask;}
                }
            },
            ResolverSubType::Nested {
                ty_ident,
                rust_size,
            } => {
                let used_bits_in_byte = 8 - self.data.available_bits_in_first_byte;
                quote! {output_byte_buffer[#starting_inject_byte] |= (#field_access_quote.into_bytes()[0]) >> #used_bits_in_byte;}
                // let used_bits_in_byte = quote_info.available_bits_in_first_byte() % 8;
                // quote!{output_byte_buffer[#starting_inject_byte] |= (#field_access_quote.into_bytes()[0]) << #used_bits_in_byte;}
            }
        };
        let not_mask = !mask;
        let clear_quote = quote! {
            output_byte_buffer[#starting_inject_byte] &= #not_mask;
        };
        Ok((finished_quote, clear_quote))
    }
    pub(crate) fn get_write_ne_multi_byte_quote(
        &self,
        field_access_quote: &TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        let right_shift: i8 = {
            let thing: ResolverDataNestedAdditive = self.data.as_ref().into();
            thing.right_shift % 8
        };
        // make a name for the buffer that we will store the number in byte form
        let field_buffer_name = self.field_buffer_ident();
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let (field_byte_buffer, size) = match self.get_resolved_ty() {
            ResolverSubType::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            } => {
                return Err(syn::Error::new(
                    self.ident().span(),
                    "Primitive was not given Endianness, please report this.",
                ))
            }
            ResolverSubType::Nested {
                ty_ident,
                rust_size,
            } => {
                let field_call = quote! {#field_access_quote.into_bytes()};
                let apply_field_to_buffer = quote! {
                    let mut #field_buffer_name = #field_call
                };
                (apply_field_to_buffer, rust_size)
            }
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
                let current_bit_mask = get_right_and_mask(self.data.available_bits_in_first_byte);
                let next_bit_mask = get_left_and_mask(8 - self.data.available_bits_in_first_byte);
                let right_shift: u32 = u32::from(right_shift.unsigned_abs());
                for i in 0usize..size {
                    let start = self.data.offset_starting_inject_byte(i);
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
                    let next_index = self.data.next_index(start);
                    if self.data.available_bits_in_first_byte + (8 * i) < self.data.bit_length() {
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
                    self.ident().span(),
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
                let current_bit_mask = get_right_and_mask(self.data.available_bits_in_first_byte);

                for i in 0usize..size {
                    let start = self.data.offset_starting_inject_byte(i);
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
    pub(crate) fn get_write_be_quote(
        &self,
        field_access_quote: &TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        let amount_of_bits = self.data.bit_length();
        if amount_of_bits > self.data.available_bits_in_first_byte {
            // calculate how many of the bits will be inside the least significant byte we are adding to.
            // this will also be the number used for shifting to the right >> because that will line up
            // our bytes for the buffer.
            if amount_of_bits < self.data.available_bits_in_first_byte {
                return Err(syn::Error::new(
                    self.ident().span(),
                    "calculating be bits_in_last_bytes failed",
                ));
            }
            self.get_write_be_multi_byte_quote(field_access_quote)
        } else {
            self.get_write_be_single_byte_quote(field_access_quote)
        }
    }
    pub(crate) fn get_write_be_single_byte_quote(
        &self,
        field_access_quote: &TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        let amount_of_bits = self.data.bit_length();
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (self.data.zeros_on_left + amount_of_bits) {
            return Err(syn::Error::new(
                self.ident().span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (self.data.zeros_on_left + amount_of_bits);
        // combining the left and right masks will give us a mask that keeps the amount og bytes we
        // have in the position we need them to be in for this byte. we use available_bytes for
        // right mask because param is amount of 1's on the side specified (right), and
        // available_bytes is (8 - zeros_on_left) which is equal to ones_on_right.
        let mask = get_right_and_mask(self.data.available_bits_in_first_byte)
            & get_left_and_mask(8 - zeros_on_right);
        // calculate how many left shifts need to occur to the number in order to position the bytes
        // we want to keep in the position we want.
        if 8 - amount_of_bits < self.data.bit_range_start() % 8 {
            return Err(syn::Error::new(
                self.ident().span(),
                format!(
                    "calculating be left_shift failed {} , {}",
                    amount_of_bits,
                    self.data.bit_range_start() % 8
                ),
            ));
        }
        let shift_left = (8 - amount_of_bits) - (self.data.bit_range_start() % 8);
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
        let field_as_u8_quote = match self.get_resolved_ty() {
            ResolverSubType::Primitive { number_ty, resolver_strategy, rust_size } => match number_ty {
                NumberType::Float => return Err(syn::Error::new(self.ident().span(), "Float not supported for single byte insert logic")),
                NumberType::Unsigned |
                NumberType::Signed |
                NumberType::Char |
                NumberType::Bool => quote!{(#field_access_quote as u8)},
            },
            ResolverSubType::Nested { ty_ident, rust_size } => return Err(syn::Error::new(self.ident().span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
        };
        let starting_inject_byte = self.data.starting_inject_byte;
        let not_mask = !mask;
        let clear_quote = quote! {
            output_byte_buffer[#starting_inject_byte] &= #not_mask;
        };
        let apply_field_to_buffer = quote! {
            output_byte_buffer[#starting_inject_byte] |= (#field_as_u8_quote << #shift_left) & #mask;
        };
        Ok((apply_field_to_buffer, clear_quote))
    }
    pub(crate) fn get_write_be_multi_byte_quote(
        &self,
        field_access_quote: &TokenStream,
    ) -> syn::Result<(TokenStream, TokenStream)> {
        let amount_of_bits = self.data.bit_length();
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
                quote! { (#field_access_quote.rotate_left(#left_shift)) },
                // if the size of the field type is the same as the bit size going into the
                // bit_buffer then we use the last byte for applying to the buffers first effected
                // byte.
                if self.ty.rust_size() * 8 == amount_of_bits {
                    self.ty.rust_size() - 1
                } else {
                    match get_be_starting_index(amount_of_bits, right_shift, self.ty.rust_size()) {
                        Ok(good) => good,
                        Err(err) => {
                            return Err(syn::Error::new(
                                self.ident().span(),
                                format!("{err} (into 1)"),
                            ))
                        }
                    }
                },
            )
        } else {
            (
                if right_shift == 0 {
                    // no shift no code, just the
                    quote! { #field_access_quote }
                } else {
                    // shift right code
                    let right_shift_usize: u32 = u32::from(right_shift.unsigned_abs());
                    quote! { (#field_access_quote.rotate_right(#right_shift_usize)) }
                },
                match get_be_starting_index(amount_of_bits, right_shift, self.ty.rust_size()) {
                    Ok(good) => good,
                    Err(err) => {
                        return Err(syn::Error::new(
                            self.ident().span(),
                            format!("{err} (into 2)"),
                        ))
                    }
                },
            )
        };
        // make a name for the buffer that we will store the number in byte form
        let field_buffer_name = self.field_buffer_ident();
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let field_byte_buffer = match self.get_resolved_ty() {
            ResolverSubType::Primitive { number_ty, resolver_strategy, rust_size } => match number_ty {
                NumberType::Float |
                NumberType::Unsigned |
                NumberType::Signed |
                NumberType::Char => {
                    let field_call = quote!{#shift.to_be_bytes()};
                    let apply_field_to_buffer = quote! {
                        let #field_buffer_name = #field_call
                    };
                    apply_field_to_buffer
                }
                NumberType::Bool => return Err(syn::Error::new(self.ident().span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            },
            ResolverSubType::Nested { ty_ident, rust_size } => return Err(syn::Error::new(self.ident().span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
        };
        let starting_inject_byte = self.data.starting_inject_byte;
        let not_first_bit_mask = !first_bit_mask;
        let mut clear_quote = quote! {
            output_byte_buffer[#starting_inject_byte] &= #not_first_bit_mask;
        };
        let mut full_quote = if first_bit_mask == u8::MAX {
            quote! {
                #field_byte_buffer;
                output_byte_buffer[#starting_inject_byte] |= #field_buffer_name[#first_bits_index];
            }
        } else {
            quote! {
                #field_byte_buffer;
                output_byte_buffer[#starting_inject_byte] |= #field_buffer_name[#first_bits_index] & #first_bit_mask;
            }
        };
        // fill in the rest of the bits
        let mut current_byte_index_in_buffer: usize = self.data.offset_starting_inject_byte(1);
        let not_last_bit_mask = !last_bit_mask;
        if right_shift > 0 {
            // right shift (this means that the last bits are in the first byte)
            if self.data.available_bits_in_first_byte + bits_in_last_byte != amount_of_bits {
                for i in first_bits_index + 1usize..self.ty.rust_size() {
                    clear_quote = quote! {
                        #clear_quote
                        output_byte_buffer[#current_byte_index_in_buffer] &= 0u8;
                    };
                    full_quote = quote! {
                        #full_quote
                        output_byte_buffer[#current_byte_index_in_buffer] |= #field_buffer_name[#i];
                    };
                    current_byte_index_in_buffer =
                        self.data.next_index(current_byte_index_in_buffer);
                }
            }
            clear_quote = quote! {
                #clear_quote
                output_byte_buffer[#current_byte_index_in_buffer] &= #not_last_bit_mask;
            };
            full_quote = quote! {
                #full_quote
                output_byte_buffer[#current_byte_index_in_buffer] |= #field_buffer_name[0] & #last_bit_mask;
            };
        } else {
            // no shift
            if self.data.available_bits_in_first_byte + bits_in_last_byte != amount_of_bits {
                for i in first_bits_index + 1..self.ty.rust_size() - 1 {
                    clear_quote = quote! {
                        #clear_quote
                        output_byte_buffer[#current_byte_index_in_buffer] &= 0u8;
                    };
                    full_quote = quote! {
                        #full_quote
                        output_byte_buffer[#current_byte_index_in_buffer] |= #field_buffer_name[#i];
                    };
                    current_byte_index_in_buffer =
                        self.data.next_index(current_byte_index_in_buffer);
                }
            }
            // this should give us the last index of the field
            let final_index = self.ty.rust_size() - 1;
            clear_quote = quote! {
                #clear_quote
                output_byte_buffer[#current_byte_index_in_buffer] &= #not_last_bit_mask;
            };
            if last_bit_mask == u8::MAX {
                full_quote = quote! {
                    #full_quote
                    output_byte_buffer[#current_byte_index_in_buffer] |= #field_buffer_name[#final_index];
                };
            } else {
                full_quote = quote! {
                    #full_quote
                    output_byte_buffer[#current_byte_index_in_buffer] |= #field_buffer_name[#final_index] & #last_bit_mask;
                };
            }
        }

        Ok((full_quote, clear_quote))
    }
}
