use syn::{spanned::Spanned, Error, Meta};

use crate::common::{
    r#enum::Info as EnumInfo, r#struct::Info as StructInfo, AttrInfo, Endianness, FillBits,
    StructEnforcement,
};

use super::{get_lit_int, get_lit_str};

impl StructInfo {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn parse_attrs_meta(
        info: &mut AttrInfo,
        meta: &Meta,
        is_variant: bool,
    ) -> Result<bool, syn::Error> {
        match meta {
            Meta::NameValue(ref value) => {
                if let Some(ident) = value.path.get_ident() {
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        EnumInfo::VARIANT_ID_NAME | EnumInfo::VARIANT_ID_NAME_KEBAB => {
                            if is_variant {
                                let val = get_lit_int(&value.value, ident, Some("variant_id = 1"))?;
                                match val.base10_parse::<u128>() {
                                    Ok(value) => {
                                        if info.id.is_none() {
                                            info.id = Some(value);
                                        } else {
                                            return Err(syn::Error::new(
                                                ident.span(),
                                                "must not have 2 ids defined.",
                                            ));
                                        }
                                    }
                                    Err(err) => {
                                        return Err(syn::Error::new(
                                            ident.span(),
                                            format!("failed parsing id value [{err}]"),
                                        ))
                                    }
                                }
                            } else {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    format!(
                                        "{} can only be used on enum variants",
                                        EnumInfo::VARIANT_ID_NAME
                                    ),
                                ));
                            }
                        }
                        "bit_traversal" => {
                            let val =
                                get_lit_str(&value.value, ident, Some("bit_traversal = \"msb\""))?
                                    .value();
                            let thing = val.trim();
                            match thing {
                                "lsb" | "lsb0" => return Err(syn::Error::new(
                                    ident.span(),
                                    format!("Please replace `bit_traversal = \"{thing}\"` with `bit_traversal = \"back\"`"),
                                )),
                                "msb" | "msb0" => return Err(syn::Error::new(
                                    ident.span(),
                                    format!("Please replace `bit_traversal = \"{thing}\"` with `bit_traversal = \"front\"`"),
                                )),
                                "back" => {
                                    if info.default_endianess.set_reverse_field_order(true).is_some() {
                                        return Err(syn::Error::new(ident.span(), "bit_traversal is defined twice"))
                                    }
                                }
                                "front" => {
                                    if info.default_endianess.set_reverse_field_order(false).is_some() {
                                        return Err(syn::Error::new(ident.span(), "bit_traversal is defined twice"))
                                    }
                                }
                                _ => return Err(Error::new(
                                    val.span(),
                                    "Expected literal str \"lsb\" or \"msb\" for `bit_traversal` attribute.",
                                )),
                            }
                        }
                        "read_from" => {
                            let val =
                                get_lit_str(&value.value, ident, Some("bit_traversal = \"msb\""))?
                                    .value();
                            let thing = val.trim();
                            let replacement = match thing {
                                "lsb" | "lsb0" => "`bit_traversal = \"back\"`",
                                "msb" | "msb0" => "`bit_traversal = \"front\"`",
                                _ => "`bit_traversal = \"<TRAVERSAL_DIRECTION>\"`",
                            };
                            return Err(syn::Error::new(
                                ident.span(),
                                format!("`read_from` has been deprecated, please replace `read_from = {thing}` with {replacement}",),
                            ));
                        }
                        "default_endianness" => {
                            let val = get_lit_str(
                                &value.value,
                                ident,
                                Some("default_endianness = \"big\""),
                            )?;
                            match val.value().as_str() {
                                "le" | "lsb" | "little" | "lil" | "ple" => {
                                    info.default_endianess = Endianness::little_packed();
                                }
                                "ale" => {
                                    info.default_endianess = Endianness::little_aligned();
                                }
                                "be" | "msb" | "big" => {
                                    info.default_endianess = Endianness::big();
                                }
                                "ne" | "native" => {
                                    info.default_endianess = Endianness::nested();
                                }
                                _ => {
                                    return Err(syn::Error::new(
                                        ident.span(),
                                        "invalid `default_endianness`, valid endianess are \"little\", \"big\", and in special cases \"none\"",
                                    ));
                                }
                            }
                        }
                        "enforce_bytes" => {
                            let val = get_lit_int(&value.value, ident, Some("enforce_bytes = 4"))?;
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    info.enforcement =
                                        StructEnforcement::EnforceBitAmount(value * 8);
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        ident.span(),
                                        format!(
                                            "`enforce_bytes` must provide a number that can be parsed as a usize [{err}]"
                                        ),
                                    ))
                                }
                            }
                        }
                        "enforce_bits" => {
                            let val = get_lit_int(&value.value, ident, Some("enforce_bits = 14"))?;
                            match val.base10_parse::<usize>() {
                                        Ok(value) => {
                                            info.enforcement =
                                                StructEnforcement::EnforceBitAmount(value);
                                        }
                                        Err(err) => {
                                            return Err(syn::Error::new(
                                                ident.span(),
                                                format!(
                                                    "`enforce_bits` must provide a number that can be parsed as a usize [{err}]"
                                                ),
                                            ))
                                        }
                                    }
                        }
                        "fill_bits" => {
                            let val = get_lit_int(&value.value, ident, Some("fill_bits = 7"))?;
                            match val.base10_parse::<usize>() {
                                        Ok(value) => {
                                            if info.fill_bits.is_none() {
                                                info.fill_bits = FillBits::Bits(value);
                                            } else {
                                                return Err(syn::Error::new(
                                                    ident.span(),
                                                    "`fill_bits` defined multiple times",
                                                ));
                                            }
                                        }
                                        Err(err) => {
                                            return Err(syn::Error::new(
                                                ident.span(),
                                                format!("`fill_bits` must provide a number that can be parsed as a usize [{err}]"),
                                            ))
                                        }
                                    }
                        }
                        "fill_bytes" => {
                            let val = get_lit_int(&value.value, ident, Some("fill_bytes = 8"))?;
                            match val.base10_parse::<usize>() {
                                        Ok(value) => {
                                            if info.fill_bits.is_none() {
                                                info.fill_bits = FillBits::Bits(value * 8);
                                            } else {
                                                return Err(syn::Error::new(
                                                    ident.span(),
                                                    "`fill_bytes` defined multiple times",
                                                ));
                                            }
                                        }
                                        Err(err) => {
                                            return Err(syn::Error::new(
                                                ident.span(),
                                                format!("`fill_bytes` must provide a number that can be parsed as a usize [{err}]"),
                                            ))
                                        }
                                    }
                        }
                        _ => {
                            return Ok(false);
                        }
                    }
                }
            }
            Meta::Path(ref value) => {
                if let Some(ident) = value.get_ident() {
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        "reverse" => {
                            if info.default_endianess.reverse_byte_order().is_some() {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    "`reverse` defined multiple times",
                                ));
                            }
                        }
                        "dump" => {
                            info.dump = true;
                        }
                        "enforce_full_bytes" => {
                            info.enforcement = StructEnforcement::EnforceFullBytes;
                        }
                        "invalid" => {
                            if is_variant {
                                info.invalid = true;
                            } else {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    "`invalid` attribute can only be used on enum variants",
                                ));
                            }
                        }
                        "fill_bits" => {
                            if info.fill_bits.is_none() {
                                info.fill_bits = FillBits::Auto;
                            } else {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    "`fill_bits` defined multiple times",
                                ));
                            }
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
