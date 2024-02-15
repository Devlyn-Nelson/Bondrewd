use crate::structs::common::{EnumInfo, FieldInfo, ObjectInfo, StructInfo};
use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

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
        let quote = self.bitfield_trait_impl_fns;
        quote
    }
}

impl GeneratedFunctions {
    pub fn merge(&mut self, other: Self) {
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
    pub fn append_bitfield_trait_impl_fns(&mut self, quote: TokenStream) {
        let old = &self.bitfield_trait_impl_fns;
        self.bitfield_trait_impl_fns = quote! {
            #old
            #quote
        };
    }
    pub fn append_impl_fns(&mut self, quote: TokenStream) {
        let old = &self.impl_fns;
        self.impl_fns = quote! {
            #old
            #quote
        };
    }
    #[cfg(feature = "dyn_fns")]
    pub fn append_checked_struct_impl_fns(&mut self, quote: TokenStream) {
        let old = &self.checked_struct_impl_fns;
        self.checked_struct_impl_fns = quote! {
            #old
            #quote
        };
    }
    #[cfg(feature = "dyn_fns")]
    pub fn append_bitfield_dyn_trait_impl_fns(&mut self, quote: TokenStream) {
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
        // todo!(
        //     "generate all code here, this means moving the code generation code inside lib.rs here"
        // )
    }
}

/// This contains incomplete function generation. this should only be used by `StructInfo` or `EnumInfo` internally.
struct FieldQuotes {
    read_fns: GeneratedFunctions,
    /// A list of field names to be used in initializing a new struct from bytes.
    field_list: TokenStream,
}

impl StructInfo {
    pub fn generate_bitfield_functions(&self) -> syn::Result<GeneratedFunctions> {
        let mut quotes = self.create_field_quotes(None)?;
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
    fn create_field_quotes(&self, enum_name: Option<&Ident>)-> syn::Result<FieldQuotes> {
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
        let fields = if enum_name.is_some() && !self.fields[0].attrs.capture_id {
            &self.fields[1..]
        } else {
            &self.fields[..]
        };
        let mut gen = GeneratedFunctions {
            #[cfg(feature = "dyn_fns")]
            impl_fns,
            ..Default::default()
        };
        let mut field_name_list = quote!{};
        for field in fields {
            self.make_read_fns(field, &variant_name, &mut field_name_list, &mut gen)?;
        }
        Ok(FieldQuotes {
            read_fns: gen,
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
    pub fn generate_bitfield_functions(
        &self,
    ) -> syn::Result<GeneratedFunctions> {
        let enum_name: Option<&Ident> = Some(&self.name);
        // function for getting the id of an enum.
        let _id_fn = quote! {};
        let _bitfield_trait_impl_fns = quote! {};
        let _impl_fns = quote! {};
        #[cfg(feature = "dyn_fns")]
        let _bitfield_dyn_trait_impl_fns = quote! {};

        todo!("finish merged (from AND into) generate functions for EnumInfo");
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
    let comment = format!("Reads bits {} through {} within `input_byte_buffer`, getting the `{field_name}` field of a `{struct_name}` in bitfield form.", bit_range.start, bit_range.end - 1);
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
