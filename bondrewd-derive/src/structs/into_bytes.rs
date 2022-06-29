use crate::structs::common::{
    get_be_starting_index, get_left_and_mask, get_right_and_mask, BitMath, Endianness,
    FieldDataType, FieldInfo, StructInfo,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub struct IntoBytesOptions {
    pub into_bytes_fn: TokenStream,
    pub set_field_fns: TokenStream,
    pub set_slice_field_fns: Option<TokenStream>,
    pub set_slice_field_unchecked_fns: Option<TokenStream>,
}

pub fn create_into_bytes_field_quotes(
    info: &StructInfo,
    set_slice: bool,
) -> Result<IntoBytesOptions, syn::Error> {
    // all of the fields setting will be appended to this
    let mut into_bytes_quote = quote! {};
    // all quote with all of the set slice functions appended to it.
    let mut set_slice_fns_option = if set_slice {
        let checked_ident = format_ident!("{}CheckedMut", &info.name);
        let check_size = info.total_bytes();
        let comment = format!("Returns a structure which allows you to read/write any field for a [{}] from/to provided mutable slice.", &info.name);
        Some((
            quote! {
                #[doc = #comment]
                pub fn check_slice_mut(buffer: &mut [u8]) -> Result<#checked_ident, BitfieldSliceError> {
                    let buf_len = buffer.len();
                    if buf_len >= #check_size {
                        Ok(#checked_ident {
                            buffer
                        })
                    }else{
                        Err(BitfieldSliceError(buf_len, #check_size))
                    }
                }
            },
            quote! {},
        ))
    } else {
        None
    };
    // all quote with all of the set functions appended to it.
    let mut set_fns_quote = quote! {};
    for field in info.fields.iter() {
        if field.attrs.reserve.is_fake_field() {
            continue;
        }
        let (field_setter, clear_quote) = get_field_quote(
            &field,
            if info.flip {
                Some(info.total_bytes() - 1)
            } else {
                None
            },
            false,
        )?;
        if field.attrs.reserve.write_field() {
            let field_name = &field.ident;
            into_bytes_quote = quote! {
                #into_bytes_quote
                let #field_name = self.#field_name;
                #field_setter
            };
        }
        let set_quote = make_set_fn(&field_setter, &field, &info, &clear_quote)?;
        set_fns_quote = quote! {
            #set_fns_quote
            #set_quote
        };

        if let Some((ref mut set_slice_fns_quote, ref mut unchecked)) = set_slice_fns_option {
            let set_slice_quote = make_set_slice_fn(&field_setter, &field, &info, &clear_quote)?;
            let set_slice_unchecked_quote =
                make_set_slice_unchecked_fn(&field_setter, &field, &clear_quote)?;
            let mut set_slice_fns_quote_temp = quote! {
                #set_slice_fns_quote
                #set_slice_quote
            };
            let mut unchecked_temp = quote! {
                #unchecked
                #set_slice_unchecked_quote
            };
            std::mem::swap(set_slice_fns_quote, &mut set_slice_fns_quote_temp);
            std::mem::swap(unchecked, &mut unchecked_temp);
        }
    }
    let struct_size = &info.total_bytes();
    // construct from bytes function. use input_byte_buffer as input name because,
    // that is what the field quotes expect to extract from.
    // wrap our list of field names with commas with Self{} so we it instantiate our struct,
    // because all of the from_bytes field quote store there data in a temporary variable with the same
    // name as its destination field the list of field names will be just fine.
    let into_bytes_fn = quote! {
        fn into_bytes(self) -> [u8;#struct_size] {
            let mut output_byte_buffer: [u8;#struct_size] = [0u8;#struct_size];
            #into_bytes_quote
            output_byte_buffer
        }
    };
    if let Some((set_slice_field_fns, set_slice_field_unchecked_fns)) = set_slice_fns_option {
        Ok(IntoBytesOptions {
            into_bytes_fn,
            set_field_fns: set_fns_quote,
            set_slice_field_fns: Some(set_slice_field_fns),
            set_slice_field_unchecked_fns: Some(set_slice_field_unchecked_fns),
        })
    } else {
        Ok(IntoBytesOptions {
            into_bytes_fn,
            set_field_fns: set_fns_quote,
            set_slice_field_fns: None,
            set_slice_field_unchecked_fns: None,
        })
    }
}

fn make_set_slice_fn(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    clear_quote: &TokenStream,
) -> syn::Result<TokenStream> {
    let field_name = format_ident!("{}", field.ident.as_ref().clone());
    let bit_range = &field.attrs.bit_range;
    let fn_field_name = format_ident!("write_slice_{}", field_name);
    let type_ident = field.ty.type_quote();
    let min_length = if info.flip {
        ((info.total_bits() - field.attrs.bit_range.start) as f64 / 8.0f64).ceil() as usize
    } else {
        (field.attrs.bit_range.end as f64 / 8.0f64).ceil() as usize
    };
    let comment = format!("Returns `Ok(())` if the bits {} through {} for the `{field_name}` field in `input_byte_buffer` could be written to, otherwise a [BitfieldSliceError](bondrewd::BitfieldSliceError) will be returned", bit_range.start, bit_range.end - 1);
    Ok(quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(output_byte_buffer: &mut [u8], #field_name: #type_ident) -> Result<(), BitfieldSliceError> {
            let slice_length = output_byte_buffer.len();
            if slice_length < #min_length {
                Err(BitfieldSliceError(slice_length, #min_length))
            } else {
                #clear_quote
                #field_quote
                Ok(())
            }
        }
    })
}

fn make_set_slice_unchecked_fn(
    field_quote: &TokenStream,
    field: &FieldInfo,
    clear_quote: &TokenStream,
) -> syn::Result<TokenStream> {
    let field_name = format_ident!("{}", field.ident.as_ref().clone());
    let bit_range = &field.attrs.bit_range;
    let fn_field_name = format_ident!("write_{}", field_name);
    let type_ident = field.ty.type_quote();
    let comment = format!(
        "Writes to the bits {} through {} for the `{field_name}` field in the provided pre-checked mutable slice.", bit_range.start, bit_range.end - 1
    );
    Ok(quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(&mut self, #field_name: #type_ident) {
            let output_byte_buffer: &mut [u8] = self.buffer;
            #clear_quote
            #field_quote
        }
    })
}

