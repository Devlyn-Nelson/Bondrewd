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
    fn make_read_fns(&self, field: &FieldInfo, enum_name: &Option<Ident>) -> syn::Result<()>{
        let mut field_name_list = quote! {};
        // all of the fields extraction will be appended to this
        let mut from_bytes_quote = quote! {};
        let mut from_vec_quote = quote! {};
        // all quote with all of the peek slice functions appended to it. the second tokenstream is an unchecked
        // version for the checked_struct.
        #[cfg(feature = "dyn_fns")]
        let mut peek_slice_fns_option = quote! {};
        #[cfg(not(feature = "dyn_fns"))]
        let peek_slice_fns_option = quote! {};
        // all quote with all of the peek functions appended to it.
        let mut peek_fns_quote;
        #[cfg(not(feature = "dyn_fns"))]
        {
            peek_fns_quote = quote! {};
        }
        #[cfg(feature = "dyn_fns")]
        {
            peek_fns_quote = if enum_name.is_some() {
                quote! {}
            } else {
                get_check_slice_fn(&info.name, info.total_bytes())
            };
        }
        let field_access = field.get_quotes(self)?;
        let _ = self.make_read_fns_inner(field, enum_name, &mut peek_fns_quote, field_access.read());
        Ok(())
    }
    fn make_read_fns_inner(&self, field: &FieldInfo, prefix: &Option<Ident>, peek_fns_quote: &mut TokenStream,field_extractor: &TokenStream) -> TokenStream {
        // let peek_name = if let Some((prefix, _, _)) = enum_name {
        //     format_ident!("read_{prefix}_{}", field_name.as_ref())
        // } else{
        //     format_ident!("read_{}", field_name.as_ref())
        // };
        // let field_extractor = get_field_quote(field, info.get_flip())?;

        // let peek_quote = generate_read_field_fn(field_extractor, field, info, enum_name);
        // *peek_fns_quote = quote! {
        //     #peek_fns_quote
        //     #peek_quote
        // };
        // // make the slice functions if applicable.
        // #[cfg(feature = "dyn_fns")]
        // {
        //     let peek_slice_quote =
        //         generate_read_slice_field_fn(field_extractor, field, info, enum_name);
        //     *peek_fns_quote = quote! {
        //         #peek_fns_quote
        //         #peek_slice_quote
        //     };

        //     let peek_slice_unchecked_quote =
        //         generate_read_slice_field_fn_unchecked(field_extractor, field, info, enum_name);
        //     *peek_slice_fns_option = quote! {
        //         #peek_slice_fns_option
        //         #peek_slice_unchecked_quote
        //     };
        // }
        // Ok(field_extractor)
        todo!("finish");
    }
    fn make_write_fns(&self){

    }
}

impl EnumInfo {
    pub fn generate_bitfield_functions(&self, enum_name: Option<&Ident>) -> syn::Result<GeneratedFunctions> {
        // function for getting the id of an enum.
        let _id_fn = quote! {};
        let _bitfield_trait_impl_fns = quote! {};
        let _impl_fns = quote! {};
        #[cfg(feature = "dyn_fns")]
        let _bitfield_dyn_trait_impl_fns = quote! {};

        todo!("finish merged (from AND into) generate functions for EnumInfo");
    }
}
