pub mod common;
#[cfg(feature = "setters")]
pub mod struct_fns;

use proc_macro2::Span;
use quote::format_ident;
use std::ops::Range;
use syn::parse::Error;
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, Lit, LitInt, LitStr, Meta, Token};

use crate::parse::common::{Endianness, FieldAttrs, FieldInfo, ReserveFieldOption};

use common::OverlapOptions;

pub struct TryFromAttrBuilderError {
    pub endianness: Box<Endianness>,
    pub reserve: ReserveFieldOption,
    pub overlap: OverlapOptions,
    pub capture_id: bool,
}

impl TryFromAttrBuilderError {
    pub fn fix(self, bit_range: Range<usize>) -> FieldAttrs {
        FieldAttrs {
            endianness: self.endianness,
            bit_range,
            reserve: self.reserve,
            overlap: self.overlap,
            capture_id: self.capture_id,
        }
    }
}

impl std::fmt::Display for TryFromAttrBuilderError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "Did not provide enough information to determine bit_length"
        )
    }
}

#[derive(Clone, Debug)]
pub enum FieldAttrBuilderType {
    None,
    Struct(usize),
    Enum(usize, Ident),
    // amount of bits for each element.
    ElementArray(usize, Box<Option<FieldAttrBuilderType>>),
    BlockArray(Box<Option<FieldAttrBuilderType>>),
}

#[derive(Clone, Debug)]
pub enum FieldBuilderRange {
    // a range of bits to use.
    Range(std::ops::Range<usize>),
    // used to pass on the last starting location to another part to figure out.
    LastEnd(usize),
    None,
}

impl Default for FieldBuilderRange {
    fn default() -> Self {
        Self::None
    }
}

fn get_lit_str<'a>(
    expr: &'a Expr,
    ident: &Ident,
    example: Option<&str>,
) -> syn::Result<&'a LitStr> {
    let example = if let Some(ex) = example {
        format!("example: `{ex}`")
    } else {
        String::new()
    };
    if let Expr::Lit(ref lit) = expr {
        if let Lit::Str(ref val) = lit.lit {
            Ok(val)
        } else {
            return Err(syn::Error::new(
                ident.span(),
                format!("{ident} requires a integer literal. {example}"),
            ));
        }
    } else {
        return Err(syn::Error::new(
            ident.span(),
            format!("{ident} requires a integer literal. {example}"),
        ));
    }
}

fn get_lit_int<'a>(
    expr: &'a Expr,
    ident: &Ident,
    example: Option<&str>,
) -> syn::Result<&'a LitInt> {
    let example = if let Some(ex) = example {
        format!("example: `{ex}`")
    } else {
        String::new()
    };
    if let Expr::Lit(ref lit) = expr {
        if let Lit::Int(ref val) = lit.lit {
            Ok(val)
        } else {
            return Err(syn::Error::new(
                ident.span(),
                format!("{ident} requires a string literal. {example}"),
            ));
        }
    } else {
        return Err(syn::Error::new(
            ident.span(),
            format!("{ident} requires a string literal. {example}"),
        ));
    }
}

fn get_lit_range<'a>(expr: &'a Expr, ident: &Ident) -> syn::Result<Option<Range<usize>>> {
    if let Expr::Range(ref lit) = expr {
        let start = if let Some(ref v) = lit.start {
            if let Expr::Lit(ref el) = v.as_ref() {
                if let Lit::Int(ref i) = el.lit {
                    i.base10_parse()?
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("start of range must be an integer."),
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("start of range must be an integer literal."),
                ));
            }
        } else {
            return Err(syn::Error::new(
                ident.span(),
                "range for bits must define a start",
            ));
        };
        let end = if let Some(ref v) = lit.end {
            if let Expr::Lit(ref el) = v.as_ref() {
                if let Lit::Int(ref i) = el.lit {
                    i.base10_parse()?
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("end of range must be an integer."),
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("end of range must be an integer literal."),
                ));
            }
        } else {
            return Err(syn::Error::new(
                ident.span(),
                "range for bits must define a end",
            ));
        };
        Ok(Some(match lit.limits {
            syn::RangeLimits::HalfOpen(_) => start..end,
            syn::RangeLimits::Closed(_) => start..end + 1,
        }))
    } else {
        Ok(None)
    }
}