fn make_set_fn(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    clear_quote: &TokenStream,
) -> syn::Result<TokenStream> {
    let field_name = format_ident!("{}", field.ident.as_ref().clone());
    let bit_range = &field.attrs.bit_range;
    let fn_field_name = format_ident!("write_{}", field_name);
    let type_ident = field.ty.type_quote();
    let struct_size = info.total_bytes();
    let struct_name = &info.name;
    let comment = format!("Writes `{field_name}` to the bits {} through {} for the `{field_name}` field of a [{struct_name}] in `output_byte_buffer`.", bit_range.start, bit_range.end - 1);
    Ok(quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(output_byte_buffer: &mut [u8;#struct_size], mut #field_name: #type_ident) {
            #clear_quote
            #field_quote
        }
    })
}

/// the flip value must be the total amount of bytes the result of into_bytes should have MINUS ONE,
/// the number is used to invert indices
fn get_field_quote(
    field: &FieldInfo,
    flip: Option<usize>,
    with_self: bool,
) -> syn::Result<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    let field_name = field.name.clone();
    let quote_field_name = match field.ty {
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
        FieldDataType::ElementArray(_, _, _) => {
            let mut clear_buffer = quote! {};
            let mut buffer = quote! {};
            let mut de_refs: syn::punctuated::Punctuated<syn::Ident, syn::token::Comma> =
                Default::default();
            let outer_field_name = &field.ident;
            let sub = field.get_element_iter()?;
            for sub_field in sub {
                let field_name = &sub_field.name;
                let (sub_field_quote, clear) = get_field_quote(&sub_field, flip, with_self)?;
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
            let mut de_refs: syn::punctuated::Punctuated<syn::Ident, syn::token::Comma> =
                Default::default();
            let outer_field_name = &field.ident;
            let sub = field.get_block_iter()?;
            for sub_field in sub {
                let field_name = &sub_field.name;
                let (sub_field_quote, clear) = get_field_quote(&sub_field, flip, with_self)?;
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
    match field.attrs.endianness.as_ref() {
        Endianness::Big => apply_be_math_to_field_access_quote(field, quote_field_name, flip),
        Endianness::Little => apply_le_math_to_field_access_quote(field, quote_field_name, flip),
        Endianness::None => apply_ne_math_to_field_access_quote(field, quote_field_name, flip),
    }
}
// first token stream is actual setter, but second one is overwrite current bits to 0.
fn apply_le_math_to_field_access_quote(
    field: &FieldInfo,
    field_access_quote: proc_macro2::TokenStream,
    flip: Option<usize>,
) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), syn::Error> {
    let (amount_of_bits, zeros_on_left, available_bits_in_first_byte, mut starting_inject_byte) =
        BitMath::from_field(field)?.into_tuple();
    let operator = if let Some(flip) = flip {
        starting_inject_byte = flip - starting_inject_byte;
        quote! {-}
    } else {
        quote! {+}
    };
    // check if we need to span multiple bytes
    if amount_of_bits > available_bits_in_first_byte {
        // calculate how many of the bits will be inside the least significant byte we are adding to.
        // this will also be the number used for shifting to the right >> because that will line up
        // our bytes for the buffer.
        if amount_of_bits < available_bits_in_first_byte {
            return Err(syn::Error::new(
                field.ident.span(),
                "calculating be bits_in_last_bytes failed",
            ));
        }
        let bits_in_last_byte = (amount_of_bits - available_bits_in_first_byte) % 8;
        // how many times to shift the number right.
        // NOTE if negative shift left.
        // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        let mut bits_needed_in_msb = amount_of_bits % 8;
        if bits_needed_in_msb == 0 {
            bits_needed_in_msb = 8;
        }
        let mut right_shift: i8 =
            (bits_needed_in_msb as i8) - ((available_bits_in_first_byte % 8) as i8);
        if right_shift == 8 {
            right_shift = 0;
        }
        // because we are applying bits in place we need masks in insure we don't effect other fields
        // data. we need one for the first byte and the last byte.
        let first_bit_mask = get_right_and_mask(available_bits_in_first_byte);
        let last_bit_mask = if bits_in_last_byte == 0 {
            get_left_and_mask(8)
        } else {
            get_left_and_mask(bits_in_last_byte)
        };

        // // create a quote that holds the bit shifting operator and shift value and the field name.
        // // first_bits_index is the index to use in the fields byte array after shift for the
        // // starting byte in the byte buffer. when left shifts happen on full sized numbers the last
        // // index of the fields byte array will be used.
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
        // make a name for the buffer that we will store the number in byte form
        let field_buffer_name = format_ident!("{}_bytes", field.ident.as_ref());
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let field_byte_buffer = match field.ty {
            FieldDataType::Number(_, _, _) |
            FieldDataType::Float(_, _) |
            FieldDataType::Char(_, _) => {
                let field_call = quote!{#field_access_quote.to_le_bytes()};
                let apply_field_to_buffer = quote! {
                    let mut #field_buffer_name = #field_call
                };
                apply_field_to_buffer
            }
            FieldDataType::Boolean => return Err(syn::Error::new(field.ident.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Enum(_, _, _) => {
                let field_call = quote!{#field_access_quote.to_le_bytes()};
                let apply_field_to_buffer = quote! {
                    let mut #field_buffer_name = #field_call
                };
                apply_field_to_buffer
            }
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(field.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad."))
        };
        let mut full_quote = quote! {
            #field_byte_buffer;
        };
        let fields_last_bits_index = (amount_of_bits as f64 / 8.0f64).ceil() as usize - 1;
        let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);
        let mid_shift: u32 = 8 - available_bits_in_first_byte as u32;
        let next_bit_mask = get_left_and_mask(mid_shift as usize);
        let mut i = 0;
        let mut clear_quote = quote! {};
        while i != fields_last_bits_index {
            let start = if let None = flip {
                starting_inject_byte + i
            } else {
                starting_inject_byte - i
            };
            let not_current_bit_mask = !current_bit_mask;
            let not_next_bit_mask = !next_bit_mask;
            if available_bits_in_first_byte == 0 && right_shift == 0 {
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
                if available_bits_in_first_byte + (8 * i) < amount_of_bits {
                    full_quote = quote! {
                        #full_quote
                        output_byte_buffer[#start] |= #field_buffer_name[#i] & #current_bit_mask;
                    };
                    clear_quote = quote! {
                        #clear_quote
                        output_byte_buffer[#start #operator 1] &= #not_next_bit_mask;
                    };
                }
                full_quote = quote! {
                    #full_quote
                    output_byte_buffer[#start #operator 1] |= #field_buffer_name[#i] & #next_bit_mask;
                };
            }
            i += 1;
        }
        // bits used after applying the first_bit_mask one more time.
        let used_bits = available_bits_in_first_byte + (8 * i);
        if right_shift > 0 {
            let start = if let None = flip {
                starting_inject_byte + i
            } else {
                starting_inject_byte - i
            };
            let right_shift: u32 = right_shift.clone() as u32;
            let not_first_bit_mask = !first_bit_mask;
            let not_last_bit_mask = !last_bit_mask;

            full_quote = quote! {
                #full_quote
                #field_buffer_name[#i] = #field_buffer_name[#i].rotate_right(#right_shift);
            };
            if used_bits < amount_of_bits {
                clear_quote = quote! {
                    #clear_quote
                    output_byte_buffer[#start] &= #not_first_bit_mask;
                    output_byte_buffer[#start #operator 1] &= #not_last_bit_mask;
                };
                full_quote = quote! {
                    #full_quote
                    output_byte_buffer[#start] |= #field_buffer_name[#i] & #first_bit_mask;
                    output_byte_buffer[#start #operator 1] |= #field_buffer_name[#i] & #last_bit_mask;
                };
            } else {
                let mut last_mask = first_bit_mask.clone();
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
            let start = if let None = flip {
                starting_inject_byte + i
            } else {
                starting_inject_byte - i
            };
            // this should give us the last index of the field
            let left_shift: u32 = right_shift.clone().abs() as u32;
            let mut last_mask = first_bit_mask.clone();
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
                output_byte_buffer[#start] |= (#field_buffer_name[#i].rotate_left(#left_shift)) & #last_mask;// fix this mask
            };
        }

        Ok((full_quote, clear_quote))
    } else {
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (zeros_on_left + amount_of_bits) {
            return Err(syn::Error::new(
                field.ident.span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (zeros_on_left + amount_of_bits);
        // combining the left and right masks will give us a mask that keeps the amount og bytes we
        // have in the position we need them to be in for this byte. we use available_bytes for
        // right mask because param is amount of 1's on the side specified (right), and
        // available_bytes is (8 - zeros_on_left) which is equal to ones_on_right.
        let mask = get_right_and_mask(available_bits_in_first_byte)
            & get_left_and_mask(8 - zeros_on_right);
        // calculate how many left shifts need to occur to the number in order to position the bytes
        // we want to keep in the position we want.
        if 8 - amount_of_bits < field.attrs.bit_range.start % 8 {
            return Err(syn::Error::new(
                field.ident.span(),
                format!(
                    "calculating be left_shift failed {} , {}",
                    amount_of_bits,
                    field.attrs.bit_range.start % 8
                ),
            ));
        }
        let shift_left = (8 - amount_of_bits) - (field.attrs.bit_range.start % 8);
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
        let field_as_u8_quote = match field.ty {
            FieldDataType::Number(_, _, _) => {
                quote!{(#field_access_quote as u8)}
            }
            FieldDataType::Boolean => {
                quote!{(#field_access_quote as u8)}
            }
            FieldDataType::Char(_, _) => return Err(syn::Error::new(field.ident.span(), "Char not supported for single byte insert logic")),
            FieldDataType::Enum(_, _, _) => field_access_quote,
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(field.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Float(_, _) => return Err(syn::Error::new(field.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad.")),
        };
        let not_mask = !mask;
        let clear_quote = quote! {
            output_byte_buffer[#starting_inject_byte] &= #not_mask;
        };
        let apply_field_to_buffer = quote! {
            output_byte_buffer[#starting_inject_byte] |= (#field_as_u8_quote << #shift_left) & #mask;
        };
        Ok((apply_field_to_buffer, clear_quote))
    }
}
fn apply_ne_math_to_field_access_quote(
    field: &FieldInfo,
    field_access_quote: proc_macro2::TokenStream,
    flip: Option<usize>,
) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), syn::Error> {
    let (amount_of_bits, zeros_on_left, available_bits_in_first_byte, mut starting_inject_byte) =
        BitMath::from_field(field)?.into_tuple();
    let operator = if let Some(flip) = flip {
        starting_inject_byte = flip - starting_inject_byte;
        quote! {-}
    } else {
        quote! {+}
    };
    // check if we need to span multiple bytes
    if amount_of_bits > available_bits_in_first_byte {
        // how many times to shift the number right.
        // NOTE if negative shift left.
        // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        if 8 < available_bits_in_first_byte % 8 {
            return Err(syn::Error::new(
                field.ident.span(),
                "calculating ne right_shift failed",
            ));
        }
        let right_shift: i8 = (8 as i8) - ((available_bits_in_first_byte % 8) as i8);
        // make a name for the buffer that we will store the number in byte form
        let field_buffer_name = format_ident!("{}_bytes", field.ident.as_ref());
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let (field_byte_buffer, size) = match field.ty {
            FieldDataType::Number(_, _,_ ) |
            FieldDataType::Float(_, _) |
            FieldDataType::Char(_, _) => return Err(syn::Error::new(field.ident.span(), "Char was not given Endianness, please report this.")),
            FieldDataType::Boolean => return Err(syn::Error::new(field.ident.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Enum(_, _, _) => return Err(syn::Error::new(field.ident.span(), "Enum was not given Endianness, please report this.")),
            FieldDataType::Struct(ref size, _) => {
                let field_call = quote!{#field_access_quote.into_bytes()};
                let apply_field_to_buffer = quote! {
                    let mut #field_buffer_name = #field_call
                };
                (apply_field_to_buffer, size.clone())
            }
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };
        let mut clear_quote = quote! {};
        let mut full_quote = quote! {
            #field_byte_buffer;
        };
        // fill in the rest of the bits
        if right_shift > 0 {
            // right shift (this means that the last bits are in the first byte)
            // because we are applying bits in place we need masks in insure we don't effect other fields
            // data. we need one for the first byte and the last byte.
            let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);
            let next_bit_mask = get_left_and_mask(8 - available_bits_in_first_byte);
            let right_shift: u32 = right_shift as u32;
            for i in 0usize..size {
                let start = if let None = flip {
                    starting_inject_byte + i
                } else {
                    starting_inject_byte - i
                };
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

                if available_bits_in_first_byte + (8 * i) < amount_of_bits {
                    clear_quote = quote! {
                        #clear_quote
                        output_byte_buffer[#start #operator 1] &= #not_next_bit_mask;//test
                    };
                    full_quote = quote! {
                        #full_quote
                        output_byte_buffer[#start #operator 1] |= #field_buffer_name[#i] & #next_bit_mask;
                    };
                }
            }
        } else if right_shift < 0 {
            return Err(syn::Error::new(
                field.ident.span(),
                "left shifting struct was removed to see if it would ever happened",
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
        } else {
            // no shift can be more faster.
            let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);

            for i in 0usize..size {
                let start = if let None = flip {
                    starting_inject_byte + i
                } else {
                    starting_inject_byte - i
                };
                let not_current_bit_mask = !current_bit_mask;
                clear_quote = quote! {
                    #clear_quote
                    output_byte_buffer[#start] &= #not_current_bit_mask;
                };
                if i == 0 {
                    full_quote = quote! {
                        #full_quote
                        output_byte_buffer[#start] |= #field_buffer_name[#i] & #current_bit_mask;
                    };
                } else {
                    full_quote = quote! {
                        #full_quote
                        output_byte_buffer[#start] |= #field_buffer_name[#i];
                    };
                }
            }
        }
        Ok((full_quote, clear_quote))
    } else {
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (zeros_on_left + amount_of_bits) {
            return Err(syn::Error::new(
                field.ident.span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (zeros_on_left + amount_of_bits);
        // combining the left and right masks will give us a mask that keeps the amount og bytes we
        // have in the position we need them to be in for this byte. we use available_bytes for
        // right mask because param is amount of 1's on the side specified (right), and
        // available_bytes is (8 - zeros_on_left) which is equal to ones_on_right.
        let mask = get_right_and_mask(available_bits_in_first_byte)
            & get_left_and_mask(8 - zeros_on_right);
        // calculate how many left shifts need to occur to the number in order to position the bytes
        // we want to keep in the position we want.
        if 8 < amount_of_bits || 8 - amount_of_bits < field.attrs.bit_range.start % 8 {
            return Err(syn::Error::new(
                field.ident.span(),
                "calculating ne shift_left failed",
            ));
        }
        let shift_left = (8 - amount_of_bits) - (field.attrs.bit_range.start % 8);

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
        let finished_quote = match field.ty {
            FieldDataType::Number(_, _, _) => return Err(syn::Error::new(field.ident.span(), "Number was not given Endianness, please report this")),
            FieldDataType::Boolean => {
                quote!{output_byte_buffer[#starting_inject_byte] |= ((#field_access_quote as u8) << #shift_left) & #mask;}
            }
            FieldDataType::Char(_, _) => return Err(syn::Error::new(field.ident.span(), "Char not supported for single byte insert logic")),
            FieldDataType::Enum(_, _, _) => return Err(syn::Error::new(field.ident.span(), "Enum was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Struct(_, _) => {
                let used_bits_in_byte = 8 - available_bits_in_first_byte;
                quote!{output_byte_buffer[#starting_inject_byte] |= (#field_access_quote.into_bytes()[0]) >> #used_bits_in_byte;}
            }
            FieldDataType::Float(_, _) => return Err(syn::Error::new(field.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };
        Ok((finished_quote, clear_quote))
    }
}
///
/// # Arguments
/// * `field' - reference to the FieldInfo.
/// * `field_access_quote` - a quote containing access to to byte array of the field.
///                             ex. quote!{(self.char_field as u32)}
fn apply_be_math_to_field_access_quote(
    field: &FieldInfo,
    field_access_quote: proc_macro2::TokenStream,
    flip: Option<usize>,
) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), syn::Error> {
    let (amount_of_bits, zeros_on_left, available_bits_in_first_byte, mut starting_inject_byte) =
        BitMath::from_field(field)?.into_tuple();
    if let Some(flip) = flip {
        starting_inject_byte = flip - starting_inject_byte;
    }
    // check if we need to span multiple bytes
    if amount_of_bits > available_bits_in_first_byte {
        // calculate how many of the bits will be inside the least significant byte we are adding to.
        // this will also be the number used for shifting to the right >> because that will line up
        // our bytes for the buffer.
        if amount_of_bits < available_bits_in_first_byte {
            return Err(syn::Error::new(
                field.ident.span(),
                "calculating be bits_in_last_bytes failed",
            ));
        }
        let bits_in_last_byte = (amount_of_bits - available_bits_in_first_byte) % 8;
        // how many times to shift the number right.
        // NOTE if negative shift left.
        // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        let mut right_shift: i8 =
            ((amount_of_bits % 8) as i8) - ((available_bits_in_first_byte % 8) as i8);
        if right_shift < 0 {
            right_shift += 8
        }
        // because we are applying bits in place we need masks in insure we don't effect other fields
        // data. we need one for the first byte and the last byte.
        let first_bit_mask = get_right_and_mask(available_bits_in_first_byte);
        let last_bit_mask = if bits_in_last_byte == 0 {
            get_left_and_mask(8)
        } else {
            get_left_and_mask(bits_in_last_byte)
        };

        // create a quote that holds the bit shifting operator and shift value and the field name.
        // first_bits_index is the index to use in the fields byte array after shift for the
        // starting byte in the byte buffer. when left shifts happen on full sized numbers the last
        // index of the fields byte array will be used.
        let (shift, first_bits_index) = if right_shift < 0 {
            // convert to left shift using absolute value
            let left_shift: u32 = right_shift.clone().abs() as u32;
            // shift left code
            (
                quote! { (#field_access_quote.rotate_left(#left_shift)) },
                // if the size of the field type is the same as the bit size going into the
                // bit_buffer then we use the last byte for applying to the buffers first effected
                // byte.
                if field.ty.size() * 8 == amount_of_bits {
                    field.ty.size() - 1
                } else {
                    match get_be_starting_index(
                        amount_of_bits,
                        right_shift,
                        field.struct_byte_size(),
                    ) {
                        Ok(good) => good,
                        Err(err) => return Err(syn::Error::new(field.ident.span(), err)),
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
                    let right_shift_usize: u32 = right_shift.clone() as u32;
                    quote! { (#field_access_quote.rotate_right(#right_shift_usize)) }
                },
                match get_be_starting_index(amount_of_bits, right_shift, field.struct_byte_size()) {
                    Ok(good) => good,
                    Err(err) => return Err(syn::Error::new(field.ident.span(), err)),
                },
            )
        };
        // make a name for the buffer that we will store the number in byte form
        let field_buffer_name = format_ident!("{}_bytes", field.ident.as_ref());
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let field_byte_buffer = match field.ty {
            FieldDataType::Number(_, _, _) |
            FieldDataType::Float(_, _) |
            FieldDataType::Char(_, _) => {
                let field_call = quote!{#shift.to_be_bytes()};
                let apply_field_to_buffer = quote! {
                    let #field_buffer_name = #field_call
                };
                apply_field_to_buffer
            }
            FieldDataType::Boolean => return Err(syn::Error::new(field.ident.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Enum(_, _, _) => {
                let field_call = quote!{#shift.to_be_bytes()};
                let apply_field_to_buffer = quote! {
                    let #field_buffer_name = #field_call
                };
                apply_field_to_buffer
            }
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(field.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad."))
        };
        let not_first_bit_mask = !first_bit_mask;
        let mut clear_quote = quote! {
            output_byte_buffer[#starting_inject_byte] &= #not_first_bit_mask;
        };
        let mut full_quote = quote! {
            #field_byte_buffer;
            output_byte_buffer[#starting_inject_byte] |= #field_buffer_name[#first_bits_index] & #first_bit_mask;
        };
        // fill in the rest of the bits
        let mut current_byte_index_in_buffer: usize = if let None = flip {
            starting_inject_byte + 1
        } else {
            starting_inject_byte - 1
        };
        let not_last_bit_mask = !last_bit_mask;
        if right_shift > 0 {
            // right shift (this means that the last bits are in the first byte)
            if available_bits_in_first_byte + bits_in_last_byte != amount_of_bits {
                for i in first_bits_index + 1usize..field.ty.size() {
                    clear_quote = quote! {
                        #clear_quote
                        output_byte_buffer[#current_byte_index_in_buffer] = 0u8;
                    };
                    full_quote = quote! {
                        #full_quote
                        output_byte_buffer[#current_byte_index_in_buffer] |= #field_buffer_name[#i];
                    };
                    current_byte_index_in_buffer = if let None = flip {
                        current_byte_index_in_buffer + 1
                    } else {
                        current_byte_index_in_buffer - 1
                    };
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
            if available_bits_in_first_byte + bits_in_last_byte != amount_of_bits {
                for i in first_bits_index + 1..field.ty.size() - 1 {
                    full_quote = quote! {
                        #full_quote
                        output_byte_buffer[#current_byte_index_in_buffer] |= #field_buffer_name[#i];
                    };
                    current_byte_index_in_buffer = if let None = flip {
                        current_byte_index_in_buffer + 1
                    } else {
                        current_byte_index_in_buffer - 1
                    };
                }
            }
            // this should give us the last index of the field
            let final_index = field.ty.size() - 1;
            clear_quote = quote! {
                #clear_quote
                output_byte_buffer[#current_byte_index_in_buffer] &= #not_last_bit_mask;
            };
            full_quote = quote! {
                #full_quote
                output_byte_buffer[#current_byte_index_in_buffer] |= #field_buffer_name[#final_index] & #last_bit_mask;
            };
        }

        Ok((full_quote, clear_quote))
    } else {
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (zeros_on_left + amount_of_bits) {
            return Err(syn::Error::new(
                field.ident.span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (zeros_on_left + amount_of_bits);
        // combining the left and right masks will give us a mask that keeps the amount og bytes we
        // have in the position we need them to be in for this byte. we use available_bytes for
        // right mask because param is amount of 1's on the side specified (right), and
        // available_bytes is (8 - zeros_on_left) which is equal to ones_on_right.
        let mask = get_right_and_mask(available_bits_in_first_byte)
            & get_left_and_mask(8 - zeros_on_right);
        // calculate how many left shifts need to occur to the number in order to position the bytes
        // we want to keep in the position we want.
        if 8 - amount_of_bits < field.attrs.bit_range.start % 8 {
            return Err(syn::Error::new(
                field.ident.span(),
                format!(
                    "calculating be left_shift failed {} , {}",
                    amount_of_bits,
                    field.attrs.bit_range.start % 8
                ),
            ));
        }
        let shift_left = (8 - amount_of_bits) - (field.attrs.bit_range.start % 8);
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
        let field_as_u8_quote = match field.ty {
            FieldDataType::Number(_, _, _) => {
                quote!{(#field_access_quote as u8)}
            }
            FieldDataType::Boolean => {
                quote!{(#field_access_quote as u8)}
            }
            FieldDataType::Char(_, _) => return Err(syn::Error::new(field.ident.span(), "Char not supported for single byte insert logic")),
            FieldDataType::Enum(_, _, _) => field_access_quote,
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(field.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Float(_, _) => return Err(syn::Error::new(field.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad.")),
        };
        let not_mask = !mask;
        let clear_quote = quote! {
            output_byte_buffer[#starting_inject_byte] &= #not_mask;
        };
        let apply_field_to_buffer = quote! {
            output_byte_buffer[#starting_inject_byte] |= (#field_as_u8_quote << #shift_left) & #mask;
        };
        Ok((apply_field_to_buffer, clear_quote))
    }
}
