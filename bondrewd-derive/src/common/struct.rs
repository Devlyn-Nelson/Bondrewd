use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::{field::Info as FieldInfo, AttrInfo};

#[derive(Clone)]
pub struct Info {
    pub name: Ident,
    pub attrs: AttrInfo,
    pub fields: Vec<FieldInfo>,
    pub vis: syn::Visibility,
    pub tuple: bool,
}

impl Info {
    pub fn dump(&self) -> bool {
        self.attrs.dump
    }
    #[cfg(feature = "dyn_fns")]
    pub fn vis(&self) -> &syn::Visibility {
        &self.vis
    }
    pub fn get_flip(&self) -> Option<usize> {
        if self.attrs.flip {
            Some(self.total_bytes() - 1)
        } else {
            None
        }
    }
    pub fn id_or_field_name(&self) -> syn::Result<TokenStream> {
        for field in &self.fields {
            if field.attrs.capture_id {
                let name = field.ident().name();
                return Ok(quote! {#name});
            }
        }
        if let Some(id) = self.attrs.id {
            match TokenStream::from_str(format!("{id}").as_str()) {
                Ok(id) => Ok(id),
                Err(err) => Err(syn::Error::new(
                    self.name.span(),
                    format!(
                        "variant id was not able to be formatted for of code generation. [{err}]"
                    ),
                )),
            }
        } else {
            Err(syn::Error::new(
                self.name.span(),
                "variant id was unknown at time of code generation",
            ))
        }
    }
    pub fn total_bits(&self) -> usize {
        let mut total: usize = 0;
        for field in &self.fields {
            total += field.bit_size();
        }

        total
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bits().div_ceil(8)
    }
}
