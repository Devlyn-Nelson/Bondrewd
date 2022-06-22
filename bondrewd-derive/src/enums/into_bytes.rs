use crate::enums::parse::{EnumInfo, EnumVariantType};
use quote::quote;

pub fn generate_into_bytes(enum_info: &EnumInfo) -> syn::Result<proc_macro2::TokenStream> {
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
            EnumVariantType::CatchPrimitive(ref field_name) => {
                if let Some(field_name) = field_name {
                    quote! {
                        Self::#name { #field_name } => #field_name,
                    }
                } else {
                    quote! {
                        Self::#name(value) => value,
                    }
                }
            }
            EnumVariantType::Skip(_) => {
                return Err(syn::Error::new(
                    var.name.span(),
                    "skip got into into bytes, please open issue.",
                ))
            }
        };
        arms = quote! {
            #arms
            #arm
        };
    }
    let comment = format!("Returns a u8 representing a Variant of `#name`.");
    Ok(quote! {
        #[doc = #comment]
        fn into_primitive(self) -> u8 {
            match self {
                #arms
            }
        }
    })
}