#[derive(Clone, Debug)]
pub struct FieldAttrBuilder {
    /// name is just so we can give better errors
    span: Span,
    pub endianness: Endianness,
    pub bit_range: FieldBuilderRange,
    pub ty: FieldAttrBuilderType,
    pub reserve: ReserveFieldOption,
    pub overlap: OverlapOptions,
    /// This should only ever be true when it the first field in a variant
    /// of an enum.
    pub capture_id: bool,
}

impl FieldAttrBuilder {
    fn new(span: Span) -> Self {
        Self {
            span,
            endianness: Endianness::None,
            bit_range: FieldBuilderRange::None,
            ty: FieldAttrBuilderType::None,
            reserve: ReserveFieldOption::NotReserve,
            overlap: OverlapOptions::None,
            capture_id: false,
        }
    }

    fn span(&self) -> Span {
        self.span
    }

    pub fn parse(
        field: &syn::Field,
        last_field: Option<&FieldInfo>,
        span: Span,
    ) -> syn::Result<FieldAttrBuilder> {
        let mut builder = FieldAttrBuilder::new(span);
        // we are just looking for attrs that can fill in the details in the builder variable above
        // sometimes having the last field is useful for example the bit range the builder wants could be
        // filled in using the end of the previous field as the start, add the length in bits you get the
        // end ( this only works if a all bit fields are in order, ex. if a bit_range attribute defines a
        // complete range which occupies the same space as this field and that field is not the "last_field"
        // you will get a conflicting fields error returned to the user... hopefully )
        for attr in &field.attrs {
            // let meta = attr.parse_meta()?;
            if attr.path().is_ident("bondrewd") {
                let nested =
                    attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                for meta in &nested {
                    Self::parse_meta(meta, last_field, &mut builder)?;
                }
            }
        }
        if let FieldBuilderRange::None = builder.bit_range {
            builder.bit_range = FieldBuilderRange::LastEnd(if let Some(last_value) = last_field {
                last_value.attrs.bit_range.end
            } else {
                0
            });
        }

        Ok(builder)
    }
    fn parse_meta(
        meta: &Meta,
        last_field: Option<&FieldInfo>,
        builder: &mut Self,
    ) -> syn::Result<()> {
        match meta {
            Meta::NameValue(value) => {
                if let Some(ident) = value.path.get_ident() {
                    let ident_as_str = ident.to_string();
                    match ident_as_str.as_str() {
                        "endianness" => {
                            let val =
                                get_lit_str(&value.value, &ident, Some("endianness = \"big\""))?;
                            builder.endianness = match val.value().to_lowercase().as_str() {
                                "le" | "lsb" | "little" | "lil" => Endianness::Little,
                                "be" | "msb" | "big" => Endianness::Big,
                                "ne" | "native" | "none" => Endianness::None,
                                _ => {
                                    return Err(syn::Error::new(
                                        ident.span(),
                                        "unknown endianness try \"little\", \"big\", or \"none\"",
                                    ));
                                }
                            };
                        }
                        "bit_length" => {
                            if let FieldBuilderRange::None = builder.bit_range {
                                let val =
                                    get_lit_int(&value.value, &ident, Some("bit_length = 8"))?;
                                match val.base10_parse::<usize>() {
                                    Ok(bit_length) => {
                                        let mut start = 0;
                                        if let Some(last_value) = last_field {
                                            start = last_value.attrs.bit_range.end;
                                        }
                                        builder.bit_range =
                                            FieldBuilderRange::Range(start..start + (bit_length));
                                    }
                                    Err(err) => {
                                        return Err(Error::new(
                                            ident.span(),
                                            format!("bit_length must be a number that can be parsed as a usize. [{err}]"),
                                        ));
                                    }
                                }
                            } else {
                                return Err(Error::new(
                                    ident.span(),
                                    "bit_length is being defined twice for this field",
                                ));
                            }
                        }
                        "byte_length" => {
                            if let FieldBuilderRange::None = builder.bit_range {
                                let val =
                                    get_lit_int(&value.value, &ident, Some("byte_length = 4"))?;
                                match val.base10_parse::<usize>() {
                                    Ok(byte_length) => {
                                        let mut start = 0;
                                        if let Some(last_value) = last_field {
                                            start = last_value.attrs.bit_range.end;
                                        }
                                        builder.bit_range = FieldBuilderRange::Range(
                                            start..start + (byte_length * 8),
                                        );
                                    }
                                    Err(err) => {
                                        return Err(Error::new(
                                            ident.span(),
                                            format!("byte_length must be a number that can be parsed as a usize. [{err}]"),
                                        ));
                                    }
                                }
                            } else {
                                return Err(Error::new(
                                    ident.span(),
                                    "byte_length is being defined twice for this field",
                                ));
                            }
                        }
                        "enum_primitive" => {
                            if !matches!(builder.ty, FieldAttrBuilderType::None) {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    "the type of this field is being assigned to twice.",
                                ));
                            }
                            let val =
                                get_lit_str(&value.value, &ident, Some("enum_primitive = \"u8\""))?;
                            let mut ty = Some(match val.value().to_lowercase().as_str() {
                                        "u8" => FieldAttrBuilderType::Enum(1, format_ident!("u8")),
                                        "u16" => {
                                            FieldAttrBuilderType::Enum(2, format_ident!("u16"))
                                        }
                                        "u32" => {
                                            FieldAttrBuilderType::Enum(4, format_ident!("u32"))
                                        }
                                        "u64" => {
                                            FieldAttrBuilderType::Enum(8, format_ident!("u64"))
                                        }
                                        "u128" => {
                                            FieldAttrBuilderType::Enum(16, format_ident!("u128"))
                                        }
                                        _ => {
                                            return Err(syn::Error::new(
                                                ident.span(),
                                                "enum_primitive must be an unsigned integer rust primitive.  example `enum_primitive = \"u8\"`",
                                            ))
                                        }
                                    });
                            match builder.ty {
                                FieldAttrBuilderType::BlockArray(ref mut sub_ty)
                                | FieldAttrBuilderType::ElementArray(_, ref mut sub_ty) => {
                                    std::mem::swap(&mut ty, sub_ty);
                                }
                                _ => {
                                    builder.ty = ty.unwrap();
                                }
                            }
                        }
                        "struct_size" => {
                            if !matches!(builder.ty, FieldAttrBuilderType::None) {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    "the type of this field is being assigned to twice.",
                                ));
                            }
                            let val = get_lit_int(&value.value, &ident, Some("struct_size = 4"))?;
                            let mut ty = Some(match val.base10_parse::<usize>() {
                                Ok(byte_length) => FieldAttrBuilderType::Struct(byte_length),
                                Err(err) => {
                                    return Err(Error::new(
                                        ident.span(),
                                        format!("struct_size must provided a number that can be parsed as a usize. [{err}]"),
                                    ));
                                }
                            });
                            match builder.ty {
                                FieldAttrBuilderType::BlockArray(ref mut sub_ty)
                                | FieldAttrBuilderType::ElementArray(_, ref mut sub_ty) => {
                                    std::mem::swap(&mut ty, sub_ty.as_mut());
                                }
                                _ => {
                                    builder.ty = ty.unwrap();
                                }
                            }
                        }
                        "bits" => {
                            if !matches!(builder.bit_range, FieldBuilderRange::None) {
                                return Err(Error::new(
                                    ident.span(),
                                    "bit-range for field was defined twice",
                                ));
                            }
                            if let Some(val) = get_lit_range(&value.value, &ident)? {
                                builder.bit_range = FieldBuilderRange::Range(val);
                            } else if let Ok(val) = get_lit_str(&value.value, &ident, None) {
                                let val_string = val.value();
                                let split = val_string.split("..").collect::<Vec<&str>>();
                                if split.len() == 2 {
                                    match (split[0].parse::<usize>(), split[1].parse::<usize>()) {
                                        (Ok(start), Ok(end)) => {
                                            builder.bit_range =
                                                FieldBuilderRange::Range(start..end);
                                        }
                                        (Ok(_), Err(_)) => {
                                            if split[1].contains('=') {
                                                return Err(Error::new(
                                                        ident.span(),
                                                        "string literals for bits range has been deprecated, remove quotes from range.",
                                                    ));
                                            }
                                            return Err(Error::new(
                                                ident.span(),
                                                "failed paring ending index for range",
                                            ));
                                        }
                                        (Err(_), Ok(_)) => {
                                            return Err(Error::new(
                                                ident.span(),
                                                "failed paring starting index for range",
                                            ));
                                        }
                                        _ => {
                                            return Err(Error::new(
                                                ident.span(),
                                                "failed paring range",
                                            ));
                                        }
                                    }
                                }
                            } else {
                                return Err(Error::new(
                                    ident.span(),
                                    "bits must provided a range literal. example `bits = 0..2`",
                                ));
                            }
                        }
                        "element_bit_length" => {
                            let val =
                                get_lit_int(&value.value, &ident, Some("element_bit_length = 10"))?;

                            match val.base10_parse::<usize>() {
                                Ok(bit_length) => {
                                    builder.bit_range = match std::mem::take(&mut builder.bit_range) {
                                    FieldBuilderRange::None => {
                                        builder.ty = match builder.ty {
                                            FieldAttrBuilderType::Struct(_) |
                                            FieldAttrBuilderType::Enum(_, _) => {
                                                FieldAttrBuilderType::ElementArray(bit_length, Box::new(Some(builder.ty.clone())))
                                            }
                                            _ => FieldAttrBuilderType::ElementArray(bit_length, Box::new(None)),
                                        };
                                        if let Some(last_value) = last_field {
                                            FieldBuilderRange::LastEnd(last_value.attrs.bit_range.end)
                                        }else{
                                            FieldBuilderRange::LastEnd(0)
                                        }
                                    }
                                    FieldBuilderRange::Range(range) => {
                                        builder.ty = match builder.ty {
                                            FieldAttrBuilderType::Struct(_) |
                                            FieldAttrBuilderType::Enum(_, _) => {
                                                FieldAttrBuilderType::ElementArray(bit_length, Box::new(Some(builder.ty.clone())))
                                            }
                                            _ => FieldAttrBuilderType::ElementArray(bit_length, Box::new(None)),
                                        };
                                        FieldBuilderRange::Range(range)
                                    }
                                    FieldBuilderRange::LastEnd(_) => return Err(Error::new(
                                        builder.span(),
                                        "found Field bit range no_end while element-bit-length attribute which should never happen",
                                    )),
                                };
                                }
                                Err(err) => {
                                    return Err(Error::new(
                                    builder.span(),
                                    format!("bit_length must be a number that can be parsed as a usize [{err}]"),
                                ));
                                }
                            }
                        }
                        "element_byte_length" => {
                            let val =
                                get_lit_int(&value.value, &ident, Some("element_byte_length = 2"))?;
                            match val.base10_parse::<usize>() {
                                Ok(byte_length) => {
                                    builder.bit_range = match std::mem::take(&mut builder.bit_range) {
                                        FieldBuilderRange::None => {
                                            builder.ty = match builder.ty {
                                                FieldAttrBuilderType::Struct(_) |
                                                FieldAttrBuilderType::Enum(_, _) => {
                                                    FieldAttrBuilderType::ElementArray(byte_length * 8, Box::new(Some(builder.ty.clone())))
                                                }
                                                _ => FieldAttrBuilderType::ElementArray(byte_length * 8, Box::new(None)),
                                            };
                                            if let Some(last_value) = last_field {
                                                FieldBuilderRange::LastEnd(last_value.attrs.bit_range.end)
                                            }else{
                                                FieldBuilderRange::LastEnd(0)
                                            }
                                        }
                                        FieldBuilderRange::Range(range) => {
                                            builder.ty = match builder.ty {
                                                FieldAttrBuilderType::Struct(_) |
                                                FieldAttrBuilderType::Enum(_, _) => {
                                                    FieldAttrBuilderType::ElementArray(byte_length * 8, Box::new(Some(builder.ty.clone())))
                                                }
                                                _ => FieldAttrBuilderType::ElementArray(byte_length * 8, Box::new(None)),
                                            };
                                            FieldBuilderRange::Range(range)
                                        }
                                        FieldBuilderRange::LastEnd(_) => return Err(Error::new(
                                            builder.span(),
                                            "found Field bit range no_end while element-byte-length attribute which should never happen",
                                        )),
                                    };
                                }
                                Err(err) => {
                                    return Err(Error::new(
                                        builder.span(),
                                        format!("byte_length must be a number that can be parsed as a usize [{err}]"),
                                    ));
                                }
                            }
                        }
                        "block_bit_length" => {
                            let val =
                                get_lit_int(&value.value, &ident, Some("block_bit_length = 14"))?;
                            match val.base10_parse::<usize>() {
                                Ok(bit_length) => {
                                    builder.bit_range = match std::mem::take(&mut builder.bit_range) {
                                            FieldBuilderRange::None => {
                                                builder.ty = match builder.ty {
                                                    FieldAttrBuilderType::Struct(_) |
                                                    FieldAttrBuilderType::Enum(_, _) => {
                                                        FieldAttrBuilderType::BlockArray(Box::new(Some(builder.ty.clone())))
                                                    }
                                                    _ => FieldAttrBuilderType::BlockArray(Box::new(None)),
                                                };
                                                if let Some(last_value) = last_field {
                                                    FieldBuilderRange::Range(last_value.attrs.bit_range.end..last_value.attrs.bit_range.end + (bit_length))
                                                }else{
                                                    FieldBuilderRange::Range(0..bit_length)
                                                }
                                            }
                                            FieldBuilderRange::Range(range) => {
                                                builder.ty = match builder.ty {
                                                    FieldAttrBuilderType::Struct(_) |
                                                    FieldAttrBuilderType::Enum(_, _) => {
                                                        FieldAttrBuilderType::BlockArray(Box::new(Some(builder.ty.clone())))
                                                    }
                                                    _ => FieldAttrBuilderType::BlockArray(Box::new(None)),
                                                };
                                                if range.end - range.start == bit_length{
                                                FieldBuilderRange::Range(range)
                                                }else{
                                                    return Err(Error::new(
                                                        builder.span(),
                                                        "size of bit-range provided by (bits, bit-length, or byte-length) does not match array-bit-length",
                                                    ));
                                                }
                                            }
                                            FieldBuilderRange::LastEnd(_) => return Err(Error::new(
                                                    builder.span(),
                                                    "found Field bit range no-end while array-bit-length attribute which should never happen",
                                                )),
                                        };
                                }
                                Err(err) => {
                                    return Err(Error::new(
                                            builder.span(),
                                            format!("array-bit-length must be a number that can be parsed as a usize [{err}]"),
                                        ));
                                }
                            }
                        }
                        "block_byte_length" => {
                            let val =
                                get_lit_int(&value.value, &ident, Some("block_bit_length = 14"))?;
                            match val.base10_parse::<usize>() {
                                Ok(byte_length) => {
                                    builder.bit_range = match std::mem::take(&mut builder.bit_range) {
                                            FieldBuilderRange::None => {
                                                builder.ty = match builder.ty {
                                                    FieldAttrBuilderType::Struct(_) |
                                                    FieldAttrBuilderType::Enum(_, _) => {
                                                        FieldAttrBuilderType::BlockArray(Box::new(Some(builder.ty.clone())))
                                                    }
                                                    _ => FieldAttrBuilderType::BlockArray(Box::new(None)),
                                                };
                                                if let Some(last_value) = last_field {
                                                    FieldBuilderRange::Range(last_value.attrs.bit_range.end..last_value.attrs.bit_range.end + (byte_length * 8))
                                                }else{
                                                    FieldBuilderRange::Range(0..byte_length*8)
                                                }
                                            }
                                            FieldBuilderRange::Range(range) => {
                                                builder.ty = match builder.ty {
                                                    FieldAttrBuilderType::Struct(_) |
                                                    FieldAttrBuilderType::Enum(_, _) => {
                                                        FieldAttrBuilderType::BlockArray(Box::new(Some(builder.ty.clone())))
                                                    }
                                                    _ => FieldAttrBuilderType::BlockArray(Box::new(None)),
                                                };
                                                if range.end - range.start == byte_length * 8{
                                                FieldBuilderRange::Range(range)
                                                }else{
                                                    return Err(Error::new(
                                                        builder.span(),
                                                        "size of bit-range provided by (bits, bit-length, or byte_length) does not match array-byte-length",
                                                    ));
                                                }
                                            }
                                            FieldBuilderRange::LastEnd(_) => return Err(Error::new(
                                                builder.span(),
                                                "found Field bit range no-end while array-byte-length attribute which should never happen",
                                            )),
                                        };
                                }
                                Err(err) => {
                                    return Err(Error::new(
                                            builder.span(),
                                            format!("array-byte-length must be a number that can be parsed as a usize [{err}]"),
                                        ));
                                }
                            }
                        }
                        "overlapping_bits" => {
                            let val =
                                get_lit_int(&value.value, &ident, Some("block_bit_length = 14"))?;
                                    match val.base10_parse::<usize>() {
                                        Ok(bits) => builder.overlap = OverlapOptions::Allow(bits),
                                        Err(err) => {
                                            return Err(Error::new(
                                            builder.span(),
                                            format!("overlapping-bits must provided a number that can be parsed as a usize [{err}]"),
                                        ));
                                        }
                                    };
                        }
                        _ => {
                            if ident_as_str.as_str() != "doc" {
                                return Err(Error::new(
                                    builder.span(),
                                    format!("\"{ident_as_str}\" is not a valid attribute"),
                                ));
                            }
                        }
                    }
                }
            }
            Meta::Path(path) => {
                if let Some(ident) = path.get_ident() {
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        "reserve" => {
                            builder.reserve = ReserveFieldOption::ReserveField;
                        }
                        "read_only" | "read-only" => {
                            builder.reserve = ReserveFieldOption::ReadOnly;
                        }
                        "capture_id" | "capture-id" => {
                            builder.capture_id = true;
                        }
                        // TODO  can not enable this until i figure out a way to express exactly the amount
                        // of overlapping bits.
                        /*"allow_overlap" => {
                            builder.overlap = OverlapOptions::Allow;
                        }*/
                        "redundant" => {
                            builder.overlap = OverlapOptions::Redundant;
                            builder.reserve = ReserveFieldOption::ReadOnly;
                        }
                        _ => {
                            if ident_str.as_str() != "doc" {
                                return Err(Error::new(
                                    builder.span(),
                                    format!("\"{ident_str}\" is not a valid attribute"),
                                ));
                            }
                        }
                    }
                }
            }
            Meta::List(_meta_list) => {
                // if meta_list.path.is_ident("bondrewd") {
                //     for nested_meta in meta_list.nested {
                //         match nested_meta {
                //             NestedMeta::Meta(meta) => {
                //                 Self::parse_meta(meta, last_field, builder)?;
                //             }
                //             NestedMeta::Lit(_) => {}
                //         }
                //     }
                // }
            }
        }
        Ok(())
    }
}

impl TryInto<FieldAttrs> for FieldAttrBuilder {
    type Error = TryFromAttrBuilderError;
    fn try_into(self) -> std::result::Result<FieldAttrs, Self::Error> {
        if let FieldBuilderRange::Range(bit_range) = self.bit_range {
            Ok(FieldAttrs {
                endianness: Box::new(self.endianness),
                bit_range,
                reserve: self.reserve,
                overlap: self.overlap,
                capture_id: self.capture_id,
            })
        } else {
            Err(TryFromAttrBuilderError {
                endianness: Box::new(self.endianness),
                reserve: self.reserve,
                overlap: self.overlap,
                capture_id: self.capture_id,
            })
        }
    }
}
