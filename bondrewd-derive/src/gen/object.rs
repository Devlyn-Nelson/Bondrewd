use std::env::current_dir;

use crate::common::{
    field::Info as FieldInfo, object::Info as ObjectInfo, r#enum::Info as EnumInfo,
    r#struct::Info as StructInfo,
};
#[cfg(feature = "dyn_fns")]
use crate::gen::field::{
    generate_read_slice_field_fn, generate_read_slice_field_fn_unchecked,
    generate_write_slice_field_fn, generate_write_slice_field_fn_unchecked, CheckedSliceGen,
};
#[cfg(feature = "setters")]
use crate::parse::struct_fns;
use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use super::{
    field::{generate_read_field_fn, generate_write_field_fn},
    GeneratedFunctions,
};

impl ObjectInfo {
    pub fn dump(&self) -> bool {
        match self {
            ObjectInfo::Struct(s) => s.dump(),
            ObjectInfo::Enum(e) => e.dump(),
        }
    }
    pub fn generate(&self) -> syn::Result<TokenStream> {
        let gen = match self {
            ObjectInfo::Struct(s) => Ok(s.generate_bitfield_functions()?.finish()),
            ObjectInfo::Enum(e) => e.generate_bitfield_functions(),
        }?;
        // get the struct size and name so we can use them in a quote.
        let struct_size = self.total_bytes();
        let struct_name = self.name();
        let impl_fns = gen.non_trait;
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
                let setters_quote = match struct_fns::create_setters_quotes(struct_info) {
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
        let bit_size = self.total_bits_no_fill();
        let trait_impl_fn = gen.bitfield_trait;
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
            let checked_structs = gen.checked_struct;
            let from_vec_quote = gen.bitfield_dyn_trait;
            output = quote! {
                #output
                #checked_structs
                impl bondrewd::BitfieldsDyn<#struct_size> for #struct_name {
                    #from_vec_quote
                }
            }
        }
        if self.dump() {
            let name = self.name().to_string().to_case(Case::Snake);
            match current_dir() {
                Ok(mut file_name) => {
                    file_name.push("target");
                    file_name.push(format!("{name}_code_gen.rs"));
                    let _ = std::fs::write(file_name, output.to_string());
                }
                Err(err) => {
                    return Err(syn::Error::new(self.name().span(), format!("Failed to dump code gen because target folder could not be located. remove `dump` from struct or enum bondrewd attributes. [{err}]")));
                }
            }
        }
        Ok(output)
    }
}

#[cfg(feature = "dyn_fns")]
pub struct CheckSliceNames {
    /// describes the check slice function name
    pub func: Ident,
    /// describes the check mut slice function name
    pub mut_func: Ident,
    /// describes the check slice struct name
    pub structure: Ident,
    /// describes the check mut slice struct name
    pub mut_structure: Ident,
}

