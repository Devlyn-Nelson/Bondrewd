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

impl ObjectInfo {
    pub fn generate(&self) -> syn::Result<GeneratedFunctions> {
        match self {
            ObjectInfo::Struct(s) => {
                s.generate_bitfield_functions();
            }
            ObjectInfo::Enum(e) => {
                e.generate_bitfield_functions(Some(&e.name));
            }
        }
        todo!(
            "generate all code here, this means moving the code generation code inside lib.rs here"
        )
    }
}

impl StructInfo {
    pub fn generate_bitfield_functions(&self) -> syn::Result<GeneratedFunctions> {
        todo!("finish merged (from AND into) generate functions for StructInfo");
    }
    fn create_field_quotes(&self, enum_name: Option<&Ident>) {
        let lower_name = if enum_name.is_some() {
            Some(format_ident!(
                "{}",
                self.name.to_string().to_case(Case::Snake)
            ))
        } else {
            None
        };
    }
    fn make_read_fns(&self, field: &FieldInfo, enum_name: &Option<Ident>, field_name_list: &mut TokenStream) -> syn::Result<()> {
        let mut impl_fns = quote!{};
        let mut checked_struct_impl_fns = quote!{};
        let field_access = field.get_quotes(self)?;
        self.make_read_fns_inner(field, enum_name, field_access.read(), &mut impl_fns, &mut checked_struct_impl_fns);
        #[cfg(feature = "dyn_fns")]
        if enum_name.is_some() {
            // TODO impl proper check_slice support for enums. (should just be get_check_slice_fn for each variant)
            impl_fns = quote! {#impl_fns};
        } else {
            let check_slice_quote = get_check_slice_fn(&self.name, self.total_bytes());
            impl_fns = quote! {
                #impl_fns
                #check_slice_quote
            };
        };
        Ok(())
    }
    fn make_read_fns_inner(
        &self,
        field: &FieldInfo,
        prefix_option: &Option<Ident>,
        field_extractor: &TokenStream,
        peek_quote: &mut TokenStream,
        peek_slice_fns_option: &mut TokenStream,
    ) {
        let field_name = field.ident().ident();
        let peek_name = if let Some(prefix) = prefix_option {
            format_ident!("{prefix}_{}", field_name)
        } else{
            format_ident!("{}", field_name)
        };
        *peek_quote = generate_read_field_fn(field_extractor, field, self, prefix_option, &peek_name);
        // make the slice functions if applicable.
        #[cfg(feature = "dyn_fns")]
        {
            let peek_slice_quote =
                generate_read_slice_field_fn(field_extractor, field, self, prefix_option, &peek_name);
            *peek_quote = quote! {
                #peek_quote
                #peek_slice_quote
            };

            let peek_slice_unchecked_quote =
                generate_read_slice_field_fn_unchecked(field_extractor, field, self, prefix_option, &peek_name);
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
        enum_name: Option<&Ident>,
    ) -> syn::Result<GeneratedFunctions> {
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
    prefix: &Option<Ident>,
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
    prefix: &Option<Ident>,
    field_name: &Ident,
) -> TokenStream {
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
#[cfg(feature = "dyn_fns")]
fn generate_read_slice_field_fn_unchecked(
    field_quote: &TokenStream,
    field: &FieldInfo,
    info: &StructInfo,
    prefix: &Option<Ident>,
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