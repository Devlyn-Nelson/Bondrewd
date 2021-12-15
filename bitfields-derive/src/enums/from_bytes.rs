use crate::enums::parse::{EnumInfo, EnumVariantType};
use quote::quote;

pub fn generate_from_bytes(enum_info: &EnumInfo) -> proc_macro2::TokenStream {
    let mut arms = quote! {};
    for var in enum_info.variants.iter() {
        let name = &var.name;
        let arm = match var.value {
            EnumVariantType::UnsignedValue(ref value) => {
                quote! {
                    #value => Self::#name,
                }
            }
            EnumVariantType::CatchAll(_) => {
                quote! {
                    _ => Self::#name,
                }
            }
            EnumVariantType::CatchPrimitive => {
                quote! {
                    _ => Self::#name(input),
                }
            }
        };
        arms = quote! {
            #arms
            #arm
        };
    }
    quote! {
        fn from_primitive(input: u8) -> Self {
            match input {
                #arms
            }
        }
    }
}
