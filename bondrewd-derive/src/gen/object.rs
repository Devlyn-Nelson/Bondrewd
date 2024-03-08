use std::str::FromStr;

use crate::structs::common::{EnumInfo, FieldInfo, ObjectInfo, StructInfo};
#[cfg(feature = "setters")]
use crate::structs::struct_fns;
use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::token::Pub;

#[derive(Clone)]
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
    pub fn generate(&self) -> syn::Result<TokenStream> {
        let gen = match self {
            ObjectInfo::Struct(s) => Ok(s.generate_bitfield_functions()?.finish()),
            ObjectInfo::Enum(e) => e.generate_bitfield_functions(),
        }?;
        // get the struct size and name so we can use them in a quote.
        let struct_size = self.total_bytes();
        let struct_name = self.name();
        let impl_fns = gen.impl_fns;
        let mut output = match self {
            #[cfg(not(feature = "setters"))]
            ObjectInfo::Struct(ref _struct_info) => {
                quote! {
                    impl #struct_name {
                        #impl_fns
                    }
                }
            }
            #[cfg(feature = "setters")]
            ObjectInfo::Struct(ref struct_info) => {
                // TODO get setter for arrays working.
                // get the setters, functions that set a field disallowing numbers
                // outside of the range the Bitfield.
                let setters_quote = match struct_fns::create_setters_quotes(&struct_info) {
                    Ok(parsed_struct) => parsed_struct,
                    Err(err) => {
                        return Err(err);
                    }
                };
                quote! {
                    impl #struct_name {
                        #impl_fns
                        #setters_quote
                    }
                }
            }
            ObjectInfo::Enum(ref _enum_info) => {
                // TODO implement getters and setters for enums.
                quote! {
                    impl #struct_name {
                        #impl_fns
                    }
                }
            }
        };
        // get the bit size of the entire set of fields to fill in trait requirement.
        let bit_size = self.total_bits();
        let trait_impl_fn = gen.bitfield_trait_impl_fns;
        output = quote! {
            #output
            impl bondrewd::Bitfields<#struct_size> for #struct_name {
                const BIT_SIZE: usize = #bit_size;
                #trait_impl_fn
            }
        };
        #[cfg(feature = "hex_fns")]
        {
            let hex_size = struct_size * 2;
            output = quote! {
                #output
                impl bondrewd::BitfieldHex<#hex_size, #struct_size> for #struct_name {}
            };
            #[cfg(feature = "dyn_fns")]
            {
                output = quote! {
                    #output
                    impl bondrewd::BitfieldHexDyn<#hex_size, #struct_size> for #struct_name {}
                };
            }
        }
        #[cfg(feature = "dyn_fns")]
        {
            let checked_structs = gen.checked_struct_impl_fns;
            let from_vec_quote = gen.bitfield_dyn_trait_impl_fns;
            output = quote! {
                #output
                #checked_structs
                impl bondrewd::BitfieldsDyn<#struct_size> for #struct_name {
                    #from_vec_quote
                }
            }
        }
        Ok(output)
    }
}

#[cfg(feature = "dyn_fns")]
pub struct SliceInfo {
    check_slice_fn_name: Ident,
    check_mut_slice_fn_name: Ident,
    check_slice_struct_name: Ident,
    check_mut_slice_struct_name: Ident,
}

// TODO nothing about field quotes should be public
/// This contains incomplete function generation. this should only be used by `StructInfo` or `EnumInfo` internally.
pub struct FieldQuotes {
    pub read_fns: GeneratedFunctions,
    pub write_fns: GeneratedFunctions,
    /// A list of field names to be used in initializing a new struct from bytes.
    field_list: TokenStream,
    #[cfg(feature = "dyn_fns")]
    slice_info: Option<SliceInfo>,
}
impl FieldQuotes {
    pub fn finish(self) -> GeneratedFunctions {
        let mut read = self.read_fns;
        let write = self.write_fns;
        read.merge(write);
        read
    }
}

