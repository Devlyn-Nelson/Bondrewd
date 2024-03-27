pub mod r#enum;
pub mod field;
pub mod object;

use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone, Default)]
pub struct GeneratedFunctions {
    /// Functions that belong in `Bitfields` impl for object.
    pub bitfield_trait: TokenStream,
    /// Functions that belong in impl for object.
    pub non_trait: TokenStream,
    /// Functions that belong in impl for generated checked slice object.
    #[cfg(feature = "dyn_fns")]
    pub checked_struct: TokenStream,
    /// Functions that belong in `BitfieldsDyn` impl for object.
    #[cfg(feature = "dyn_fns")]
    pub bitfield_dyn_trait: TokenStream,
}

impl From<GeneratedFunctions> for TokenStream {
    fn from(val: GeneratedFunctions) -> Self {
        let trait_fns = val.bitfield_trait;
        let impl_fns = val.non_trait;
        #[cfg(feature = "dyn_fns")]
        let unchecked = val.checked_struct;
        #[cfg(feature = "dyn_fns")]
        let dyn_trait_fns = val.bitfield_dyn_trait;
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
    pub fn merge(&mut self, other: &Self) {
        let bitfield_trait_impl_fns = &self.bitfield_trait;
        let other_bitfield_trait_impl_fns = &other.bitfield_trait;
        self.bitfield_trait = quote! {
            #bitfield_trait_impl_fns
            #other_bitfield_trait_impl_fns
        };
        let impl_fns = &self.non_trait;
        let other_impl_fns = &other.non_trait;
        self.non_trait = quote! {
            #impl_fns
            #other_impl_fns
        };
        #[cfg(feature = "dyn_fns")]
        {
            let checked_struct_impl_fns = &self.checked_struct;
            let other_checked_struct_impl_fns = &other.checked_struct;
            self.checked_struct = quote! {
                #checked_struct_impl_fns
                #other_checked_struct_impl_fns
            };
            let bitfield_dyn_trait_impl_fns = &self.bitfield_dyn_trait;
            let other_bitfield_dyn_trait_impl_fns = &other.bitfield_dyn_trait;
            self.bitfield_dyn_trait = quote! {
                #bitfield_dyn_trait_impl_fns
                #other_bitfield_dyn_trait_impl_fns
            };
        }
    }
    pub fn append_bitfield_trait_impl_fns(&mut self, quote: &TokenStream) {
        let old = &self.bitfield_trait;
        self.bitfield_trait = quote! {
            #old
            #quote
        };
    }
    pub fn append_impl_fns(&mut self, quote: &TokenStream) {
        let old = &self.non_trait;
        self.non_trait = quote! {
            #old
            #quote
        };
    }
    #[cfg(feature = "dyn_fns")]
    pub fn append_checked_struct_impl_fns(&mut self, quote: &TokenStream) {
        let old = &self.checked_struct;
        self.checked_struct = quote! {
            #old
            #quote
        };
    }
    #[cfg(feature = "dyn_fns")]
    pub fn append_bitfield_dyn_trait_impl_fns(&mut self, quote: &TokenStream) {
        let old = &self.bitfield_dyn_trait;
        self.bitfield_dyn_trait = quote! {
            #old
            #quote
        };
    }
}
