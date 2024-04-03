use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::{field::Info as FieldInfo, AttrInfo, Visibility};

#[derive(Clone, Debug)]
pub struct Info {
    /// Name of the variant or struct
    pub name: Ident,
    /// ATtributes describing the bit layout
    pub attrs: AttrInfo,
    /// All fields in the struct/variant
    pub fields: Vec<FieldInfo>,
    /// The viability of the struct/enum
    pub vis: Visibility,
    /// Is it a tuple struct/variant
    pub tuple: bool,
}

impl Info {
    pub fn dump(&self) -> bool {
        self.attrs.dump
    }
    #[cfg(feature = "dyn_fns")]
    pub fn vis(&self) -> &syn::Visibility {
        &*self.vis
    }
    // TODO move the check to field attrs making this return usize.
    pub fn get_flip(&self) -> Option<usize> {
        if self.attrs.default_endianess.is_byte_order_reversed() {
            Some(self.total_bytes() - 1)
        } else {
            None
        }
    }
    pub(crate) fn get_id_field(&self) -> syn::Result<Option<&FieldInfo>> {
        if self.attrs.id.is_none() {
            return Ok(None);
        }
        let thing = if self.attrs.default_endianess.is_field_order_reversed() {
            self.fields.last()
        } else {
            self.fields.first()
        };
        if let Some(field) = thing {
            Ok(Some(field))
        } else {
            Err(syn::Error::new(
                self.name.span(),
                format!(
                    "`StructInfo` had variant id but no fields. (this is a bondrewd problem, please report issue)"
                ),
            ))
        }
    }
    pub(crate) fn get_fields_for_gen(&self) -> syn::Result<&[FieldInfo]> {
        if if let Some(field) = self.get_id_field()? {
            !field.attrs.capture_id
        } else {
            false
        } {
            if self.attrs.default_endianess.is_field_order_reversed() {
                Ok(&self.fields[..self.fields.len() - 1])
            } else {
                Ok(&self.fields[1..])
            }
        } else {
            Ok(&self.fields)
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
                "variant id was unknown at time of code generation (this is a bondrewd problem, please report issue)",
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
    pub fn total_bits_no_fill(&self) -> usize {
        let mut total: usize = 0;
        for field in &self.fields {
            total += field.bit_size_no_fill();
        }

        total
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bits().div_ceil(8)
    }
}