impl StructInfo {
    // TODO this should not be public
    pub fn generate_bitfield_functions(&self) -> syn::Result<FieldQuotes> {
        // generate basic generated code for field access functions.
        let mut quotes = self.create_field_quotes(None)?;
        // Gather information to finish [`Bitfields::from_bytes`]
        let struct_size = self.total_bytes();
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
        let into_bytes_quote = &quotes.write_fns.bitfield_trait_impl_fns;
        quotes.write_fns.bitfield_trait_impl_fns = quote! {
            fn into_bytes(self) -> [u8;#struct_size] {
                let mut output_byte_buffer: [u8;#struct_size] = [0u8;#struct_size];
                #into_bytes_quote
                output_byte_buffer
            }
        };
        Ok(quotes)
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
        let mut gen_read = GeneratedFunctions::default();
        let mut gen_write = GeneratedFunctions::default();
        // If we are building code for an enum variant that does not capture the id
        // then we should skip the id field to avoid creating an get_id function for each variant.
        let fields = if enum_name.is_some() && !self.fields[0].attrs.capture_id {
            &self.fields[1..]
        } else {
            &self.fields[..]
        };
        let mut field_name_list = quote! {};
        for field in fields {
            let field_access = field.get_quotes(self)?;
            self.make_read_fns(
                field,
                &variant_name,
                &mut field_name_list,
                &mut gen_read,
                &field_access,
            )?;
            self.make_write_fns(field, &variant_name, &mut gen_write, &field_access);
        }
        // Do checked struct of this type
        #[cfg(feature = "dyn_fns")]
        let checked = if !fields.is_empty() {
            let struct_name = if let Some(e_name) = enum_name {
                quote::format_ident!("{e_name}{}", &self.name)
            } else {
                self.name.clone()
            };
            let vis = self.vis();
            let checked_ident = quote::format_ident!("{struct_name}Checked");
            let checked_mut_ident = quote::format_ident!("{struct_name}CheckedMut");
            let unchecked_functions = gen_read.checked_struct_impl_fns;
            let unchecked_mut_functions = gen_write.checked_struct_impl_fns;
            let comment = format!("A Structure which provides functions for getting the fields of a [{struct_name}] in its bitfield form.");
            let comment_mut = format!("A Structure which provides functions for getting and setting the fields of a [{struct_name}] in its bitfield form.");
            let unchecked_comment = format!("Panics if resulting `{checked_ident}` does not contain enough bytes to read a field that is attempted to be read.");
            let unchecked_comment_mut = format!("Panics if resulting `{checked_mut_ident}` does not contain enough bytes to read a field that is attempted to be read or written.");
            gen_write.checked_struct_impl_fns = quote! {
                #[doc = #comment_mut]
                #vis struct #checked_mut_ident<'a> {
                    buffer: &'a mut [u8],
                }
                impl<'a> #checked_mut_ident<'a> {
                    #unchecked_functions
                    #unchecked_mut_functions
                    #[doc = #unchecked_comment_mut]
                    pub fn from_unchecked_slice(data: &'a mut [u8]) -> Self {
                        Self{
                            buffer: data
                        }
                    }
                }
            };
            gen_read.checked_struct_impl_fns = quote! {
                #[doc = #comment]
                #vis struct #checked_ident<'a> {
                    buffer: &'a [u8],
                }
                impl<'a> #checked_ident<'a> {
                    #unchecked_functions
                    #[doc = #unchecked_comment]
                    pub fn from_unchecked_slice(data: &'a [u8]) -> Self {
                        Self{
                            buffer: data
                        }
                    }
                }
            };
            let ename = if enum_name.is_some() {
                Some(&struct_name)
            } else {
                Some(&self.name)
            };

            let check_slice_info = CheckedSliceGen::new(&self.name, self.total_bytes(), ename);

            let check_slice_fn = check_slice_info.fn_gen;
            let impl_fns = gen_read.impl_fns;
            gen_read.impl_fns = quote! {
                #impl_fns
                #check_slice_fn
            };

            let check_slice_mut_fn = check_slice_info.mut_fn_gen;
            let impl_fns = gen_write.impl_fns;
            gen_write.impl_fns = quote! {
                #impl_fns
                #check_slice_mut_fn
            };
            Some(SliceInfo {
                check_slice_fn_name: check_slice_info.fn_name,
                check_mut_slice_fn_name: check_slice_info.mut_fn_name,
                check_slice_struct_name: checked_ident,
                check_mut_slice_struct_name: checked_mut_ident,
            })
        } else {
            None
        };
        Ok(FieldQuotes {
            read_fns: gen_read,
            write_fns: gen_write,
            field_list: field_name_list,
            #[cfg(feature = "dyn_fns")]
            slice_info: checked,
        })
    }
    fn make_read_fns(
        &self,
        field: &FieldInfo,
        variant_name: &Option<Ident>,
        field_name_list: &mut TokenStream,
        gen: &mut GeneratedFunctions,
        field_access: &super::field::FieldQuotes,
    ) -> syn::Result<()> {
        let field_name = field.ident().ident();
        let prefixed_name = if let Some(prefix) = variant_name {
            format_ident!("{prefix}_{field_name}")
        } else {
            format_ident!("{field_name}")
        };

        let mut impl_fns = quote! {};
        #[cfg(feature = "dyn_fns")]
        let mut checked_struct_impl_fns = quote! {};
        let field_extractor = field_access.read();
        self.make_read_fns_inner(
            field,
            &prefixed_name,
            field_extractor,
            &mut impl_fns,
            #[cfg(feature = "dyn_fns")]
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
        field_name: &Ident,
        field_extractor: &TokenStream,
        peek_quote: &mut TokenStream,
        #[cfg(feature = "dyn_fns")] peek_slice_fns_option: &mut TokenStream,
    ) {
        *peek_quote = generate_read_field_fn(field_extractor, field, self, &field_name);
        // make the slice functions if applicable.
        #[cfg(feature = "dyn_fns")]
        {
            let peek_slice_quote =
                generate_read_slice_field_fn(field_extractor, field, self, &field_name);
            *peek_quote = quote! {
                #peek_quote
                #peek_slice_quote
            };

            let peek_slice_unchecked_quote =
                generate_read_slice_field_fn_unchecked(field_extractor, field, self, &field_name);
            *peek_slice_fns_option = quote! {
                #peek_slice_fns_option
                #peek_slice_unchecked_quote
            };
        }
    }
    fn make_write_fns(
        &self,
        field: &FieldInfo,
        variant_name: &Option<Ident>,
        gen: &mut GeneratedFunctions,
        field_access: &super::field::FieldQuotes,
    ) {
        let field_name = field.ident().ident();
        let prefixed_name = if let Some(prefix) = variant_name {
            format_ident!("{prefix}_{field_name}")
        } else {
            format_ident!("{field_name}")
        };
        if field.attrs.reserve.is_fake_field() || field.attrs.capture_id {
            return;
        }
        let (field_setter, clear_quote) = (field_access.write(), field_access.zero());
        if field.attrs.reserve.write_field() {
            if variant_name.is_some() {
                let fn_name = format_ident!("write_{prefixed_name}");
                gen.append_bitfield_trait_impl_fns(quote! {
                    Self::#fn_name(&mut output_byte_buffer, #field_name);
                });
            } else {
                gen.append_bitfield_trait_impl_fns(quote! {
                    let #field_name = self.#field_name;
                    #field_setter
                });
            }
        }

        let mut impl_fns = quote! {};
        #[cfg(feature = "dyn_fns")]
        let mut checked_struct_impl_fns = quote! {};
        self.make_write_fns_inner(
            field,
            &prefixed_name,
            field_setter,
            clear_quote,
            &mut impl_fns,
            #[cfg(feature = "dyn_fns")]
            &mut checked_struct_impl_fns,
        );

        gen.append_impl_fns(impl_fns);
        #[cfg(feature = "dyn_fns")]
        gen.append_checked_struct_impl_fns(checked_struct_impl_fns);
    }
    fn make_write_fns_inner(
        &self,
        field: &FieldInfo,
        field_name: &Ident,
        field_setter: &TokenStream,
        clear_quote: &TokenStream,
        write_quote: &mut TokenStream,
        #[cfg(feature = "dyn_fns")] write_slice_fns_option: &mut TokenStream,
    ) {
        *write_quote =
            generate_write_field_fn(&field_setter, &clear_quote, field, self, field_name);
        #[cfg(feature = "dyn_fns")]
        {
            let set_slice_quote =
                generate_write_slice_field_fn(&field_setter, &clear_quote, field, self, field_name);
            *write_quote = quote! {
                #write_quote
                #set_slice_quote
            };
            let set_slice_unchecked_quote = generate_write_slice_field_fn_unchecked(
                field_setter,
                clear_quote,
                field,
                self,
                field_name,
            );
            *write_slice_fns_option = quote! {
                #write_slice_fns_option
                #set_slice_unchecked_quote
            };
        }
    }
}

