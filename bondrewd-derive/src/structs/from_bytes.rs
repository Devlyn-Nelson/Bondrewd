use std::{cmp::Ordering, collections::VecDeque, str::FromStr};

use crate::structs::common::{
    get_be_starting_index, get_left_and_mask, get_right_and_mask, BitMath, Endianness,
    FieldDataType, FieldInfo, StructInfo,
};

use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated,
    token::{Comma, Pub},
};

use super::common::{EnumInfo, NumberSignage};

pub struct GeneratedFromBytesFunctions {
    pub from_bytes_fn: TokenStream,
    pub peek_field_fns: TokenStream,
    pub from_slice_field_fns: Option<TokenStream>,
    pub peek_slice_field_fns: Option<TokenStream>,
    pub peek_slice_field_unchecked_fns: Option<TokenStream>,
}

struct FieldQuotes {
    field_name_list: TokenStream,
    from_bytes_quote: TokenStream,
    from_vec_quote: TokenStream,
    peek_fns_quote: TokenStream,
    peek_slice_fns_option: Option<(TokenStream, TokenStream)>,
}
fn make_read_fns(
    field: &FieldInfo,
    info: &StructInfo,
    enum_name: &Option<Ident>,
    peek_fns_quote: &mut TokenStream,
    peek_slice_fns_option: &mut Option<(TokenStream, TokenStream)>,
) -> syn::Result<TokenStream> {
    // let peek_name = if let Some((prefix, _, _)) = enum_name {
    //     format_ident!("read_{prefix}_{}", field_name.as_ref())
    // } else{
    //     format_ident!("read_{}", field_name.as_ref())
    // };
    let field_extractor = get_field_quote(
        field,
        if info.attrs.flip {
            Some(info.total_bytes() - 1)
        } else {
            None
        },
    )?;

    let peek_quote = generate_read_field_fn(&field_extractor, field, info, enum_name);
    *peek_fns_quote = quote! {
        #peek_fns_quote
        #peek_quote
    };
    // make the slice functions if applicable.
    if let Some((ref mut the_peek_slice_fns_quote, ref mut unchecked_quote)) = peek_slice_fns_option
    {
        let peek_slice_quote =
            generate_read_slice_field_fn(&field_extractor, field, info, enum_name);
        let peek_slice_unchecked_quote =
            generate_read_slice_field_fn_unchecked(&field_extractor, field, info, enum_name);
        let mut the_peek_slice_fns_quote_temp = quote! {
            #the_peek_slice_fns_quote
            #peek_slice_quote
        };
        let mut unchecked_quote_temp = quote! {
            #unchecked_quote
            #peek_slice_unchecked_quote
        };
        std::mem::swap(the_peek_slice_fns_quote, &mut the_peek_slice_fns_quote_temp);
        std::mem::swap(unchecked_quote, &mut unchecked_quote_temp);
    }
    Ok(field_extractor)
}

fn get_check_slice_fn(
    name: &Ident,
    // total_bytes
    check_size: usize,
) -> TokenStream {
    let checked_ident = format_ident!("{name}Checked");
    let comment = format!(
        "Returns a [{checked_ident}] which allows you to read any field for a `{name}` from provided slice.",
    );
    quote! {
        #[doc = #comment]
        pub fn check_slice(buffer: &[u8]) -> Result<#checked_ident, bondrewd::BitfieldLengthError> {
            let buf_len = buffer.len();
            if buf_len >= #check_size {
                Ok(#checked_ident {
                    buffer
                })
            }else{
                Err(bondrewd::BitfieldLengthError(buf_len, #check_size))
            }
        }
    }
}

