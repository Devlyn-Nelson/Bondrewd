use crate::enums::parse::{EnumInfo, EnumVariantType};
use quote::quote;

/// Generates a `PartialEq` implementation for the given `enum_info`.
/// N.B. The `PartialEq` implementation will only be generated iff `enum_partial_eq` feature is enabled.
///
/// This generates the equivalent of the following code:
/// ```
/// use bondrewd::*;
///
/// #[derive(BitfieldEnum)]
/// #[bondrewd_enum(u8)]
/// pub enum Test {
///     Zero,
///     One,
///     Two,
///     Invalid
/// }
///
/// impl PartialEq<u8> for Test {
///     fn eq(&self, other: &u8) -> bool {
///         match (self, other) {
///             (Self::Zero, 0) => true,
///             (Self::One, 1) => true,
///             (Self::Two, 2) => true,
///             _ => false
///         }
///     }
/// }
/// ```
pub fn generate_partial_eq(enum_info: &EnumInfo) -> proc_macro2::TokenStream {
    // Short circuit if we're not generating parital_eq
    if !enum_info.partial_eq {
        return quote! {};
    }

    let mut comp_arms = quote! {};
    let enum_name = &enum_info.name;
    let primitive_ty = &enum_info.primitive;
    for var in enum_info.variants.iter() {
        let name = &var.name;
        let arm = match var.value {
            EnumVariantType::UnsignedValue(v) => {
                quote! { (Self::#name, #v) => true, }
            },
            EnumVariantType::CatchAll(_) => {
                quote! { _ => false, }
            },
            EnumVariantType::CatchPrimitive => {
                quote! { _ => false, }
            }
        };
        comp_arms = quote! {
            #comp_arms
            #arm
        }
    }
    quote! {
        impl std::cmp::PartialEq<#primitive_ty> for #enum_name {
            fn eq(&self, other: &#primitive_ty) -> bool {
                match (self, other) {
                    #comp_arms
                }
            }
        }
    }
}