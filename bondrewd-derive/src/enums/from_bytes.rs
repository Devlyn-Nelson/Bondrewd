use crate::enums::parse::{EnumInfo, EnumVariantType};
use quote::quote;

pub fn generate_from_bytes(enum_info: &EnumInfo) -> syn::Result<proc_macro2::TokenStream> {
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
            EnumVariantType::CatchPrimitive(ref field_name) => {
                if let Some(ref field_name) = field_name {
                    quote! {
                        _ => Self::#name { #field_name = input },
                    }
                } else {
                    quote! {
                        _ => Self::#name(input),
                    }
                }
            }
            EnumVariantType::Skip(_) => {
                return Err(syn::Error::new(
                    var.name.span(),
                    "skip got into from bytes, please open issue.",
                ))
            }
        };
        arms = quote! {
            #arms
            #arm
        };
    }
    let struct_name = &enum_info.name;
    let comment =
        format!("Returns `{struct_name}` Variant that was represented by the provided `u8`.");
    Ok(quote! {
        #[doc = #comment]
        fn from_primitive(input: u8) -> Self {
            match input {
                #arms
            }
        }
    })
}
