use std::str::FromStr;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

#[cfg(feature = "dyn_fns")]
use super::field::{generate_read_slice_field_fn, generate_write_slice_field_fn};
use super::{
    field::{generate_read_field_fn, generate_write_field_fn},
    GeneratedFunctions,
};
use crate::common::{r#enum::Info as EnumInfo, r#struct::Info as StructInfo};

impl EnumInfo {
    pub fn get_temp_struct_for_id_gen(&self) -> syn::Result<StructInfo> {
        let invalids: Vec<&StructInfo> = self
            .variants
            .iter()
            .filter(|variant| variant.attrs.invalid)
            .collect();
        let mut variant = match invalids.len().cmp(&1) {
            std::cmp::Ordering::Less => {
                let v = self.variants.first();
                if let Some(variant) = v {
                    variant.clone()
                }else{
                    return Err(syn::Error::new(self.name.span(), "No Variants, this is a bug of the Bondrewd crate because it was not detected until the code generation step, please report issue. try adding a variant to your structure or not deriving `Bitfields` until you have one."));
                }
            }
            std::cmp::Ordering::Equal => {
                invalids[0].clone()
            }
            std::cmp::Ordering::Greater => return Err(syn::Error::new(self.name.span(), "2 invalid variants exist, this is a bug of the Bondrewd crate because it was not detected until the code generation step, please report issue. if using `#[bondrewd(invalid)]` on a variant please remove it, if using 2 remove at least 1.")),
        };
        if let Some(id_field) = variant.get_id_field_mut()? {
            id_field.ident = Box::new(format_ident!("{}", EnumInfo::VARIANT_ID_NAME).into());
        } else {
            return Err(syn::Error::new(self.name.span(), "First Variant had no fields, this is a bug of the Bondrewd crate please report issue. try adding a variant with an unsigned number that can contain the id_bit_length at the first field position with `#[bondrewd(capture_id)]`"));
        }

        Ok(variant)
    }

    pub fn generate_bitfield_functions(&self) -> syn::Result<GeneratedFunctions> {
        let enum_name: Option<&Ident> = Some(&self.name);
        let mut gen = GeneratedFunctions {
            non_trait: {
                let temp_struct_info = self.get_temp_struct_for_id_gen()?;
                // println!("enum - {temp_struct_info:?}");
                let field = if let Some(id_field) = temp_struct_info.get_id_field()? {
                    id_field.clone()
                } else {
                    return Err(syn::Error::new(self.name.span(), "fake Variant had id fields, this is a bug of the Bondrewd crate please report issue."));
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
        let (mut checked_slice_enum, mut checked_slice_enum_mut, mut lifetime): (
            TokenStream,
            TokenStream,
            bool,
        ) = (quote! {}, quote! {}, false);
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
            let v_bit_size = variant.total_bits_no_fill();
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
                gen.append_checked_struct_impl_fns(&thing.read_fns.checked_struct);
                gen.append_checked_struct_impl_fns(&thing.write_fns.checked_struct);
            }
            gen.append_impl_fns(&thing.read_fns.non_trait);
            gen.append_impl_fns(&thing.write_fns.non_trait);
            gen.append_impl_fns(&quote! {
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
            let from_bytes_quote = &thing.read_fns.bitfield_trait;
            from_bytes_fn = quote! {
                #from_bytes_fn
                #variant_id => {
                    #from_bytes_quote
                    #variant_constructor
                }
            };
            #[cfg(feature = "dyn_fns")]
            {
                let bitfield_dyn_trait_impl_fns = &gen.bitfield_dyn_trait;
                let from_vec_quote = &thing.read_fns.bitfield_dyn_trait;
                gen.bitfield_dyn_trait = quote! {
                    #bitfield_dyn_trait_impl_fns
                    #variant_id => {
                        #from_vec_quote
                        #variant_constructor
                    }
                };
                // Check Slice
                if let Some(slice_info) = thing.slice_info {
                    // do the match statement stuff
                    let check_slice_name = &slice_info.func;
                    let check_slice_struct = &slice_info.structure;
                    check_slice_fn = quote! {
                        #check_slice_fn
                        #variant_id => {
                            Ok(#checked_ident :: #variant_name (Self::#check_slice_name(buffer)?))
                        }
                    };
                    let check_slice_name_mut = &slice_info.mut_func;
                    let check_slice_struct_mut = &slice_info.mut_structure;
                    check_slice_mut_fn = quote! {
                        #check_slice_mut_fn
                        #variant_id => {
                            Ok(#checked_ident_mut :: #variant_name (Self::#check_slice_name_mut(buffer)?))
                        }
                    };

                    // do enum stuff
                    if !lifetime {
                        lifetime = true;
                    }
                    checked_slice_enum = quote! {
                        #checked_slice_enum
                        #v_name (#check_slice_struct<'a>),
                    };
                    checked_slice_enum_mut = quote! {
                        #checked_slice_enum_mut
                        #v_name (#check_slice_struct_mut<'a>),
                    };
                } else {
                    // do the match statement stuff
                    check_slice_fn = quote! {
                        #check_slice_fn
                        #variant_id => {
                            Ok(#checked_ident :: #variant_name)
                        }
                    };
                    check_slice_mut_fn = quote! {
                        #check_slice_mut_fn
                        #variant_id => {
                            Ok(#checked_ident_mut :: #variant_name)
                        }
                    };
                    // do enum stuff
                    checked_slice_enum = quote! {
                        #checked_slice_enum
                        #v_name,
                    };
                    checked_slice_enum_mut = quote! {
                        #checked_slice_enum_mut
                        #v_name,
                    };
                }
            }
            // Into Bytes
            let into_bytes_quote = &thing.write_fns.bitfield_trait;
            into_bytes_fn = quote! {
                #into_bytes_fn
                #variant_constructor => {
                    Self::#v_id_write_call(&mut output_byte_buffer, #variant_value);
                    #into_bytes_quote
                }
            };
            // Variant Id fn
            let id_field = if let Some(id_field) = variant.get_id_field()? {
                id_field
            } else {
                return Err(syn::Error::new(
                    variant.name.span(),
                    "variant didn't return its id field. (this is a bondrewd issue. please report)",
                ));
            };
            if id_field.attrs.capture_id {
                let id_field_name = id_field.ident().name();
                variant_value = quote! {#id_field_name};
            }

            let mut ignore_fields = if id_field.attrs.capture_id {
                let id_field_name = variant_value.clone();
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
            let from_vec_fn_inner = gen.bitfield_dyn_trait.clone();
            let comment_take = "Creates a new instance of `Self` by copying field from the bitfields, removing bytes that where used. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
            let comment = "Creates a new instance of `Self` by copying field from the bitfields. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
            gen.bitfield_dyn_trait = quote! {
                #[doc = #comment]
                fn from_slice(input_byte_buffer: &[u8]) -> Result<Self, bondrewd::BitfieldLengthError> {
                    if input_byte_buffer.len() < Self::BYTE_SIZE {
                        return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                    }
                    let #v_id = Self::#v_id_read_slice_call(&input_byte_buffer)?;
                    let out = match #v_id {
                        #from_vec_fn_inner
                    };
                    Ok(out)
                }
            };
            #[cfg(feature = "std")]
            {
                let from_vec_fn = &gen.bitfield_dyn_trait;
                gen.bitfield_dyn_trait = quote! {
                    #from_vec_fn
                    #[doc = #comment_take]
                    fn from_vec(input_byte_buffer: &mut Vec<u8>) -> Result<Self, bondrewd::BitfieldLengthError> {
                        if input_byte_buffer.len() < Self::BYTE_SIZE {
                            return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                        }
                        let #v_id = Self::#v_id_read_slice_call(&input_byte_buffer)?;
                        let out = match #v_id {
                            #from_vec_fn_inner
                        };
                        let _ = input_byte_buffer.drain(..Self::BYTE_SIZE);
                        Ok(out)
                    }
                };
            }
            let comment = format!(
                "Returns a checked structure which allows you to read any field for a `{}` from provided slice.",
                &self.name
            );
            gen.append_impl_fns(&quote! {
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
            gen.append_impl_fns(&quote! {
                #[doc = #comment]
                pub fn check_slice_mut(buffer: &mut [u8]) -> Result<#checked_ident_mut, bondrewd::BitfieldLengthError> {
                    let #v_id = Self::#v_id_read_slice_call(&buffer)?;
                    match #v_id {
                        #check_slice_mut_fn
                    }
                }
            });
            let lifetime = if lifetime {
                quote! {<'a>}
            } else {
                quote! {}
            };
            gen.append_checked_struct_impl_fns(&quote! {
                pub enum #checked_ident #lifetime {
                    #checked_slice_enum
                }
                pub enum #checked_ident_mut #lifetime {
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
        let id_ident = self.id_type_quote()?;
        gen.append_impl_fns(&quote! {
            pub fn id(&self) -> #id_ident {
                match self {
                    #id_fn
                }
            }
        });

        gen.bitfield_trait = quote! {
            #from_bytes_fn
            #into_bytes_fn
        };

        Ok(gen)
    }
}