impl EnumInfo {
    pub fn generate_bitfield_functions(&self) -> syn::Result<GeneratedFunctions> {
        let enum_name: Option<&Ident> = Some(&self.name);
        let mut gen = GeneratedFunctions {
            impl_fns: {
                let field = self.generate_id_field()?;
                let mut attrs = self.attrs.attrs.clone();
                // TODO Still don't know if flipping should be ignored.
                attrs.flip = false;
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
                let access = field.get_quotes(&temp_struct_info)?;
                let id_field_read =
                    generate_read_field_fn(access.read(), &field, &temp_struct_info, field_name);
                let id_field_write = generate_write_field_fn(
                    access.write(),
                    access.zero(),
                    &field,
                    &temp_struct_info,
                    field_name,
                );
                let output = quote! {
                    #id_field_read
                    #id_field_write
                };
                #[cfg(feature = "dyn_fns")]
                {
                    let id_slice_read = generate_read_slice_field_fn(
                        access.read(),
                        &field,
                        &temp_struct_info,
                        field_name,
                    );
                    let id_slice_write = generate_write_slice_field_fn(
                        access.write(),
                        access.zero(),
                        &field,
                        &temp_struct_info,
                        field_name,
                    );
                    quote! {
                        #output
                        #id_slice_read
                        #id_slice_write
                    }
                }
                #[cfg(not(feature = "dyn_fns"))]
                {
                    output
                }
            },
            ..Default::default()
        }; //
        let struct_size = self.total_bytes();
        let last_variant = self.variants.len() - 1;
        // stores all of the into/from bytes functions across variants.
        let mut into_bytes_fn: TokenStream = quote! {};
        let mut from_bytes_fn: TokenStream = quote! {};
        // stores the build up for the id function.
        let mut id_fn: TokenStream = quote! {};
        // stores the build up for the `check_slice` fn for an enum.
        #[cfg(feature = "dyn_fns")]
        let (mut check_slice_fn, checked_ident): (TokenStream, Ident) =
            (quote! {}, format_ident!("{}Checked", &self.name));
        // stores the build up for the `check_slice_mut` fn for an enum.
        #[cfg(feature = "dyn_fns")]
        let (mut check_slice_mut_fn, checked_ident_mut): (TokenStream, Ident) =
            (quote! {}, format_ident!("{}CheckedMut", &self.name));
        // Stores a build up for creating a match enum type that contains CheckStruct for each variant.
        #[cfg(feature = "dyn_fns")]
        let (mut checked_slice_enum, mut checked_slice_enum_mut): (TokenStream, TokenStream) = (quote! {}, quote! {});
        // the string `variant_id` as an Ident
        let v_id = format_ident!("{}", EnumInfo::VARIANT_ID_NAME);
        // setup function names for getting variant id.
        let v_id_read_call = format_ident!("read_{v_id}");
        let v_id_write_call = format_ident!("write_{v_id}");
        #[cfg(feature = "dyn_fns")]
        let v_id_read_slice_call = format_ident!("read_slice_{v_id}");
        for (i, variant) in self.variants.iter().enumerate() {
            // this is the slice indexing that will fool the set function code into thinking
            // it is looking at a smaller array.
            //
            // v_name is the name of the variant.
            let v_name = &variant.name;
            // upper_v_name is an Screaming Snake Case of v_name.
            let upper_v_name = v_name.to_string().to_case(Case::UpperSnake);
            // constant names for variant bit and byte sizings.
            let v_byte_const_name = format_ident!("{upper_v_name}_BYTE_SIZE");
            let v_bit_const_name = format_ident!("{upper_v_name}_BIT_SIZE");
            // constant values for variant bit and byte sizings.
            let v_byte_size = variant.total_bytes();
            let v_bit_size = variant.total_bits();
            // TokenStream of v_name.
            let variant_name = quote! {#v_name};
            // #[cfg(feature = "dyn_fns")]
            // let stuff: (
            //     TokenStream,
            //     TokenStream,
            //     TokenStream,
            //     TokenStream,
            //     TokenStream,
            // );
            // #[cfg(not(feature = "dyn_fns"))]
            // let stuff: (TokenStream, TokenStream, TokenStream);
            // stuff = {
            //     let thing = variant.create_field_quotes(enum_name)?;
            //     (
            //         thing.field_list,
            //         thing.read_fns.impl_fns,
            //         thing.read_fns.bitfield_trait_impl_fns,
            //         #[cfg(feature = "dyn_fns")]
            //         thing.read_fns.checked_struct_impl_fns,
            //         #[cfg(feature = "dyn_fns")]
            //         thing.read_fns.bitfield_dyn_trait_impl_fns,
            //     )
            // };
            // #[cfg(feature = "dyn_fns")]
            // let (
            //     field_name_list,
            //     peek_fns_quote_temp,
            //     from_bytes_quote,
            //     peek_slice_field_unchecked_fns,
            //     from_vec_quote,
            // ) = (stuff.0, stuff.1, stuff.2, stuff.3, stuff.4);
            // #[cfg(not(feature = "dyn_fns"))]
            // let (field_name_list, peek_fns_quote_temp, from_bytes_quote) =
            //     (stuff.0, stuff.1, stuff.2);

            let thing = variant.create_field_quotes(enum_name)?;
            #[cfg(feature = "dyn_fns")]
            {
                gen.append_checked_struct_impl_fns(thing.read_fns.checked_struct_impl_fns);
                gen.append_checked_struct_impl_fns(thing.write_fns.checked_struct_impl_fns);
            }
            gen.append_impl_fns(thing.read_fns.impl_fns);
            gen.append_impl_fns(thing.write_fns.impl_fns);
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
            let mut variant_value = variant.id_or_field_name()?;
            let variant_constructor = if thing.field_list.is_empty() {
                quote! {Self::#variant_name}
            } else if variant.tuple {
                let field_name_list = thing.field_list;
                quote! {Self::#variant_name ( #field_name_list )}
            } else {
                let field_name_list = thing.field_list;
                quote! {Self::#variant_name { #field_name_list }}
            };
            // From Bytes
            let from_bytes_quote = &thing.read_fns.bitfield_trait_impl_fns;
            from_bytes_fn = quote! {
                #from_bytes_fn
                #variant_id => {
                    #from_bytes_quote
                    #variant_constructor
                }
            };
            #[cfg(feature = "dyn_fns")]
            {
                let bitfield_dyn_trait_impl_fns = &gen.bitfield_dyn_trait_impl_fns;
                let from_vec_quote = &thing.read_fns.bitfield_dyn_trait_impl_fns;
                gen.bitfield_dyn_trait_impl_fns = quote! {
                    #bitfield_dyn_trait_impl_fns
                    #variant_id => {
                        #from_vec_quote
                        #variant_constructor
                    }
                };
                // Check Slice
                if let Some(slice_info) = thing.slice_info {
                    // do the match statement stuff
                    let check_slice_name = format_ident!("check_slice_{}", slice_info.check_slice_fn_name);
                    let check_slice_struct = &slice_info.check_slice_struct_name;
                    check_slice_fn = quote! {
                        #check_slice_fn
                        #variant_id => {
                            #checked_ident :: #variant_name (Self::#check_slice_name(buffer))
                        }
                    };
                    let check_slice_name_mut = format_ident!("check_slice_mut_{}", slice_info.check_mut_slice_fn_name);
                    let check_slice_struct_mut = &slice_info.check_mut_slice_struct_name;
                    check_slice_mut_fn = quote! {
                        #check_slice_mut_fn
                        #variant_id => {
                            #checked_ident_mut :: #variant_name (Self::#check_slice_name_mut(buffer))
                        }
                    };
                    // do enum stuff
                    checked_slice_enum = quote!{
                        #checked_slice_enum
                        #checked_ident (#check_slice_struct),
                    };
                    checked_slice_enum_mut = quote!{
                        #checked_slice_enum_mut
                        #checked_ident_mut (#check_slice_struct_mut),
                    };
                }else{
                    // do the match statement stuff
                    check_slice_fn = quote! {
                        #check_slice_fn
                        #variant_id => {
                            #checked_ident :: #variant_name
                        }
                    };
                    check_slice_mut_fn = quote! {
                        #check_slice_mut_fn
                        #variant_id => {
                            #checked_ident_mut :: #variant_name
                        }
                    };
                    // do enum stuff
                    checked_slice_enum = quote!{
                        #checked_slice_enum
                        #checked_ident,
                    };
                    checked_slice_enum_mut = quote!{
                        #checked_slice_enum_mut
                        #checked_ident_mut,
                    };
                }
            }
            // Into Bytes
            let into_bytes_quote = &thing.write_fns.bitfield_trait_impl_fns;
            into_bytes_fn = quote! {
                #into_bytes_fn
                #variant_constructor => {
                    Self::#v_id_write_call(&mut output_byte_buffer, #variant_value);
                    #into_bytes_quote
                }
            };
            // Variant Id fn
            if !variant.fields.is_empty() && variant.fields[0].attrs.capture_id {
                let id_field_name = &variant.fields[0].ident().name();
                variant_value = quote! {#id_field_name};
            }

            let mut ignore_fields = if variant.fields[0].attrs.capture_id {
                let id_field_name = &variant.fields[0].ident().name();
                variant_value = quote! {*#variant_value};
                quote! { #id_field_name, }
            } else {
                quote! {}
            };
            if variant.fields.len() > 1 {
                ignore_fields = quote! { #ignore_fields .. };
            } else {
                ignore_fields = quote! { #ignore_fields };
            };
            if variant.tuple {
                ignore_fields = quote! {(#ignore_fields)};
            } else {
                ignore_fields = quote! {{#ignore_fields}};
            }
            id_fn = quote! {
                #id_fn
                Self::#variant_name #ignore_fields => #variant_value,
            };
        }
        // Finish `from_bytes` function.
        from_bytes_fn = quote! {
            fn from_bytes(mut input_byte_buffer: [u8;#struct_size]) -> Self {
                let #v_id = Self::#v_id_read_call(&input_byte_buffer);
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
                    let #v_id = Self::#v_id_read_slice_call(&input_byte_buffer)?;
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
                    let #v_id = Self::#v_id_read_slice_call(&input_byte_buffer)?;
                    let out = match #v_id {
                        #from_vec_fn
                    };
                    Ok(out)
                }
            };
            let comment = format!(
                "Returns a checked structure which allows you to read any field for a `{}` from provided slice.",
                &self.name
            );
            gen.append_impl_fns(quote! {
                #[doc = #comment]
                pub fn check_slice(buffer: &[u8]) -> Result<#checked_ident, bondrewd::BitfieldLengthError> {
                    let #v_id = Self::#v_id_read_slice_call(&buffer)?;
                    match #v_id {
                        #check_slice_fn
                    }
                }
            });
            let comment = format!(
                "Returns a checked mutable structure which allows you to read/write any field for a `{}` from provided mut slice.",
                &self.name
            );
            gen.append_impl_fns(quote! {
                #[doc = #comment]
                pub fn check_slice_mut(buffer: &mut [u8]) -> Result<#checked_ident_mut, bondrewd::BitfieldLengthError> {
                    let #v_id = Self::#v_id_read_slice_call(&buffer)?;
                    match #v_id {
                        #check_slice_mut_fn
                    }
                }
            });
            gen.append_checked_struct_impl_fns(quote!{
                pub enum #checked_ident {
                    #checked_slice_enum
                }
                pub enum #checked_ident_mut {
                    #checked_slice_enum_mut
                }
            });
        }
        // Finish `into_bytes` function.
        into_bytes_fn = quote! {
            fn into_bytes(self) -> [u8;#struct_size] {
                let mut output_byte_buffer = [0u8;#struct_size];
                match self {
                    #into_bytes_fn
                }
                output_byte_buffer
            }
        };
        // Finish Variant Id function.
        let id_ident = self.id_type_ident()?;
        gen.append_impl_fns(quote! {
            pub fn id(&self) -> #id_ident {
                match self {
                    #id_fn
                }
            }
        });

        gen.bitfield_trait_impl_fns = quote! {
            #from_bytes_fn
            #into_bytes_fn
        };


        Ok(gen)
    }
}

