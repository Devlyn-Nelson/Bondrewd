use crate::{
    enums::parse::EnumInfo,
    structs::common::{ObjectInfo, StructInfo},
};
use quote::quote;

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
        todo!(
            "generate all code here, this means moving the code generation code inside lib.rs here"
        )
    }
}

impl StructInfo {
    pub fn generate_bitfield_functions(&self) -> syn::Result<GeneratedFunctions> {
        todo!("finish merged (from AND into) generate functions for StructInfo");
    }
}

impl EnumInfo {
    pub fn generate_bitfield_functions(&self) -> syn::Result<GeneratedFunctions> {
        // function for getting the id of an enum.
        let _id_fn = quote! {};
        let _bitfield_trait_impl_fns = quote! {};
        let _impl_fns = quote! {};
        #[cfg(feature = "dyn_fns")]
        let _bitfield_dyn_trait_impl_fns = quote! {};
        todo!("finish merged (from AND into) generate functions for EnumInfo");
    }
}
