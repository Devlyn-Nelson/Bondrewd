use syn::{spanned::Spanned, Meta};

use crate::common::{
    r#enum::{IdPosition, Info as EnumInfo},
    r#struct::Info as StructInfo,
    AttrInfo, StructEnforcement,
};

use super::get_lit_int;

#[derive(Clone)]
pub struct AttrInfoBuilder {
    pub id_bits: Option<usize>,
    pub id_position: IdPosition,
    pub total_bit_size: Option<usize>,
    pub payload_bit_size: Option<usize>,
}

impl Default for AttrInfoBuilder {
    fn default() -> Self {
        Self {
            id_bits: None,
            id_position: IdPosition::Leading,
            total_bit_size: None,
            payload_bit_size: None,
        }
    }
}

impl EnumInfo {
    pub(crate) fn parse_attrs_meta(
        info: &mut AttrInfo,
        enum_info: &mut AttrInfoBuilder,
        meta: &Meta,
    ) -> Result<(), syn::Error> {
        let enum_result = Self::parse_attrs_meta_enum(enum_info, meta)?;
        let struct_result = StructInfo::parse_attrs_meta(info, meta, false)?;
        if !(enum_result || struct_result) {
            return Err(syn::Error::new(meta.span(), "invalid enum attribute"));
        }
        if let StructEnforcement::EnforceBitAmount(bits) = info.enforcement {
            enum_info.total_bit_size = Some(bits);
            info.enforcement = StructEnforcement::NoRules;
        }
        Ok(())
    }
    /// a return of `Ok(false)` means that no attribute was found. but due to enums
    /// using struct and enum attribute we can't throw an error unless neither have
    /// an error.
    fn parse_attrs_meta_enum(
        enum_info: &mut AttrInfoBuilder,
        meta: &Meta,
    ) -> Result<bool, syn::Error> {
        match meta {
            Meta::NameValue(value) => {
                if let Some(ident) = value.path.get_ident() {
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        "id_bit_length" => {
                            let val =
                                get_lit_int(&value.value, ident, Some("id_bit_length = \"2\""))?;
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    if value > 128 {
                                        return Err(syn::Error::new(
                                            ident.span(),
                                            "Maximum id bits is 128.",
                                        ));
                                    }
                                    enum_info.id_bits = Some(value);
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        ident.span(),
                                        format!("failed parsing id-bits value [{err}]"),
                                    ));
                                }
                            }
                        }
                        "id_byte_length" => {
                            let val =
                                get_lit_int(&value.value, ident, Some("id_byte_length = \"2\""))?;
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    if value > 16 {
                                        return Err(syn::Error::new(
                                            ident.span(),
                                            "Maximum id bytes is 16.",
                                        ));
                                    }
                                    enum_info.id_bits = Some(value * 8);
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        ident.span(),
                                        format!("failed parsing id-bytes value [{err}]"),
                                    ));
                                }
                            }
                        }
                        "payload_bit_length" => {
                            let val = get_lit_int(
                                &value.value,
                                ident,
                                Some("payload_bit_length = \"6\""),
                            )?;
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    enum_info.payload_bit_size = Some(value);
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        ident.span(),
                                        format!("failed parsing payload-bits value [{err}]"),
                                    ));
                                }
                            }
                        }
                        "payload_byte_length" => {
                            let val = get_lit_int(
                                &value.value,
                                ident,
                                Some("payload_byte_length = \"6\""),
                            )?;
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    enum_info.payload_bit_size = Some(value * 8);
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        ident.span(),
                                        format!("failed parsing payload-bytes value [{err}]"),
                                    ));
                                }
                            }
                        }
                        _ => {
                            return Ok(false);
                        }
                    }
                }
            }
            Meta::Path(value) => {
                if let Some(ident) = value.get_ident() {
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        "id_tail" => {
                            enum_info.id_position = IdPosition::Trailing;
                        }
                        "id_head" => {
                            enum_info.id_position = IdPosition::Leading;
                        }
                        _ => {
                            return Ok(false);
                        }
                    }
                }
            }
            Meta::List(ref meta_list) => {
                return Err(syn::Error::new(
                    meta_list.span(),
                    "bondrewd does not offer any list attribute for fields",
                ))
            }
        }
        Ok(true)
    }
}
