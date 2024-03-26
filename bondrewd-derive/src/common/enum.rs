use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::{r#struct::StructInfo, AttrInfo};

#[derive(Clone)]
pub enum IdPosition {
    Leading,
    Trailing,
}

pub struct EnumInfo {
    pub name: Ident,
    pub variants: Vec<StructInfo>,
    pub attrs: EnumAttrInfo,
    pub vis: syn::Visibility,
}

impl EnumInfo {
    pub const VARIANT_ID_NAME: &'static str = "variant_id";
    pub const VARIANT_ID_NAME_KEBAB: &'static str = "variant-id";
    // #[cfg(feature = "dyn_fns")]
    // pub fn vis(&self) -> &syn::Visibility {
    //     &self.vis
    // }
    pub fn dump(&self) -> bool {
        self.attrs.attrs.dump
    }
    pub fn total_bits(&self) -> usize {
        let mut total = self.variants[0].total_bits();
        for variant in self.variants.iter().skip(1) {
            let t = variant.total_bits();
            if t > total {
                total = t;
            }
        }
        total
    }
    pub fn total_bytes(&self) -> usize {
        self.total_bits().div_ceil(8)
    }
    pub fn id_type_ident(&self) -> syn::Result<TokenStream> {
        match self.attrs.id_bits {
            0..=8 => Ok(quote! {u8}),
            9..=16 => Ok(quote! {u16}),
            17..=32 => Ok(quote! {u32}),
            33..=64 => Ok(quote! {u64}),
            65..=128 => Ok(quote! {u128}),
            _ => Err(syn::Error::new(
                self.name.span(),
                "variant id size is invalid",
            )),
        }
    }
}

#[derive(Clone)]
pub struct EnumAttrInfo {
    pub id_bits: usize,
    pub id_position: IdPosition,
    // TODO we should add an option of where to but the fill bytes. currently the generative code will always
    // have the "useful" data proceeding each other then filler. maybe someone will want id -> fill -> variant_data
    /// The Full size of the enum. while we allow variants to be take differing sizes, the
    /// enum will always use the full size, filling unused space with a pattern
    /// of bytes. `payload_bit_size` is simply the largest variant's size and
    /// therefore the total bytes used by the enum regardless of differing sized variants.
    pub payload_bit_size: usize,
    pub attrs: AttrInfo,
}