fn create_fields_quotes(
    info: &StructInfo,
    enum_name: Option<&Ident>,
    peek_slice: bool,
) -> syn::Result<FieldQuotes> {
    let lower_name = if enum_name.is_some() {
        Some(format_ident!(
            "{}",
            info.name.to_string().to_case(Case::Snake)
        ))
    } else {
        None
    };
    let mut field_name_list = quote! {};
    // all of the fields extraction will be appended to this
    let mut from_bytes_quote = quote! {};
    let mut from_vec_quote = quote! {};
    // all quote with all of the peek slice functions appended to it. the second tokenstream is an unchecked
    // version for the checked_struct.
    let mut peek_slice_fns_option: Option<(TokenStream, TokenStream)> = if peek_slice {
        if enum_name.is_some() {
            Some((quote! {}, quote! {}))
        } else {
            Some((
                get_check_slice_fn(&info.name, info.total_bytes()),
                quote! {},
            ))
        }
    } else {
        None
    };
    // all quote with all of the peek functions appended to it.
    let mut peek_fns_quote = quote! {};
    // TODO make each variant decide if the id field needs to be accounted for.
    // currently the id fields is added the each enum variant as the first field so we
    // skip it assuming there will be one made that is common across all variants.
    let fields = if enum_name.is_some() && !info.fields[0].attrs.capture_id {
        &info.fields[1..]
    } else {
        &info.fields[..]
    };

    for field in fields {
        let field_name = &field.ident().ident();
        let field_extractor = make_read_fns(
            field,
            info,
            &lower_name,
            &mut peek_fns_quote,
            &mut peek_slice_fns_option,
        )?;
        // fake fields do not exist in the actual structure and should only have functions
        // that read or write values into byte arrays.
        if !field.attrs.reserve.is_fake_field() {
            // put the name of the field into the list of fields that are needed to create
            // the struct.
            field_name_list = quote! {#field_name_list #field_name,};
            let peek_call = if field.attrs.capture_id {
                // put the field extraction in the actual from bytes.
                if field.attrs.reserve.read_field() {
                    let id_name = format_ident!("{}", EnumInfo::VARIANT_ID_NAME);
                    quote! {
                        let #field_name = #id_name;
                    }
                } else {
                    return Err(syn::Error::new(
                        field.span(),
                        "fields with attribute 'capture_id' are automatically considered 'read_only', meaning it can not have the 'reserve' attribute.",
                    ));
                }
            } else {
                // put the field extraction in the actual from bytes.
                if field.attrs.reserve.read_field() {
                    let fn_field_name = if let Some(ref p) = lower_name {
                        format_ident!("read_{p}_{field_name}")
                    } else {
                        format_ident!("read_{field_name}")
                    };
                    quote! {
                        let #field_name = Self::#fn_field_name(&input_byte_buffer);
                    }
                } else {
                    quote! { let #field_name = Default::default(); }
                }
            };
            from_bytes_quote = quote! {
                #from_bytes_quote
                #peek_call;
            };
            from_vec_quote = quote! {
                #from_vec_quote
                let #field_name = #field_extractor;
            };
        }
    }
    Ok(FieldQuotes {
        field_name_list,
        from_bytes_quote,
        from_vec_quote,
        peek_fns_quote,
        peek_slice_fns_option,
    })
}
#[allow(clippy::too_many_lines)]
pub fn create_from_bytes_field_quotes_enum(
    info: &EnumInfo,
    peek_slice: bool,
) -> Result<GeneratedFromBytesFunctions, syn::Error> {
    let mut from_bytes_fn: TokenStream = quote! {};
    let mut from_vec_fn: TokenStream = quote! {};
    let (mut peek_fns_quote, mut peek_slice_fns_option) = {
        (
            {
                let field = info.generate_id_field()?;
                let flip = false;
                let field_extractor = get_field_quote(
                    &field,
                    if flip {
                        // condition use to be `info.attrs.flip` i think this only applies to the variants
                        // and id_position is what is used here. but it should be done none the less.
                        Some(info.total_bytes() - 1)
                    } else {
                        None
                    },
                )?;
                let attrs = info.attrs.attrs.clone();
                let mut fields = vec![field.clone()];
                fields[0].attrs.bit_range = 0..info.total_bits();
                let temp_struct_info = StructInfo {
                    name: info.name.clone(),
                    attrs,
                    fields,
                    vis: syn::Visibility::Public(Pub::default()),
                    tuple: false,
                };
                let id_field =
                    generate_read_field_fn(&field_extractor, &field, &temp_struct_info, &None);
                if peek_slice {
                    let id_slice_peek = generate_read_slice_field_fn(
                        &field_extractor,
                        &field,
                        &temp_struct_info,
                        &None,
                    );
                    quote! {
                        #id_field
                        #id_slice_peek
                    }
                } else {
                    quote! {
                        #id_field
                    }
                }
            },
            if peek_slice {
                Some((
                    get_check_slice_fn(&info.name, info.total_bytes()),
                    quote! {},
                ))
            } else {
                None
            },
        )
    };
    let struct_size = info.total_bytes();
    let last_variant = info.variants.len() - 1;
    for (i, variant) in info.variants.iter().enumerate() {
        // this is the slice indexing that will fool the set function code into thinking
        // it is looking at a smaller array.
        let v_name = &variant.name;
        let upper_v_name = v_name.to_string().to_case(Case::UpperSnake);
        let v_byte_const_name = format_ident!("{upper_v_name}_BYTE_SIZE");
        let v_bit_const_name = format_ident!("{upper_v_name}_BIT_SIZE");
        let v_byte_size = variant.total_bytes();
        let v_bit_size = variant.total_bits();
        let variant_name = quote! {#v_name};
        let (
            field_name_list,
            peek_fns_quote_temp,
            from_bytes_quote,
            peek_slice_fns_option_temp,
            from_vec_quote,
        ) = {
            let thing = create_fields_quotes(variant, Some(&info.name), peek_slice)?;
            (
                thing.field_name_list,
                thing.peek_fns_quote,
                thing.from_bytes_quote,
                thing.peek_slice_fns_option,
                thing.from_vec_quote,
            )
        };
        if let (
            Some((mut peek_slice_fns_quote_temp, mut unchecked_temp)),
            Some((peek_slice_fns_quote, unchecked)),
        ) = (peek_slice_fns_option_temp, peek_slice_fns_option.as_mut())
        {
            peek_slice_fns_quote_temp = quote! {
                pub const #v_byte_const_name: usize = #v_byte_size;
                pub const #v_bit_const_name: usize = #v_bit_size;
                #peek_slice_fns_quote
                #peek_slice_fns_quote_temp
            };
            unchecked_temp = quote! {
                #unchecked
                #unchecked_temp
            };
            std::mem::swap(peek_slice_fns_quote, &mut peek_slice_fns_quote_temp);
            std::mem::swap(unchecked, &mut unchecked_temp);
        }
        peek_fns_quote = quote! {
            #peek_fns_quote
            #peek_fns_quote_temp
        };
        // make setter for each field.
        // construct from bytes function. use input_byte_buffer as input name because,
        // that is what the field quotes expect to extract from.
        // wrap our list of field names with commas with Self{} so we it instantiate our struct,
        // because all of the from_bytes field quote store there data in a temporary variable with the same
        // name as its destination field the list of field names will be just fine.

        let variant_id = if i == last_variant {
            quote! {_}
        } else if let Some(id) = variant.attrs.id {
            if let Ok(yes) = TokenStream::from_str(&format!("{id}")) {
                yes
            } else {
                return Err(syn::Error::new(
                    variant.name.span(),
                    "failed to construct id, this is a bug in bondrewd.",
                ));
            }
        } else {
            return Err(syn::Error::new(
                variant.name.span(),
                "failed to find id for variant, this is a bug in bondrewd.",
            ));
        };
        let variant_constructor = if field_name_list.is_empty() {
            quote! {Self::#variant_name}
        } else if variant.tuple {
            quote! {Self::#variant_name ( #field_name_list )}
        } else {
            quote! {Self::#variant_name { #field_name_list }}
        };
        from_bytes_fn = quote! {
            #from_bytes_fn
            #variant_id => {
                #from_bytes_quote
                #variant_constructor
            }
        };
        if peek_slice {
            from_vec_fn = quote! {
                #from_vec_fn
                #variant_id => {
                    #from_vec_quote
                    #variant_constructor
                }
            };
        }
    }
    let v_id = format_ident!("{}", EnumInfo::VARIANT_ID_NAME);
    let v_id_call = format_ident!("read_{v_id}");
    let v_id_slice_call = format_ident!("read_slice_{v_id}");
    let from_bytes_fn = quote! {
        fn from_bytes(mut input_byte_buffer: [u8;#struct_size]) -> Self {
            let #v_id = Self::#v_id_call(&input_byte_buffer);
            match #v_id {
                #from_bytes_fn
            }
        }
    };
    if let Some((peek_slice_field_fns, peek_slice_field_unchecked_fns)) = peek_slice_fns_option {
        let comment_take = "Creates a new instance of `Self` by copying field from the bitfields, removing bytes that where used. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
        let comment = "Creates a new instance of `Self` by copying field from the bitfields. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
        let from_slice_field_fns = quote! {
            #[doc = #comment_take]
            fn from_vec(input_byte_buffer: &mut Vec<u8>) -> Result<Self, bondrewd::BitfieldLengthError> {
                if input_byte_buffer.len() < Self::BYTE_SIZE {
                    return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                }
                let #v_id = Self::#v_id_slice_call(&input_byte_buffer)?;
                let out = match #v_id {
                    #from_vec_fn
                };
                let _ = input_byte_buffer.drain(..Self::BYTE_SIZE);
                Ok(out)
            }
            #[doc = #comment]
            fn from_slice(input_byte_buffer: &[u8]) -> Result<Self, bondrewd::BitfieldLengthError> {
                if input_byte_buffer.len() < Self::BYTE_SIZE {
                    return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                }
                let #v_id = Self::#v_id_slice_call(&input_byte_buffer)?;
                let out = match #v_id {
                    #from_vec_fn
                };
                Ok(out)
            }
        };
        Ok(GeneratedFromBytesFunctions {
            from_bytes_fn,
            peek_field_fns: peek_fns_quote,
            from_slice_field_fns: Some(from_slice_field_fns),
            peek_slice_field_fns: Some(peek_slice_field_fns),
            peek_slice_field_unchecked_fns: Some(peek_slice_field_unchecked_fns),
        })
    } else {
        Ok(GeneratedFromBytesFunctions {
            from_bytes_fn,
            peek_field_fns: peek_fns_quote,
            from_slice_field_fns: None,
            peek_slice_field_fns: None,
            peek_slice_field_unchecked_fns: None,
        })
    }
}

pub fn create_from_bytes_field_quotes(
    info: &StructInfo,
    peek_slice: bool,
) -> Result<GeneratedFromBytesFunctions, syn::Error> {
    let (
        peek_fns_quote,
        from_bytes_struct_quote,
        from_bytes_quote,
        peek_slice_fns_option,
        from_vec_fn,
    ) = {
        let thing = create_fields_quotes(info, None, peek_slice)?;
        (
            thing.peek_fns_quote,
            thing.field_name_list,
            thing.from_bytes_quote,
            thing.peek_slice_fns_option,
            thing.from_vec_quote,
        )
    };

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
    if let Some((peek_slice_field_fns, peek_slice_field_unchecked_fns)) = peek_slice_fns_option {
        let comment_take = "Creates a new instance of `Self` by copying field from the bitfields, removing bytes that where used. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
        let comment = "Creates a new instance of `Self` by copying field from the bitfields. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
        let from_slice_field_fns = quote! {
            #[doc = #comment_take]
            fn from_vec(input_byte_buffer: &mut Vec<u8>) -> Result<Self, bondrewd::BitfieldLengthError> {
                if input_byte_buffer.len() < Self::BYTE_SIZE {
                    return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                }
                let out = {
                    #from_vec_fn
                    Self{
                        #from_bytes_struct_quote
                    }
                };
                let _ = input_byte_buffer.drain(..Self::BYTE_SIZE);
                Ok(out)
            }
            #[doc = #comment]
            fn from_slice(input_byte_buffer: &[u8]) -> Result<Self, bondrewd::BitfieldLengthError> {
                if input_byte_buffer.len() < Self::BYTE_SIZE {
                    return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                }
                let out = {
                    #from_vec_fn
                    Self{
                        #from_bytes_struct_quote
                    }
                };
                Ok(out)
            }
        };
        Ok(GeneratedFromBytesFunctions {
            from_bytes_fn,
            peek_field_fns: peek_fns_quote,
            from_slice_field_fns: Some(from_slice_field_fns),
            peek_slice_field_fns: Some(peek_slice_field_fns),
            peek_slice_field_unchecked_fns: Some(peek_slice_field_unchecked_fns),
        })
    } else {
        Ok(GeneratedFromBytesFunctions {
            from_bytes_fn,
            peek_field_fns: peek_fns_quote,
            from_slice_field_fns: None,
            peek_slice_field_fns: None,
            peek_slice_field_unchecked_fns: None,
        })
    }
}

/// Generates a `read_slice_field_name()` function for a slice.
fn generate_read_slice_field_fn(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    prefix: &Option<Ident>,
) -> TokenStream {
    let field_name = if let Some(p) = prefix {
        format_ident!("{p}_{}", field.ident().ident())
    } else {
        field.ident().ident()
    };
    let fn_field_name = format_ident!("read_slice_{field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let min_length = if info.attrs.flip {
        (info.total_bits() - field.attrs.bit_range.start).div_ceil(8)
    } else {
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
fn generate_read_slice_field_fn_unchecked(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    prefix: &Option<Ident>,
) -> TokenStream {
    let field_name = if let Some(p) = prefix {
        format_ident!("{p}_{}", field.ident().ident())
    } else {
        field.ident().ident()
    };
    let fn_field_name = format_ident!("read_{field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let comment = format!(
        "Reads bits {} through {} in pre-checked slice, getting the `{field_name}` field of a [{struct_name}] in bitfield form.", bit_range.start, bit_range.end - 1
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

/// Generates a `read_field_name()` function.
fn generate_read_field_fn(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    prefix: &Option<Ident>,
) -> TokenStream {
    let field_name = if let Some(p) = prefix {
        format_ident!("{p}_{}", field.ident().ident())
    } else {
        field.ident().ident()
    };
    let fn_field_name = format_ident!("read_{field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let struct_size = &info.total_bytes();
    let comment = format!("Reads bits {} through {} within `input_byte_buffer`, getting the `{field_name}` field of a `{struct_name}` in bitfield form.", bit_range.start, bit_range.end - 1);
    quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(input_byte_buffer: &[u8;#struct_size]) -> #type_ident {
            #field_quote
        }
    }
}

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
#[allow(clippy::too_many_lines)]
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
    let field_buffer_name = format_ident!("{}_bytes", field.ident().ident());
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
        #[allow(clippy::cast_possible_truncation)]
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
        let new_array_quote =
            if let Some(a) = add_sign_fix_quote(field, amount_of_bits, right_shift)? {
                a
            } else {
                quote! {[0u8;#size]}
            };
        let mut full_quote = quote! {
            let mut #field_buffer_name: [u8;#size] = #new_array_quote;
        };
        // normally we would round up and subtract one, but clippy doesn't like it.
        let fields_last_bits_index = amount_of_bits.div_ceil(8) - 1;
        let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);
        #[allow(clippy::cast_possible_truncation)]
        let mid_shift: u32 = 8 - available_bits_in_first_byte as u32;
        let next_bit_mask = get_left_and_mask(mid_shift as usize);
        let mut i = 0;
        while i != fields_last_bits_index {
            let start = if flip.is_none() {
                starting_inject_byte + i
            } else {
                starting_inject_byte - i
            };
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
                let mut field_value = quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left)};
                if let NumberSignage::Signed = sign {
                    field_value = add_sign_fix_quote_single_bit(field_value, field, amount_of_bits, starting_inject_byte);
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
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(field.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Float(_, _) => return Err(syn::Error::new(field.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad.")),
        };
        Ok(output_quote)
    }
}
#[allow(clippy::too_many_lines)]
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
        #[allow(clippy::cast_possible_truncation)]
        let right_shift: i8 = 8_i8 - ((available_bits_in_first_byte % 8) as i8);
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let full_quote = match field.ty {
            FieldDataType::Number(_, _,_ ) |
            FieldDataType::Float(_, _) |
            FieldDataType::Char(_, _) => return Err(syn::Error::new(field.ident.span(), "Char was not given Endianness, please report this.")),
            FieldDataType::Boolean => return Err(syn::Error::new(field.ident.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Enum(_, _, _) => return Err(syn::Error::new(field.ident.span(), "Enum was not given Endianness, please report this.")),
            FieldDataType::Struct(ref size, _) => {
                let buffer_ident = format_ident!("{}_buffer", field.ident().ident());
                let mut quote_builder = quote!{let mut #buffer_ident: [u8;#size] = [0u8;#size];};
                match right_shift.cmp(&0) {
                    Ordering::Greater => {
                        // right shift (this means that the last bits are in the first byte)
                        // because we are applying bits in place we need masks in insure we don't effect other fields
                        // data. we need one for the first byte and the last byte.
                        let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);
                        let next_bit_mask = get_left_and_mask(8 - available_bits_in_first_byte);
                        let right_shift: u32 = u32::from(right_shift.unsigned_abs());
                        for i in 0..*size {
                            let start = if flip.is_none() {starting_inject_byte + i}else{starting_inject_byte - i};
                            let mut first = if current_bit_mask == u8::MAX {
                                quote!{
                                    #buffer_ident[#i] = input_byte_buffer[#start];
                                }
                            }else{
                                quote!{
                                    #buffer_ident[#i] = input_byte_buffer[#start] & #current_bit_mask;
                                }
                            };
                            if available_bits_in_first_byte + (8 * i) < amount_of_bits && next_bit_mask != 0 {
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
                    }
                    Ordering::Less => {
                        return Err(syn::Error::new(
                            field.ident.span(),
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
                        let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);
                        for i in 0..*size {
                            let start = if flip.is_none() {starting_inject_byte + i}else{starting_inject_byte - i};
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

        let output = match field.ty {
            FieldDataType::Number(_, _, _) => return Err(syn::Error::new(field.ident.span(), "Number was not given Endianness, please report this")),
            FieldDataType::Boolean => {
                quote!{(((input_byte_buffer[#starting_inject_byte] & #mask)) != 0)}
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
/// # Arguments
/// * `field` - reference to the `FieldInfo`.
/// * `field_access_quote` - a quote containing access to to byte array of the field.
///                             ex. `quote!{(self.char_field as u32)}`
#[allow(clippy::too_many_lines)]
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
    let field_buffer_name = format_ident!("{}_bytes", field.ident().ident());
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
        // how many times to shift the number right(for into_bytes).
        // NOTE if negative shift left.
        // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        #[allow(clippy::cast_possible_truncation)]
        let mut right_shift: i8 =
            ((amount_of_bits % 8) as i8) - ((available_bits_in_first_byte % 8) as i8);
        // TODO this right_shift modification is a fix because left shifts in be number are broken.
        // this exists in both from and into bytes for big endian. right shift should not be mut.
        if right_shift < 0 {
            right_shift += 8;
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
            let left_shift: u32 = u32::from(right_shift.unsigned_abs());
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
                    let right_shift_usize: u32 = u32::from(right_shift.unsigned_abs());
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
                let full_quote = build_number_quote(field, BuildNumberQuotePackage { amount_of_bits, bits_in_last_byte, field_buffer_name, size, first_bits_index, starting_inject_byte, first_bit_mask, last_bit_mask, right_shift, available_bits_in_first_byte, flip})?;
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
                let full_quote = build_number_quote(field, BuildNumberQuotePackage { amount_of_bits, bits_in_last_byte, field_buffer_name, size, first_bits_index, starting_inject_byte, first_bit_mask, last_bit_mask, right_shift, available_bits_in_first_byte, flip})?;
                let apply_field_to_buffer = quote! {
                    #alt_type_quote::from_be_bytes({
                        #full_quote
                    })#shift
                };
                apply_field_to_buffer
            }
            FieldDataType::Char(size, _) => {
                let full_quote = build_number_quote(field, BuildNumberQuotePackage { amount_of_bits, bits_in_last_byte, field_buffer_name, size, first_bits_index, starting_inject_byte, first_bit_mask, last_bit_mask, right_shift, available_bits_in_first_byte, flip})?;
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
                let mut field_value = quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left)};
                if let NumberSignage::Signed = sign {
                    field_value = add_sign_fix_quote_single_bit(field_value, field, amount_of_bits, starting_inject_byte);
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
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(field.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Float(_, _) => return Err(syn::Error::new(field.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(field.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad.")),
        };
        Ok(output_quote)
    }
}
struct BuildNumberQuotePackage {
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
}
fn build_number_quote(
    field: &FieldInfo,
    stuff: BuildNumberQuotePackage,
) -> syn::Result<TokenStream> {
    let amount_of_bits = stuff.amount_of_bits;
    let bits_in_last_byte = stuff.bits_in_last_byte;
    let field_buffer_name = stuff.field_buffer_name;
    let size = stuff.size;
    let first_bits_index = stuff.first_bits_index;
    let starting_inject_byte = stuff.starting_inject_byte;
    let first_bit_mask = stuff.first_bit_mask;
    let last_bit_mask = stuff.last_bit_mask;
    let right_shift = stuff.right_shift;
    let available_bits_in_first_byte = stuff.available_bits_in_first_byte;
    let flip = stuff.flip;
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
            for i in first_bits_index + 1usize..field.ty.size() {
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
            for i in first_bits_index + 1..field.ty.size() - 1 {
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
        let final_index = field.ty.size() - 1;
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
