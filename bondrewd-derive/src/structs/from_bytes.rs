use crate::structs::common::{
    get_be_starting_index, get_left_and_mask, get_right_and_mask, BitMath, Endianness,
    FieldDataType, FieldInfo, StructInfo,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::common::NumberSignage;

pub struct FromBytesOptions {
    pub from_bytes_fn: TokenStream,
    pub peek_field_fns: TokenStream,
    pub peek_slice_field_fns: Option<TokenStream>,
}

pub fn create_from_bytes_field_quotes(
    info: &StructInfo,
    peek_slice: bool,
) -> Result<FromBytesOptions, syn::Error> {
    // make a quote which is a list of the fields separated by a comma then a newline
    let mut from_bytes_struct_quote = quote! {};
    // all of the fields extraction will be appended to this
    let mut from_bytes_quote = quote! {};
    // all quote with all of the peek slice functions appended to it.
    let mut peek_slice_fns_quote = quote! {};
    // all quote with all of the peek functions appended to it.
    let mut peek_fns_quote = quote! {};
    for field in info.fields.iter() {
        let field_name = &field.ident;
        let peek_name = format_ident!("peek_{}", field_name.as_ref());
        let field_extractor = get_field_quote(
            &field,
            if info.flip {
                Some(info.total_bytes() - 1)
            } else {
                None
            },
        )?;
        let peek_call = if !field.attrs.reserve {
            quote! {Self::#peek_name(&input_byte_buffer)}
        } else {
            // TODO make this detect the type to use.
            let type_quote = field.ty.type_quote();
            quote! {0 as #type_quote}
        };
        from_bytes_quote = quote! {
            #from_bytes_quote
            let #field_name = #peek_call;
        };
        from_bytes_struct_quote = quote! {
            #from_bytes_struct_quote
            #field_name,
        };

        let peek_quote = make_peek_fn(&field_extractor, &field, &info)?;
        peek_fns_quote = quote! {
            #peek_fns_quote
            #peek_quote
        };

        if peek_slice {
            let peek_slice_quote = make_peek_slice_fn(&field_extractor, &field, &info)?;
            peek_slice_fns_quote = quote! {
                #peek_slice_fns_quote
                #peek_slice_quote
            };
        }
    }
    let struct_size = &info.total_bytes();
    // construct from bytes function. use input_byte_buffer as input name because,
    // that is what the field quotes expect to extract from.
    // wrap our list of field names with commas with Self{} so we it instantiate our struct,
    // because all of the from_bytes field quote store there data in a temporary variable with the same
    // name as its destination field the list of field names will be just fine.
    let from_bytes_fn = quote! {
        fn from_bytes(mut input_byte_buffer: [u8;#struct_size]) -> Self {
            #from_bytes_quote
            Self{
                #from_bytes_struct_quote
            }
        }
    };
    Ok(FromBytesOptions {
        from_bytes_fn,
        peek_field_fns: peek_fns_quote,
        peek_slice_field_fns: if peek_slice {
            Some(peek_slice_fns_quote)
        } else {
            None
        },
    })
}

fn make_peek_slice_fn(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
) -> syn::Result<TokenStream> {
    let field_name = format_ident!("peek_slice_{}", field.ident.as_ref().clone());
    let type_ident = field.ty.type_quote();
    let min_length = if info.flip {
        ((info.total_bits() - field.attrs.bit_range.start) as f64 / 8.0f64).ceil() as usize
    } else {
        (field.attrs.bit_range.end as f64 / 8.0f64).ceil() as usize
    };
    Ok(quote! {
        pub fn #field_name(input_byte_buffer: &[u8]) -> Result<#type_ident, BitfieldSliceError> {
            let slice_length = input_byte_buffer.len();
            if slice_length < #min_length {
                Err(BitfieldSliceError(slice_length, #min_length))
            } else {
                Ok(
                    #field_quote
                )
            }
        }
    })
}

fn make_peek_fn(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
) -> syn::Result<TokenStream> {
    let field_name = format_ident!("peek_{}", field.ident.as_ref().clone());
    let type_ident = field.ty.type_quote();
    let struct_size = info.total_bytes();
    Ok(quote! {
        #[inline]
        pub fn #field_name(input_byte_buffer: &[u8;#struct_size]) -> #type_ident {
            #field_quote
        }
    })
}

/// if is_inner is false the field will be put into a variable with the fields name, otherwise
/// it will be returned.
fn get_field_quote(
    field: &FieldInfo,
    flip: Option<usize>,
) -> syn::Result<proc_macro2::TokenStream> {
    let value_retrieval = match field.ty {
        FieldDataType::ElementArray(_, _, _) => {
            let mut buffer = quote! {};
            let sub = field.get_element_iter()?;
            for sub_field in sub {
                let sub_field_quote = get_field_quote(&sub_field, flip)?;
                buffer = quote! {
                    #buffer
                    {#sub_field_quote},
                };
            }
            let buffer = quote! { [#buffer] };
            buffer
        }
        FieldDataType::BlockArray(_, _, _) => {
            let mut buffer = quote! {};
            let sub = field.get_block_iter()?;
            for sub_field in sub {
                let sub_field_quote = get_field_quote(&sub_field, flip)?;
                buffer = quote! {
                    #buffer
                    {#sub_field_quote},
                };
            }
            let buffer = quote! { [#buffer] };
            buffer
        }
        _ => match field.attrs.endianness.as_ref() {
            Endianness::Big => apply_be_math_to_field_access_quote(field, flip)?,
            Endianness::Little => apply_le_math_to_field_access_quote(field, flip)?,
            Endianness::None => apply_ne_math_to_field_access_quote(field, flip)?,
        },
    };

    let output = match field.ty {
        FieldDataType::Float(_, ref ident) => {
            quote! {#ident::from_bits(#value_retrieval)}
        }
        FieldDataType::Char(_, _) => {
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
        FieldDataType::Enum(_, _, ref ident) => {
            quote! {#ident::from_primitive(#value_retrieval)}
        }
        FieldDataType::Struct(_, ref ident) => {
            quote! {#ident::from_bytes({#value_retrieval})}
        }
        _ => {
            quote! {#value_retrieval}
        }
    };
    Ok(output)
}
fn apply_le_math_to_field_access_quote(
    field: &FieldInfo,
    flip: Option<usize>,
) -> syn::Result<proc_macro2::TokenStream> {
    let (amount_of_bits, zeros_on_left, available_bits_in_first_byte, mut starting_inject_byte) =
        BitMath::from_field(field)?.into_tuple();
    let operator = if let Some(flip) = flip {
        starting_inject_byte = flip - starting_inject_byte;
        quote! {-}
    } else {
        quote! {+}
    };
    // make a name for the buffer that we will store the number in byte form
    let field_buffer_name = format_ident!("{}_bytes", field.ident.as_ref());
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
        let size = field.ty.size();
        let mut full_quote = quote! {
            let mut #field_buffer_name: [u8;#size] = [0u8;#size];
        };

        let fields_last_bits_index = (amount_of_bits as f64 / 8.0f64).ceil() as usize - 1;
        let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);
        let mid_shift: u32 = 8 - available_bits_in_first_byte as u32;
        let next_bit_mask = get_left_and_mask(mid_shift as usize);
        let mut i = 0;
        while i != fields_last_bits_index {
            let start = if let None = flip {
                starting_inject_byte + i
            } else {
                starting_inject_byte - i
            };
            if available_bits_in_first_byte == 0 && right_shift == 0 {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#start];
                };
            }else{
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#start] & #current_bit_mask;
                };
                if available_bits_in_first_byte + (8 * i) < amount_of_bits {
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] |= input_byte_buffer[#start #operator 1] & #next_bit_mask;
                    }
                }

                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] = #field_buffer_name[#i].rotate_left(#mid_shift);
                };
            }
            i += 1;
        }
        if right_shift > 0 {
            let start = if let None = flip {
                starting_inject_byte + i
            } else {
                starting_inject_byte - i
            };
            let right_shift: u32 = right_shift.clone() as u32;
            full_quote = quote! {
                #full_quote
                #field_buffer_name[#i] |= input_byte_buffer[#start + 1] & #last_bit_mask;
                #field_buffer_name[#i] = #field_buffer_name[#i].rotate_left(#right_shift);
            };
        } else {
            let start = if let None = flip {
                starting_inject_byte + i
            } else {
                starting_inject_byte - i
            };
            // this should give us the last index of the field
            let left_shift: u32 = right_shift.clone().abs() as u32;
            let mid_mask = first_bit_mask & last_bit_mask;
            full_quote = quote! {
                #full_quote
                #field_buffer_name[#i] |= (input_byte_buffer[#start] & #mid_mask);
                #field_buffer_name[#i] = #field_buffer_name[#i].rotate_right(#left_shift);
            };
        }
        add_sign_fix_quote(&mut full_quote, &field, &amount_of_bits, &field_buffer_name);
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
                let alt_type_quote = if size == 4 {
                    quote!{u32}
                }else if size == 8 {
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
        let output_quote = match field.ty {
            FieldDataType::Number(_, ref sign, ref ident) => {
                let field_value = quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left)};
                if let NumberSignage::Signed = sign {
                    let mut value = quote!{
                        let mut #field_buffer_name = #field_value;
                    };
                    add_sign_fix_quote_single_bit(&mut value, &field, &amount_of_bits, &field_buffer_name);
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
                quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left) != 0}
            }
            FieldDataType::Char(_, _) => return Err(syn::Error::new(field.ident.span(), "Char not supported for single byte insert logic")),
            FieldDataType::Enum(ref primitive_ident, _, _) => quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left) as #primitive_ident},
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(field.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Float(_, _) => return Err(syn::Error::new(field.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad.")),
        };
        Ok(output_quote)
    }
}
fn apply_ne_math_to_field_access_quote(
    field: &FieldInfo,
    flip: Option<usize>,
) -> Result<proc_macro2::TokenStream, syn::Error> {
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
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let full_quote = match field.ty {
            FieldDataType::Number(_, _,_ ) |
            FieldDataType::Float(_, _) |
            FieldDataType::Char(_, _) => return Err(syn::Error::new(field.ident.span(), "Char was not given Endianness, please report this.")),
            FieldDataType::Boolean => return Err(syn::Error::new(field.ident.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Enum(_, _, _) => return Err(syn::Error::new(field.ident.span(), "Enum was not given Endianness, please report this.")),
            FieldDataType::Struct(ref size, _) => {
                let buffer_ident = format_ident!("{}_buffer", field.ident.as_ref());
                let mut quote_builder = quote!{let mut #buffer_ident: [u8;#size] = [0u8;#size];};
                if right_shift > 0 {
                    // right shift (this means that the last bits are in the first byte)
                    // because we are applying bits in place we need masks in insure we don't effect other fields
                    // data. we need one for the first byte and the last byte.
                    let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);
                    let next_bit_mask = get_left_and_mask(8 - available_bits_in_first_byte);
                    let right_shift: u32 = right_shift as u32;
                    for i in 0..*size {
                        let start = if let None = flip {starting_inject_byte + i}else{starting_inject_byte - i};
                        let mut first = quote!{
                            #buffer_ident[#i] = input_byte_buffer[#start] & #current_bit_mask;
                        };
                        if available_bits_in_first_byte + (8 * i) < amount_of_bits {
                            first = quote!{
                                #first
                                #buffer_ident[#i] |= input_byte_buffer[#start #operator 1] & #next_bit_mask;
                            };
                        }
                        quote_builder = quote!{
                            #quote_builder
                            #first
                            #buffer_ident[#i] = #buffer_ident[#i].rotate_left(#right_shift);
                        };
                    }
                }else if right_shift < 0{
                    return Err(syn::Error::new(
                        field.ident.span(),
                        "left shifting struct was removed to see if it would ever happened",
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
                }else{
                    // no shift can be more faster.
                    let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);
                    for i in 0..*size {
                        let start = if let None = flip {starting_inject_byte + i}else{starting_inject_byte - i};
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
                // return the value
                quote_builder = quote!{
                    #quote_builder
                    #buffer_ident
                };
                quote_builder
            }
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };

        Ok(full_quote)
    } else {
        if 8 < (zeros_on_left + amount_of_bits) {
            return Err(syn::Error::new(
                field.ident.span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (zeros_on_left + amount_of_bits);
        let mask = get_right_and_mask(available_bits_in_first_byte)
            & get_left_and_mask(8 - zeros_on_right);
        if 8 < amount_of_bits || 8 - amount_of_bits < field.attrs.bit_range.start % 8 {
            return Err(syn::Error::new(
                field.ident.span(),
                "calculating ne shift_left failed",
            ));
        }
        let shift_left = (8 - amount_of_bits) - (field.attrs.bit_range.start % 8);

        let output = match field.ty {
            FieldDataType::Number(_, _, _) => return Err(syn::Error::new(field.ident.span(), "Number was not given Endianness, please report this")),
            FieldDataType::Boolean => {
                quote!{(((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left) != 0)}
            }
            FieldDataType::Char(_, _) => return Err(syn::Error::new(field.ident.span(), "Char not supported for single byte insert logic")),
            FieldDataType::Enum(_, _, _) => return Err(syn::Error::new(field.ident.span(), "Enum was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Struct(_, _) => {
                let used_bits_in_byte = 8 - available_bits_in_first_byte;
                quote!{([((input_byte_buffer[#starting_inject_byte] & #mask)) << #used_bits_in_byte])}
            }
            FieldDataType::Float(_, _) => return Err(syn::Error::new(field.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };
        Ok(output)
    }
}
///
/// # Arguments
/// * `field' - reference to the FieldInfo.
/// * `field_access_quote` - a quote containing access to to byte array of the field.
///                             ex. quote!{(self.char_field as u32)}
fn apply_be_math_to_field_access_quote(
    field: &FieldInfo,
    flip: Option<usize>,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let (amount_of_bits, zeros_on_left, available_bits_in_first_byte, mut starting_inject_byte) =
        BitMath::from_field(field)?.into_tuple();
    if let Some(flip) = flip {
        starting_inject_byte = flip - starting_inject_byte;
    }

    // make a name for the buffer that we will store the number in byte form
    let field_buffer_name = format_ident!("{}_bytes", field.ident.as_ref());
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
        let right_shift: i8 =
            ((amount_of_bits % 8) as i8) - ((available_bits_in_first_byte % 8) as i8);
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
                quote! { .rotate_right(#left_shift) },
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
                    quote! {}
                } else {
                    // shift right code
                    let right_shift_usize: u32 = right_shift.clone() as u32;
                    quote! { .rotate_left(#right_shift_usize) }
                },
                match get_be_starting_index(amount_of_bits, right_shift, field.struct_byte_size()) {
                    Ok(good) => good,
                    Err(err) => return Err(syn::Error::new(field.ident.span(), err)),
                },
            )
        };
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let output = match field.ty {
            FieldDataType::Number(size, _, ref type_quote) |
            FieldDataType::Enum(ref type_quote, size, _) => {
                let full_quote = build_number_quote(field, amount_of_bits, bits_in_last_byte, field_buffer_name, size, first_bits_index, starting_inject_byte, first_bit_mask, last_bit_mask, right_shift, available_bits_in_first_byte, flip);
                let apply_field_to_buffer = quote! {
                    #type_quote::from_be_bytes({
                        #full_quote
                    })#shift
                };
                apply_field_to_buffer
            }
            FieldDataType::Float(size, _) => {
                let alt_type_quote = if size == 4 {
                    quote!{u32}
                }else if size == 8 {
                    quote!{u64}
                }else{
                    return Err(syn::Error::new(field.ident.span(), "unsupported floating type"))
                };
                let full_quote = build_number_quote(field, amount_of_bits, bits_in_last_byte, field_buffer_name, size, first_bits_index, starting_inject_byte, first_bit_mask, last_bit_mask, right_shift, available_bits_in_first_byte, flip);
                let apply_field_to_buffer = quote! {
                    #alt_type_quote::from_be_bytes({
                        #full_quote
                    })#shift
                };
                apply_field_to_buffer
            }
            FieldDataType::Char(size, _) => {
                let full_quote = build_number_quote(field, amount_of_bits, bits_in_last_byte, field_buffer_name, size, first_bits_index, starting_inject_byte, first_bit_mask, last_bit_mask, right_shift, available_bits_in_first_byte, flip);
                let apply_field_to_buffer = quote! {
                    u32::from_be_bytes({
                        #full_quote
                    })#shift
                };
                apply_field_to_buffer
            }
            FieldDataType::Boolean => return Err(syn::Error::new(field.ident.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(field.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad."))
        };

        Ok(output)
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
        let output_quote = match field.ty {
            FieldDataType::Number(_, ref sign,ref ident) => {
                let field_value = quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left)};
                if let NumberSignage::Signed = sign {
                    let mut value = quote!{
                        let mut #field_buffer_name = #field_value;
                    };
                    add_sign_fix_quote_single_bit(&mut value, &field, &amount_of_bits, &field_buffer_name);
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
                quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left) != 0}
            }
            FieldDataType::Char(_, _) => return Err(syn::Error::new(field.ident.span(), "Char not supported for single byte insert logic")),
            FieldDataType::Enum(ref primitive_ident, _, _) => quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left) as #primitive_ident},
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(field.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Float(_, _) => return Err(syn::Error::new(field.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad.")),
        };
        Ok(output_quote)
    }
}

fn build_number_quote(
    field: &FieldInfo,
    amount_of_bits: usize,
    bits_in_last_byte: usize,
    field_buffer_name: syn::Ident,
    size: usize,
    first_bits_index: usize,
    starting_inject_byte: usize,
    first_bit_mask: u8,
    last_bit_mask: u8,
    right_shift: i8,
    available_bits_in_first_byte: usize,
    flip: Option<usize>,
) -> TokenStream {
    let mut full_quote = quote! {
        let mut #field_buffer_name: [u8;#size] = [0u8;#size];
        #field_buffer_name[#first_bits_index] = input_byte_buffer[#starting_inject_byte] & #first_bit_mask;
    };
    // fill in the rest of the bits
    let mut current_byte_index_in_buffer: usize = if let None = flip {
        starting_inject_byte + 1
    } else {
        starting_inject_byte - 1
    };
    if right_shift > 0 {
        // right shift (this means that the last bits are in the first byte)
        if available_bits_in_first_byte + bits_in_last_byte != amount_of_bits {
            for i in first_bits_index + 1usize..field.ty.size() {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#current_byte_index_in_buffer] ;
                };
                current_byte_index_in_buffer = if let None = flip {
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
            for i in first_bits_index + 1..field.ty.size() - 1 {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#current_byte_index_in_buffer];
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
        //TODO make rotation optimizer.
        full_quote = quote! {
            #full_quote
            #field_buffer_name[#final_index] |= input_byte_buffer[#current_byte_index_in_buffer] & #last_bit_mask;
        };
        add_sign_fix_quote(&mut full_quote, &field, &amount_of_bits, &field_buffer_name);
        full_quote = quote! {
            #full_quote
            #field_buffer_name
        };
    }
    full_quote
}

fn isolate_sign_bit_mask(bit_index: &usize) -> u8 {
    match bit_index {
        7 => 0b01000000,
        6 => 0b00100000,
        5 => 0b00010000,
        4 => 0b00001000,
        3 => 0b00000100,
        2 => 0b00000010,
        1 => 0b00000001,
        _ => 0b10000000,
    }
}

fn add_sign_fix_quote(
    field_quote: &mut TokenStream,
    field: &FieldInfo,
    amount_of_bits: &usize,
    field_buffer_quote: &proc_macro2::Ident,
) {
    if let FieldDataType::Number(ref size, ref sign, _) = field.ty {
        if *amount_of_bits != *size * 8 {
            if let NumberSignage::Signed = sign {
                let bits_in_last_byte = amount_of_bits % 8;
                let bit_index = 8 - bits_in_last_byte;
                let sign_mask = isolate_sign_bit_mask(&bit_index);
                let take_sign_mask = !sign_mask;
                let sign_index = match field.attrs.endianness.as_ref() {
                    Endianness::Big => {
                        (size - 1) - (((*amount_of_bits as f64 / 8.0_f64).ceil() - 1_f64) as usize)
                    }
                    Endianness::Little => {
                        ((*amount_of_bits as f64 / 8.0_f64).ceil() - 1_f64) as usize
                    }
                    Endianness::None => return,
                };
                let mut sign_bit = quote! {
                    (#field_buffer_quote[#sign_index] & #sign_mask)
                };
                if bit_index == 0 {
                    sign_bit = quote! {
                        (#sign_bit << #bit_index)
                    };
                }
                let mut add_me = quote! {
                    #field_quote
                    #field_buffer_quote[0] = #sign_bit & 0b01111111;
                    #field_buffer_quote[#sign_index] &= #take_sign_mask;
                };
                std::mem::swap(&mut add_me, field_quote);
            }
        }
    }
}

fn add_sign_fix_quote_single_bit(
    field_quote: &mut TokenStream,
    field: &FieldInfo,
    amount_of_bits: &usize,
    field_buffer_quote: &proc_macro2::Ident,
) {
    if let FieldDataType::Number(ref size, ref sign, _) = field.ty {
        if *amount_of_bits != *size * 8 {
            if let NumberSignage::Signed = sign {
                let bits_in_last_byte = amount_of_bits % 8;
                let bit_index = 8 - bits_in_last_byte;
                let sign_mask = isolate_sign_bit_mask(&bit_index);
                let mut sign_bit = quote! {
                    (#field_buffer_quote & #sign_mask)
                };
                let take_sign_mask = !sign_mask;
                if bit_index == 0 {
                    sign_bit = quote! {
                        (#sign_bit << #bit_index)
                    };
                }
                let mut add_me = quote! {
                    #field_quote
                    #field_buffer_quote = #sign_bit & 0b01111111;
                    #field_buffer_quote &= #take_sign_mask;
                };
                std::mem::swap(&mut add_me, field_quote);
            }
        }
    }
}