// TODO nothing about field quotes should be public
/// This contains incomplete function generation. this should only be used by `StructInfo` or `EnumInfo` internally.
pub struct FieldQuotes {
    pub read_fns: GeneratedFunctions,
    pub write_fns: GeneratedFunctions,
    /// A list of field names to be used in initializing a new struct from bytes.
    pub field_list: TokenStream,
    #[cfg(feature = "dyn_fns")]
    pub slice_info: Option<CheckSliceNames>,
}
impl FieldQuotes {
    pub fn finish(self) -> GeneratedFunctions {
        let mut read = self.read_fns;
        read.merge(&self.write_fns);
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
        let from_bytes_quote = &quotes.read_fns.bitfield_trait;
        let fields_list = &quotes.field_list;
        // construct from bytes function. use input_byte_buffer as input name because,
        // that is what the field quotes expect to extract from.
        // wrap our list of field names with commas with Self{} so we it instantiate our struct,
        // because all of the from_bytes field quote store there data in a temporary variable with the same
        // name as its destination field the list of field names will be just fine.
        quotes.read_fns.bitfield_trait = quote! {
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
            let from_bytes_dyn_quote_inner = quotes.read_fns.bitfield_dyn_trait.clone();
            let comment_take = "Creates a new instance of `Self` by copying field from the bitfields, removing bytes that where used. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
            let comment = "Creates a new instance of `Self` by copying field from the bitfields. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
            quotes.read_fns.bitfield_dyn_trait = quote! {
                #[doc = #comment]
                fn from_slice(input_byte_buffer: &[u8]) -> Result<Self, bondrewd::BitfieldLengthError> {
                    if input_byte_buffer.len() < Self::BYTE_SIZE {
                        return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                    }
                    let out = {
                        #from_bytes_dyn_quote_inner
                        Self {
                            #fields_list
                        }
                    };
                    Ok(out)
                }
            };
            #[cfg(feature = "std")]
            {
                let from_bytes_dyn_quote = &quotes.read_fns.bitfield_dyn_trait;
                quotes.read_fns.bitfield_dyn_trait = quote! {
                    #from_bytes_dyn_quote
                    #[doc = #comment_take]
                    fn from_vec(input_byte_buffer: &mut Vec<u8>) -> Result<Self, bondrewd::BitfieldLengthError> {
                        if input_byte_buffer.len() < Self::BYTE_SIZE {
                            return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                        }
                        let out = {
                            #from_bytes_dyn_quote_inner
                            Self {
                                #fields_list
                            }
                        };
                        let _ = input_byte_buffer.drain(..Self::BYTE_SIZE);
                        Ok(out)
                    }
                };
            }
        }
        let into_bytes_quote = &quotes.write_fns.bitfield_trait;
        quotes.write_fns.bitfield_trait = quote! {
            fn into_bytes(self) -> [u8;#struct_size] {
                let mut output_byte_buffer: [u8;#struct_size] = [0u8;#struct_size];
                #into_bytes_quote
                output_byte_buffer
            }
        };
        Ok(quotes)
        // todo!("finish merged (from AND into) generate functions for StructInfo");
    }
    pub fn create_field_quotes(&self, enum_name: Option<&Ident>) -> syn::Result<FieldQuotes> {
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
        let fields = self.get_fields_for_gen()?;
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
        let checked = if fields.is_empty() {
            None
        } else {
            let struct_name = if let Some(e_name) = enum_name {
                quote::format_ident!("{e_name}{}", &self.name)
            } else {
                self.name.clone()
            };
            let vis = self.vis();
            let checked_ident = quote::format_ident!("{struct_name}Checked");
            let checked_mut_ident = quote::format_ident!("{struct_name}CheckedMut");
            let unchecked_functions = gen_read.checked_struct;
            let unchecked_mut_functions = gen_write.checked_struct;
            let comment = format!("A Structure which provides functions for getting the fields of a [{struct_name}] in its bitfield form.");
            let comment_mut = format!("A Structure which provides functions for getting and setting the fields of a [{struct_name}] in its bitfield form.");
            let unchecked_comment = format!("Panics if resulting `{checked_ident}` does not contain enough bytes to read a field that is attempted to be read.");
            let unchecked_comment_mut = format!("Panics if resulting `{checked_mut_ident}` does not contain enough bytes to read a field that is attempted to be read or written.");
            gen_write.checked_struct = quote! {
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
            gen_read.checked_struct = quote! {
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
                None
            };

            let check_slice_info = CheckedSliceGen::new(&self.name, self.total_bytes(), ename);

            let check_slice_fn = check_slice_info.fn_gen;
            let impl_fns = gen_read.non_trait;
            gen_read.non_trait = quote! {
                #impl_fns
                #check_slice_fn
            };

            let check_slice_mut_fn = check_slice_info.mut_fn_gen;
            let impl_fns = gen_write.non_trait;
            gen_write.non_trait = quote! {
                #impl_fns
                #check_slice_mut_fn
            };
            Some(CheckSliceNames {
                func: check_slice_info.fn_name,
                mut_func: check_slice_info.mut_fn_name,
                structure: checked_ident,
                mut_structure: checked_mut_ident,
            })
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
        field_access: &super::field::GeneratedQuotes,
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
            #[cfg(feature = "dyn_fns")]
            &field_name,
            &prefixed_name,
            field_extractor,
            &mut impl_fns,
            #[cfg(feature = "dyn_fns")]
            &mut checked_struct_impl_fns,
        );
        gen.append_impl_fns(&impl_fns);
        #[cfg(feature = "dyn_fns")]
        gen.append_checked_struct_impl_fns(&checked_struct_impl_fns);

        // fake fields do not exist in the actual structure and should only have functions
        // that read or write values into byte arrays.
        if !field.attrs.reserve.is_fake_field() {
            // put the name of the field into the list of fields that are needed to create
            // the struct.
            if self.attrs.default_endianess.is_field_order_reversed() {
                *field_name_list = quote! {#field_name, #field_name_list}
            } else {
                *field_name_list = quote! {#field_name_list #field_name,}
            };
            let peek_call = if field.attrs.capture_id {
                // put the field extraction in the actual from bytes.
                if field.attrs.reserve.wants_read_fns() {
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
                let read_stuff = field_access.read();
                if field.attrs.reserve.wants_read_fns() {
                    // let fn_field_name = format_ident!("read_{prefixed_name}");
                    quote! {
                        let #field_name = #read_stuff;
                    }
                } else {
                    quote! { let #field_name = Default::default(); }
                }
            };
            gen.append_bitfield_trait_impl_fns(&peek_call);
            #[cfg(feature = "dyn_fns")]
            gen.append_bitfield_dyn_trait_impl_fns(&quote! {
                let #field_name = #field_extractor;
            });
        }
        Ok(())
    }
    fn make_read_fns_inner(
        &self,
        field: &FieldInfo,
        #[cfg(feature = "dyn_fns")] field_name: &Ident,
        prefixed_name: &Ident,
        field_extractor: &TokenStream,
        peek_quote: &mut TokenStream,
        #[cfg(feature = "dyn_fns")] peek_slice_fns_option: &mut TokenStream,
    ) {
        *peek_quote = generate_read_field_fn(field_extractor, field, self, prefixed_name);
        // make the slice functions if applicable.
        #[cfg(feature = "dyn_fns")]
        {
            let peek_slice_quote =
                generate_read_slice_field_fn(field_extractor, field, self, prefixed_name);
            *peek_quote = quote! {
                #peek_quote
                #peek_slice_quote
            };

            let peek_slice_unchecked_quote =
                generate_read_slice_field_fn_unchecked(field_extractor, field, self, field_name);
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
        field_access: &super::field::GeneratedQuotes,
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
        if field.attrs.reserve.wants_write_fns() {
            if variant_name.is_some() {
                let fn_name = format_ident!("write_{prefixed_name}");
                gen.append_bitfield_trait_impl_fns(&quote! {
                    Self::#fn_name(&mut output_byte_buffer, #field_name);
                });
            } else {
                gen.append_bitfield_trait_impl_fns(&quote! {
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

        gen.append_impl_fns(&impl_fns);
        #[cfg(feature = "dyn_fns")]
        gen.append_checked_struct_impl_fns(&checked_struct_impl_fns);
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
        *write_quote = generate_write_field_fn(field_setter, clear_quote, field, self, field_name);
        #[cfg(feature = "dyn_fns")]
        {
            let set_slice_quote =
                generate_write_slice_field_fn(field_setter, clear_quote, field, self, field_name);
            *write_quote = quote! {
                #write_quote
                #set_slice_quote
            };
            let set_slice_unchecked_quote =
                generate_write_slice_field_fn_unchecked(field_setter, clear_quote, field, self);
            *write_slice_fns_option = quote! {
                #write_slice_fns_option
                #set_slice_unchecked_quote
            };
        }
    }
}
