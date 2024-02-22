use std::str::FromStr;

use crate::structs::common::{EnumInfo, FieldInfo, ObjectInfo, StructInfo};
use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::token::Pub;

pub struct GeneratedFunctions {
    /// Functions that belong in `Bitfields` impl for object.
    pub bitfield_trait_impl_fns: proc_macro2::TokenStream,
    /// Functions that belong in impl for object.
    pub impl_fns: proc_macro2::TokenStream,
    /// Functions that belong in impl for generated checked slice object.
    #[cfg(feature = "dyn_fns")]
    pub checked_struct_impl_fns: proc_macro2::TokenStream,
    /// Functions that belong in `BitfieldsDyn` impl for object.
    #[cfg(feature = "dyn_fns")]
    pub bitfield_dyn_trait_impl_fns: proc_macro2::TokenStream,
}

impl Default for GeneratedFunctions {
    fn default() -> Self {
        Self {
            bitfield_trait_impl_fns: Default::default(),
            impl_fns: Default::default(),
            #[cfg(feature = "dyn_fns")]
            checked_struct_impl_fns: Default::default(),
            #[cfg(feature = "dyn_fns")]
            bitfield_dyn_trait_impl_fns: Default::default(),
        }
    }
}

impl Into<TokenStream> for GeneratedFunctions {
    fn into(self) -> TokenStream {
        let trait_fns = self.bitfield_trait_impl_fns;
        let impl_fns = self.impl_fns;
        #[cfg(feature = "dyn_fns")]
        let unchecked = self.checked_struct_impl_fns;
        #[cfg(feature = "dyn_fns")]
        let dyn_trait_fns = self.bitfield_dyn_trait_impl_fns;
        #[cfg(feature = "dyn_fns")]
        let quote = quote! {
            #trait_fns
            #impl_fns
            #unchecked
            #dyn_trait_fns
        };
        #[cfg(not(feature = "dyn_fns"))]
        let quote = quote! {
            #trait_fns
            #impl_fns
        };
        quote
    }
}

impl GeneratedFunctions {
    fn merge(&mut self, other: Self) {
        let bitfield_trait_impl_fns = &self.bitfield_trait_impl_fns;
        let other_bitfield_trait_impl_fns = &other.bitfield_trait_impl_fns;
        self.bitfield_trait_impl_fns = quote! {
            #bitfield_trait_impl_fns
            #other_bitfield_trait_impl_fns
        };
        let impl_fns = &self.impl_fns;
        let other_impl_fns = &other.impl_fns;
        self.impl_fns = quote! {
            #impl_fns
            #other_impl_fns
        };
        #[cfg(feature = "dyn_fns")]
        {
            let checked_struct_impl_fns = &self.checked_struct_impl_fns;
            let other_checked_struct_impl_fns = &other.checked_struct_impl_fns;
            self.checked_struct_impl_fns = quote! {
                #checked_struct_impl_fns
                #other_checked_struct_impl_fns
            };
            let bitfield_dyn_trait_impl_fns = &self.bitfield_dyn_trait_impl_fns;
            let other_bitfield_dyn_trait_impl_fns = &other.bitfield_dyn_trait_impl_fns;
            self.bitfield_dyn_trait_impl_fns = quote! {
                #bitfield_dyn_trait_impl_fns
                #other_bitfield_dyn_trait_impl_fns
            };
        }
    }
    fn append_bitfield_trait_impl_fns(&mut self, quote: TokenStream) {
        let old = &self.bitfield_trait_impl_fns;
        self.bitfield_trait_impl_fns = quote! {
            #old
            #quote
        };
    }
    fn append_impl_fns(&mut self, quote: TokenStream) {
        let old = &self.impl_fns;
        self.impl_fns = quote! {
            #old
            #quote
        };
    }
    #[cfg(feature = "dyn_fns")]
    fn append_checked_struct_impl_fns(&mut self, quote: TokenStream) {
        let old = &self.checked_struct_impl_fns;
        self.checked_struct_impl_fns = quote! {
            #old
            #quote
        };
    }
    #[cfg(feature = "dyn_fns")]
    fn append_bitfield_dyn_trait_impl_fns(&mut self, quote: TokenStream) {
        let old = &self.bitfield_dyn_trait_impl_fns;
        self.bitfield_dyn_trait_impl_fns = quote! {
            #old
            #quote
        };
    }
}

impl ObjectInfo {
    pub fn generate(&self) -> syn::Result<GeneratedFunctions> {
        match self {
            ObjectInfo::Struct(s) => s.generate_bitfield_functions(),
            ObjectInfo::Enum(e) => e.generate_bitfield_functions(),
        }
    }
}