#[cfg(feature = "dyn_fns")]
/// generates the check_slice fn. please do not use, use `CheckedSliceGen`.
/// returns (fn, fn_name).
///
/// `name` is the name of the structure or variant
/// `check_size` is the total byte size of the struct or variant
/// `enum_name` if we are generating code for a variant (not a structure) then a
///     Some value containing the prefixed name shall be provided.
///     ex. enum and variant -> Test::One = "test_one" <- prefixed name
fn get_check_mut_slice_fn(
    name: &Ident,
    // total_bytes
    check_size: usize,
    enum_name: Option<&Ident>,
) -> (TokenStream, Ident) {
    let (checked_ident_mut, fn_name) = if let Some(ename) = enum_name {
        (
            format_ident!("{ename}Checked"),
            format_ident!("check_slice_mut_{}", name.to_string().to_case(Case::Snake)),
        )
    } else {
        (
            format_ident!("{name}Checked"),
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
/// generates the check_slice fn. please do not use, use `CheckedSliceGen`.
/// returns (fn, fn_name).
///
/// `name` is the name of the structure or variant
/// `check_size` is the total byte size of the struct or variant
/// `enum_name` if we are generating code for a variant (not a structure) then a
///     Some value containing the prefixed name shall be provided.
///     ex. enum and variant -> Test::One = "test_one" <- prefixed name
fn get_check_slice_fn(
    name: &Ident,
    // total_bytes
    check_size: usize,
    enum_name: Option<&Ident>,
) -> (TokenStream, Ident) {
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
struct CheckedSliceGen {
    fn_gen: TokenStream,
    mut_fn_gen: TokenStream,
    fn_name: Ident,
    mut_fn_name: Ident,
}
#[cfg(feature = "dyn_fns")]
impl CheckedSliceGen {
    fn new(
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
    prefixed_field_name: &Ident,
) -> TokenStream {
    let field_name = field.ident().name();
    let fn_field_name = format_ident!("write_slice_{prefixed_field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let min_length = if info.attrs.flip {
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
fn generate_write_slice_field_fn_unchecked(
    field_quote: &TokenStream,
    clear_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    prefixed_field_name: &Ident,
) -> TokenStream {
    let field_name = field.ident().name();
    let fn_field_name = format_ident!("write_{prefixed_field_name}");
    let bit_range = &field.attrs.bit_range;
    let type_ident = field.ty.type_quote();
    let struct_name = &info.name;
    let comment_bits = if bit_range.end - bit_range.start > 1 {
        format!("bits {} through {}", bit_range.start, bit_range.end - 1)
    } else {
        format!("bit {}", bit_range.start)
    };
    let comment = format!(
        "Writes to {comment_bits} in pre-checked mutable slice, setting the `{prefixed_field_name}` field of a [{struct_name}] in bitfield form.",
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
