use crate::enums::parse::{EnumInfo, EnumVariantType};
use quote::quote;

pub fn generate_into_bytes(enum_info: &EnumInfo) -> proc_macro2::TokenStream {
    let mut arms = quote! {};
    for var in enum_info.variants.iter() {
        let name = &var.name;
        let arm = match var.value {
            EnumVariantType::UnsignedValue(ref value) => {
                quote! {
                    Self::#name => #value,
                }
            }
            EnumVariantType::CatchAll(ref output_id) => {
                quote! {
                    Self::#name => #output_id,
                }
            }
            EnumVariantType::CatchPrimitive => {
                quote! {
                    Self::#name(value) => value,
                }
            }
        };
        arms = quote! {
            #arms
            #arm
        };
    }
    quote! {
        fn into_primitive(self) -> u8 {
            match self {
                #arms
            }
        }
    }
}