/// This contains incomplete function generation. this should only be used by `StructInfo` or `EnumInfo` internally.
struct FieldQuotes {
    read_fns: GeneratedFunctions,
    write_fns: GeneratedFunctions,
    /// A list of field names to be used in initializing a new struct from bytes.
    field_list: TokenStream,
}

impl StructInfo {
    fn generate_bitfield_functions(&self) -> syn::Result<GeneratedFunctions> {
        // generate basic generated code for field access functions.
        let mut quotes = self.create_field_quotes(None)?;
        // Gather information to finish [`Bitfields::from_bytes`]
        let struct_size = &self.total_bytes();
        let from_bytes_quote = &quotes.read_fns.bitfield_trait_impl_fns;
        let fields_list = &quotes.field_list;
        // construct from bytes function. use input_byte_buffer as input name because,
        // that is what the field quotes expect to extract from.
        // wrap our list of field names with commas with Self{} so we it instantiate our struct,
        // because all of the from_bytes field quote store there data in a temporary variable with the same
        // name as its destination field the list of field names will be just fine.
        quotes.read_fns.bitfield_trait_impl_fns = quote! {
            fn from_bytes(mut input_byte_buffer: [u8;#struct_size]) -> Self {
                #from_bytes_quote
                Self{
                    #fields_list
                }
            }
        };
        #[cfg(feature = "dyn_fns")]
        {
            // do what we did for `Bitfields` impl for `BitfieldsDyn` impl
            let from_bytes_dyn_quote = &quotes.read_fns.bitfield_dyn_trait_impl_fns;
            let comment_take = "Creates a new instance of `Self` by copying field from the bitfields, removing bytes that where used. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
            let comment = "Creates a new instance of `Self` by copying field from the bitfields. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
            quotes.read_fns.bitfield_dyn_trait_impl_fns = quote! {
                #[doc = #comment_take]
                fn from_vec(input_byte_buffer: &mut Vec<u8>) -> Result<Self, bondrewd::BitfieldLengthError> {
                    if input_byte_buffer.len() < Self::BYTE_SIZE {
                        return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                    }
                    let out = {
                        #from_bytes_dyn_quote
                        Self {
                            #fields_list
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
                        #from_bytes_dyn_quote
                        Self {
                            #fields_list
                        }
                    };
                    Ok(out)
                }
            };
        }
        Ok(quotes.read_fns)
        // todo!("finish merged (from AND into) generate functions for StructInfo");
    }
    fn create_field_quotes(&self, enum_name: Option<&Ident>) -> syn::Result<FieldQuotes> {
        let variant_name = if enum_name.is_some() {
            // We what to use the name of the struct because enum variants are just StructInfos internally.
            Some(format_ident!(
                "{}",
                self.name.to_string().to_case(Case::Snake)
            ))
        } else {
            None
        };
        #[cfg(feature = "dyn_fns")]
        let impl_fns = if enum_name.is_some() {
            // TODO impl proper check_slice support for enums. (should just be get_check_slice_fn for each variant)
            quote! {}
        } else {
            get_check_slice_fn(&self.name, self.total_bytes())
        };
        let mut gen = GeneratedFunctions {
            #[cfg(feature = "dyn_fns")]
            impl_fns,
            ..Default::default()
        };
        // If we are building code for an enum variant that does not capture the id
        // then we should skip the id field to avoid creating an get_id function for each variant.
        let fields = if enum_name.is_some() && !self.fields[0].attrs.capture_id {
            &self.fields[1..]
        } else {
            &self.fields[..]
        };
        let mut field_name_list = quote! {};
        for field in fields {
            self.make_read_fns(field, &variant_name, &mut field_name_list, &mut gen)?;
        }
        Ok(FieldQuotes {
            read_fns: gen,
            write_fns: todo!("make writing side of new code gen"),
            // write_fns: GeneratedFunctions::default(),
            field_list: field_name_list,
        })
    }
    fn make_read_fns(
        &self,
        field: &FieldInfo,
        variant_name: &Option<Ident>,
        field_name_list: &mut TokenStream,
        gen: &mut GeneratedFunctions,
    ) -> syn::Result<()> {
        let field_name = field.ident().ident();
        let prefixed_name = if let Some(prefix) = variant_name {
            format_ident!("{prefix}_{field_name}")
        } else {
            format_ident!("{field_name}")
        };

        let mut impl_fns = quote! {};
        let mut checked_struct_impl_fns = quote! {};
        let field_access = field.get_quotes(self)?;
        let field_extractor = field_access.read();
        self.make_read_fns_inner(
            field,
            &prefixed_name,
            field_extractor,
            &mut impl_fns,
            &mut checked_struct_impl_fns,
        );
        gen.append_impl_fns(impl_fns);
        #[cfg(feature = "dyn_fns")]
        gen.append_checked_struct_impl_fns(checked_struct_impl_fns);

        // fake fields do not exist in the actual structure and should only have functions
        // that read or write values into byte arrays.
        if !field.attrs.reserve.is_fake_field() {
            // put the name of the field into the list of fields that are needed to create
            // the struct.
            *field_name_list = quote! {#field_name_list #field_name,};
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
                    let fn_field_name = format_ident!("read_{prefixed_name}");
                    quote! {
                        let #field_name = Self::#fn_field_name(&input_byte_buffer);
                    }
                } else {
                    quote! { let #field_name = Default::default(); }
                }
            };
            gen.append_bitfield_trait_impl_fns(peek_call);
            #[cfg(feature = "dyn_fns")]
            gen.append_bitfield_dyn_trait_impl_fns(quote! {
                let #field_name = #field_extractor;
            });
        }
        Ok(())
    }
    fn make_read_fns_inner(
        &self,
        field: &FieldInfo,
        peek_name: &Ident,
        field_extractor: &TokenStream,
        peek_quote: &mut TokenStream,
        peek_slice_fns_option: &mut TokenStream,
    ) {
        *peek_quote = generate_read_field_fn(field_extractor, field, self, &peek_name);
        // make the slice functions if applicable.
        #[cfg(feature = "dyn_fns")]
        {
            let peek_slice_quote =
                generate_read_slice_field_fn(field_extractor, field, self, &peek_name);
            *peek_quote = quote! {
                #peek_quote
                #peek_slice_quote
            };

            let peek_slice_unchecked_quote =
                generate_read_slice_field_fn_unchecked(field_extractor, field, self, &peek_name);
            *peek_slice_fns_option = quote! {
                #peek_slice_fns_option
                #peek_slice_unchecked_quote
            };
        }
    }
    fn make_write_fns(&self) {}
}

impl EnumInfo {
    fn generate_bitfield_functions(&self) -> syn::Result<GeneratedFunctions> {
        let enum_name: Option<&Ident> = Some(&self.name);
        let mut gen = GeneratedFunctions {
            impl_fns: {
                let field = self.generate_id_field()?;
                let flip = false;
                let access = field.get_quotes_no_flip()?;
                let attrs = self.attrs.attrs.clone();
                let mut fields = vec![field.clone()];
                fields[0].attrs.bit_range = 0..self.total_bits();
                let temp_struct_info = StructInfo {
                    name: self.name.clone(),
                    attrs,
                    fields,
                    vis: syn::Visibility::Public(Pub::default()),
                    tuple: false,
                };
                let field_name = &field.ident().ident();
                let id_field_read =
                    generate_read_field_fn(access.read(), &field, &temp_struct_info, field_name);
                let id_field_write =
                    generate_write_field_fn(access.write(), access.zero(), &field, &temp_struct_info, field_name);
                let output = quote! {
                    #id_field_read
                    #id_field_write
                };
                #[cfg(feature = "dyn_fns")]
                {
                    let check_slice_fn = get_check_slice_fn(&self.name, self.total_bytes());
                    let check_slice_mut_fn = get_check_slice_mut_fn(&self.name, self.total_bytes());
                    let id_slice_read = generate_read_slice_field_fn(
                        access.read(),
                        &field,
                        &temp_struct_info,
                        field_name,
                    );
                    // START_HERE i just added `id_slice_write` to the output.
                    let id_slice_write = generate_write_slice_field_fn(access.write(), access.zero(), &field, &temp_struct_info, field_name);
                    quote!{
                        #output
                        #id_slice_read
                        #id_slice_write
                        #check_slice_fn
                        #check_slice_mut_fn
                    }
                }
                #[cfg(not(feature = "dyn_fns"))]
                {
                    output
                }
            },
            ..Default::default()
        };
        let struct_size = self.total_bytes();
        let last_variant = self.variants.len() - 1;

        for (i, variant) in self.variants.iter().enumerate() {
            // this is the slice indexing that will fool the set function code into thinking
            // it is looking at a smaller array.
            let v_name = &variant.name;
            let upper_v_name = v_name.to_string().to_case(Case::UpperSnake);
            let v_byte_const_name = format_ident!("{upper_v_name}_BYTE_SIZE");
            let v_bit_const_name = format_ident!("{upper_v_name}_BIT_SIZE");
            let v_byte_size = variant.total_bytes();
            let v_bit_size = variant.total_bits();
            let variant_name = quote! {#v_name};
            #[cfg(feature = "dyn_fns")]
            let stuff: (
                TokenStream,
                TokenStream,
                TokenStream,
                TokenStream,
                TokenStream,
            );
            #[cfg(not(feature = "dyn_fns"))]
            let stuff: (TokenStream, TokenStream, TokenStream);
            stuff = {
                let thing = variant.create_field_quotes(enum_name)?;
                (
                    thing.field_list,
                    thing.read_fns.impl_fns,
                    thing.read_fns.bitfield_trait_impl_fns,
                    #[cfg(feature = "dyn_fns")]
                    thing.read_fns.checked_struct_impl_fns,
                    #[cfg(feature = "dyn_fns")]
                    thing.read_fns.bitfield_dyn_trait_impl_fns,
                )
            };
            #[cfg(feature = "dyn_fns")]
            let (
                field_name_list,
                peek_fns_quote_temp,
                from_bytes_quote,
                peek_slice_field_unchecked_fns,
                from_vec_quote,
            ) = (stuff.0, stuff.1, stuff.2, stuff.3, stuff.4);
            #[cfg(not(feature = "dyn_fns"))]
            let (field_name_list, peek_fns_quote_temp, from_bytes_quote) =
                (stuff.0, stuff.1, stuff.2);
            #[cfg(feature = "dyn_fns")]
            gen.append_checked_struct_impl_fns(peek_slice_field_unchecked_fns);
            gen.append_impl_fns(peek_fns_quote_temp);
            gen.append_impl_fns(quote! {
                pub const #v_byte_const_name: usize = #v_byte_size;
                pub const #v_bit_const_name: usize = #v_bit_size;
            });
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
            gen.append_bitfield_trait_impl_fns(quote! {
                #variant_id => {
                    #from_bytes_quote
                    #variant_constructor
                }
            });
            #[cfg(feature = "dyn_fns")]
            {
                gen.append_bitfield_dyn_trait_impl_fns(quote! {
                    #variant_id => {
                        #from_vec_quote
                        #variant_constructor
                    }
                });
            }
        }

        let v_id = format_ident!("{}", EnumInfo::VARIANT_ID_NAME);
        let v_id_call = format_ident!("read_{v_id}");
        #[cfg(feature = "dyn_fns")]
        let v_id_slice_call = format_ident!("read_slice_{v_id}");
        let from_bytes_fn = &gen.bitfield_trait_impl_fns;
        gen.bitfield_trait_impl_fns = quote! {
            fn from_bytes(mut input_byte_buffer: [u8;#struct_size]) -> Self {
                let #v_id = Self::#v_id_call(&input_byte_buffer);
                match #v_id {
                    #from_bytes_fn
                }
            }
        };
        #[cfg(feature = "dyn_fns")]
        {
            let from_vec_fn = &gen.bitfield_dyn_trait_impl_fns;
            let comment_take = "Creates a new instance of `Self` by copying field from the bitfields, removing bytes that where used. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
            let comment = "Creates a new instance of `Self` by copying field from the bitfields. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
            gen.bitfield_dyn_trait_impl_fns = quote! {
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
        }
        Ok(gen)
    }
}

#[cfg(feature = "dyn_fns")]
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
fn get_check_slice_mut_fn(name: &Ident, struct_size: usize) -> TokenStream {
    // all quote with all of the set slice functions appended to it.
    let checked_ident = format_ident!("{name}CheckedMut");
    let comment = format!("Returns a [{checked_ident}] which allows you to read/write any field for a `{name}` from/to provided mutable slice.");
    quote! {
        #[doc = #comment]
        pub fn check_slice_mut(buffer: &mut [u8]) -> Result<#checked_ident, bondrewd::BitfieldLengthError> {
            let buf_len = buffer.len();
            if buf_len >= #struct_size {
                Ok(#checked_ident {
                    buffer
                })
            }else{
                Err(bondrewd::BitfieldLengthError(buf_len, #struct_size))
            }
        }
    }
}
/// Generates a `read_field_name()` function.
fn generate_read_field_fn(
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
fn generate_read_slice_field_fn(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    field_name: &Ident,
) -> TokenStream {
    let fn_field_name = format_ident!("read_slice_{field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let min_length = if info.attrs.flip {
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
fn generate_read_slice_field_fn_unchecked(
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
fn generate_write_field_fn(
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
fn generate_write_slice_field_fn(
    field_quote: &TokenStream,
    clear_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    field_name: &Ident,
) -> TokenStream {
    let fn_field_name = format_ident!("write_slice_{field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let min_length = if info.attrs.flip {
        (info.total_bits() - field.attrs.bit_range.start).div_ceil(8)
    } else {
        field.attrs.bit_range.end.div_ceil(8)
    };
    let comment = format!("Writes to bits {} through {} in `input_byte_buffer` if enough bytes are present in slice, setting the `{field_name}` field of a `{struct_name}` in bitfield form. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned", bit_range.start, bit_range.end - 1);
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