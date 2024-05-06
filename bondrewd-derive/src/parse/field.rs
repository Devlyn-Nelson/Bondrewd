use proc_macro2::Span;
use quote::{format_ident, quote};
use std::ops::Range;
use syn::parse::Error;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Ident, Meta, Token, Type};

use crate::common::AttrInfo;
use crate::common::{
    field::{
        Attributes, DataType, DynamicIdent, Info as FieldInfo, NumberSignage, OverlapOptions,
        ReserveFieldOption, SubInfo as SubFieldInfo,
    },
    r#enum::Info as EnumInfo,
    Endianness,
};

use super::{get_lit_int, get_lit_range, get_lit_str};

impl FieldInfo {
    /// `fields` should be all previous fields that have been parsed already.
    pub fn from_syn_field(
        field: &syn::Field,
        fields: &[FieldInfo],
        attrs: &AttrInfo,
    ) -> syn::Result<Self> {
        let ident: DynamicIdent = if let Some(ref name) = field.ident {
            name.clone().into()
        } else {
            (fields.len(), field.span()).into()
            // return Err(Error::new(Span::call_site(), "all fields must be named"));
        };
        // parse all attrs. which will also give us the bit locations
        // NOTE read only attribute assumes that the value should not effect the placement of the rest og
        let last_relevant_field = fields
            .iter()
            .filter(|x| !x.attrs.overlap.is_redundant())
            .last();
        let mut attrs_builder = AttrBuilder::parse(field, last_relevant_field)?;
        // check the field for supported types.
        let data_type = DataType::parse(&field.ty, &mut attrs_builder, &attrs.default_endianess)?;

        let attrs: Attributes = match attrs_builder.try_into() {
            Ok(attr) => attr,
            Err(fix_me) => {
                let mut start = 0;
                if let Some(last_value) = last_relevant_field {
                    start = last_value.attrs.bit_range.end;
                }
                fix_me.fix(start..start + (data_type.size() * 8))
            }
        };

        // construct the field we are parsed.
        let new_field = FieldInfo {
            ident: Box::new(ident),
            ty: data_type,
            attrs,
        };
        // check to verify there are no overlapping bit ranges from previously parsed fields.
        for (i, parsed_field) in fields.iter().enumerate() {
            if parsed_field.overlapping(&new_field) {
                return Err(Error::new(
                    Span::call_site(),
                    format!("fields {} and {} overlap", i, fields.len()),
                ));
            }
        }

        Ok(new_field)
    }
}

impl DataType {
    #[allow(clippy::too_many_lines)]
    pub fn parse(
        ty: &syn::Type,
        attrs: &mut AttrBuilder,
        default_endianess: &Endianness,
    ) -> syn::Result<DataType> {
        let data_type = match ty {
            Type::Path(ref path) => match attrs.ty {
                AttrBuilderType::Struct(size) => DataType::Struct {
                    size,
                    type_quote: if let Some(last_segment) = path.path.segments.last() {
                        let asdf = &last_segment.ident;
                        quote! {#asdf}
                    } else {
                        return Err(syn::Error::new(ty.span(), "field has no Type?"));
                    },
                },
                AttrBuilderType::Enum(size, ref prim) => DataType::Enum {
                    type_quote: quote! {#prim},
                    size,
                    name_quote: if let Some(last_segment) = path.path.segments.last() {
                        let asdf = &last_segment.ident;
                        quote! {#asdf}
                    } else {
                        return Err(syn::Error::new(ty.span(), "field has no Type?"));
                    },
                },
                _ => Self::parse_path(&path.path, attrs)?,
            },
            Type::Array(ref array_path) => {
                // arrays must use a literal for length, because its would be hard any other way.
                let lit_int = get_lit_int(
                    &array_path.len,
                    &Ident::new("array_length", ty.span()),
                    None,
                )?;
                if let Ok(array_length) = lit_int.base10_parse::<usize>() {
                    match attrs.ty {
                        AttrBuilderType::ElementArray(ref element_bit_size, ref sub) => {
                            attrs.bit_range = match std::mem::take(&mut attrs.bit_range) {
                                BuilderRange::Range(ref range) => {
                                    if range.end < range.start {
                                        return Err(syn::Error::new(
                                            ty.span(),
                                            "range end is less than range start",
                                        ));
                                    }
                                    if range.end - range.start != *element_bit_size * array_length {
                                        return Err(
                                                    syn::Error::new(
                                                        ty.span(),
                                                        "Element arrays bit range didn't match (element bit size * array length)"
                                                    )
                                                );
                                    }
                                    BuilderRange::Range(range.clone())
                                }
                                BuilderRange::LastEnd(ref last_end) => BuilderRange::Range(
                                    *last_end..last_end + (array_length * *element_bit_size),
                                ),
                                BuilderRange::None => {
                                    return Err(syn::Error::new(
                                        ty.span(),
                                        "failed getting Range for element array",
                                    ));
                                }
                            };
                            let mut sub_attrs = attrs.clone();
                            if let Type::Array(_) = array_path.elem.as_ref() {
                            } else if let Some(ref ty) = sub.as_ref() {
                                sub_attrs.ty = ty.clone();
                            } else {
                                sub_attrs.ty = AttrBuilderType::None;
                            }
                            let mut sub_ty =
                                Self::parse(&array_path.elem, &mut sub_attrs, default_endianess)?;

                            match sub_ty {
                                DataType::Enum { ref mut size, .. }
                                | DataType::Struct { ref mut size, .. } => {
                                    *size = size.div_ceil(array_length)
                                }
                                _ => {}
                            }

                            let type_ident = &sub_ty.type_quote();
                            DataType::ElementArray {
                                sub_type: Box::new(SubFieldInfo { ty: sub_ty }),
                                length: array_length,
                                type_quote: quote! {[#type_ident;#array_length]},
                            }
                        }
                        AttrBuilderType::BlockArray(_) => {
                            let mut sub_attrs = attrs.clone();
                            if let Type::Array(_) = array_path.elem.as_ref() {
                            } else {
                                sub_attrs.ty = AttrBuilderType::None;
                            }

                            let sub_ty =
                                Self::parse(&array_path.elem, &mut sub_attrs, default_endianess)?;
                            attrs.endianness = sub_attrs.endianness;
                            let type_ident = &sub_ty.type_quote();
                            DataType::BlockArray {
                                sub_type: Box::new(SubFieldInfo { ty: sub_ty }),
                                length: array_length,
                                type_quote: quote! {[#type_ident;#array_length]},
                            }
                        }
                        AttrBuilderType::Enum(_, _) | AttrBuilderType::Struct(_) => {
                            let mut sub_attrs = attrs.clone();
                            if let Type::Array(_) = array_path.elem.as_ref() {
                            } else {
                                sub_attrs.ty = attrs.ty.clone();
                            }

                            let sub_ty =
                                Self::parse(&array_path.elem, &mut sub_attrs, default_endianess)?;
                            attrs.endianness = sub_attrs.endianness;
                            let type_ident = &sub_ty.type_quote();
                            DataType::BlockArray {
                                sub_type: Box::new(SubFieldInfo { ty: sub_ty }),
                                length: array_length,
                                type_quote: quote! {[#type_ident;#array_length]},
                            }
                        }
                        AttrBuilderType::None => {
                            let mut sub_attrs = attrs.clone();
                            if let Type::Array(_) = array_path.elem.as_ref() {
                            } else {
                                sub_attrs.ty = AttrBuilderType::None;
                            }
                            let sub_ty =
                                Self::parse(&array_path.elem, &mut sub_attrs, default_endianess)?;
                            attrs.bit_range = match std::mem::take(&mut attrs.bit_range) {
                                BuilderRange::Range(ref range) => {
                                    if range.end < range.start {
                                        return Err(syn::Error::new(
                                            ty.span(),
                                            "range end is less than range start",
                                        ));
                                    }
                                    if range.end - range.start % array_length != 0 {
                                        return Err(
                                                    syn::Error::new(
                                                        ty.span(),
                                                        "Array Inference failed because given total bit_length does not split up evenly between elements, perhaps try using `element_bit_length` attribute"
                                                    )
                                                );
                                    }
                                    BuilderRange::Range(range.clone())
                                }
                                BuilderRange::LastEnd(ref last_end) => {
                                    let element_bit_length = sub_ty.get_element_bit_length();
                                    BuilderRange::Range(
                                        *last_end..last_end + (array_length * element_bit_length),
                                    )
                                }
                                BuilderRange::None => {
                                    return Err(syn::Error::new(
                                        ty.span(),
                                        "failed getting Range for element array",
                                    ));
                                }
                            };
                            let type_ident = &sub_ty.type_quote();
                            DataType::ElementArray {
                                sub_type: Box::new(SubFieldInfo { ty: sub_ty }),
                                length: array_length,
                                type_quote: quote! {[#type_ident;#array_length]},
                            }
                        }
                    }
                } else {
                    return Err(Error::new(
                        array_path.bracket_token.span.span(),
                        "failed parsing array length as literal integer",
                    ));
                }
            }
            _ => {
                return Err(Error::new(ty.span(), "Unsupported field type"));
            }
        };
        // if the type is a number and its endianess is None (numbers should have endianess) then we
        // apply the structs default (which might also be None)
        if data_type.is_number() && !attrs.endianness.perhaps_endianness(data_type.size()) {
            if default_endianess.has_endianness() {
                attrs.endianness = default_endianess.clone();
            } else if data_type.size() == 1 {
                let mut big = Endianness::big();
                std::mem::swap(&mut attrs.endianness, &mut big);
            } else {
                let mut little = Endianness::little_packed();
                std::mem::swap(&mut attrs.endianness, &mut little);
                // return Err(Error::new(ident.span(), "field without defined endianess found, please set endianess of struct or fields"));
            }
        }

        Ok(data_type)
    }
    #[allow(clippy::too_many_lines)]
    fn parse_path(path: &syn::Path, attrs: &mut AttrBuilder) -> syn::Result<DataType> {
        match attrs.ty {
            AttrBuilderType::None => {
                if let Some(last_segment) = path.segments.last() {
                    let type_quote = &last_segment.ident;
                    let field_type_name = last_segment.ident.to_string();
                    match field_type_name.as_str() {
                        "bool" => match attrs.bit_range {
                            #[allow(clippy::range_plus_one)]
                            BuilderRange::LastEnd(start) => {
                                attrs.bit_range = BuilderRange::Range(start..start + 1);
                                Ok(DataType::Boolean)
                            }
                            _ => Ok(DataType::Boolean),
                        },
                        "u8" => Ok(DataType::Number {
                            size: 1,
                            sign: NumberSignage::Unsigned,
                            type_quote: quote! {#type_quote},
                        }),
                        "i8" => Ok(DataType::Number {
                            size: 1,
                            sign: NumberSignage::Signed,
                            type_quote: quote! {#type_quote},
                        }),
                        "u16" => Ok(DataType::Number {
                            size: 2,
                            sign: NumberSignage::Unsigned,
                            type_quote: quote! {#type_quote},
                        }),
                        "i16" => Ok(DataType::Number {
                            size: 2,
                            sign: NumberSignage::Signed,
                            type_quote: quote! {#type_quote},
                        }),
                        "f32" => {
                            if let BuilderRange::Range(ref span) = attrs.bit_range {
                                if 32 != span.end - span.start {
                                    return Err(syn::Error::new(path.span(), format!("f32 must be full sized, if this is a problem for you open an issue.. provided bit length = {}.", span.end - span.start)));
                                }
                            }
                            Ok(DataType::Float {
                                size: 4,
                                type_quote: quote! {#type_quote},
                            })
                        }
                        "u32" => Ok(DataType::Number {
                            size: 4,
                            sign: NumberSignage::Unsigned,
                            type_quote: quote! {#type_quote},
                        }),
                        "i32" => Ok(DataType::Number {
                            size: 4,
                            sign: NumberSignage::Signed,
                            type_quote: quote! {#type_quote},
                        }),
                        "char" => Ok(DataType::Char {
                            size: 4,
                            type_quote: quote! {#type_quote},
                        }),
                        "f64" => {
                            if let BuilderRange::Range(ref span) = attrs.bit_range {
                                if 64 != span.end - span.start {
                                    return Err(syn::Error::new(path.span(), format!("f64 must be full sized, if this is a problem for you open an issue. provided bit length = {}.", span.end - span.start)));
                                }
                            }
                            Ok(DataType::Float {
                                size: 8,
                                type_quote: quote! {#type_quote},
                            })
                        }
                        "u64" => Ok(DataType::Number {
                            size: 8,
                            sign: NumberSignage::Unsigned,
                            type_quote: quote! {#type_quote},
                        }),
                        "i64" => Ok(DataType::Number {
                            size: 8,
                            sign: NumberSignage::Signed,
                            type_quote: quote! {#type_quote},
                        }),
                        "u128" => Ok(DataType::Number {
                            size: 16,
                            sign: NumberSignage::Unsigned,
                            type_quote: quote! {#type_quote},
                        }),
                        "i128" => Ok(DataType::Number {
                            size: 16,
                            sign: NumberSignage::Signed,
                            type_quote: quote! {#type_quote},
                        }),
                        "usize" | "isize" => Err(Error::new(
                            path.span(),
                            "usize and isize are not supported due to ambiguous sizing".to_string(),
                        )),
                        _ => Ok(DataType::Struct {
                            size: match attrs.bit_range {
                                BuilderRange::Range(ref range) => {
                                    (range.end - range.start).div_ceil(8)
                                }
                                BuilderRange::LastEnd(_) | BuilderRange::None => {
                                    return Err(Error::new(
                                            path.span(),
                                            format!("unknown primitive type. If this type is a Bitfield as well you need to define the bit_length because bondrewd has no way to determine the size of another struct at compile time. [{field_type_name}]"),
                                        ));
                                }
                            },
                            type_quote: quote! {#type_quote},
                        }),
                    }
                } else {
                    Err(syn::Error::new(path.span(), "field has no Type?"))
                }
            }
            AttrBuilderType::Struct(size) => {
                if let Some(ident) = path.get_ident() {
                    Ok(DataType::Struct {
                        size,
                        type_quote: quote! {#ident},
                    })
                } else {
                    Err(syn::Error::new(path.span(), "field has no Type?"))
                }
            }
            AttrBuilderType::Enum(size, ref type_ident) => {
                if let Some(ident) = path.get_ident() {
                    Ok(DataType::Enum {
                        type_quote: quote! {#type_ident},
                        size,
                        name_quote: quote! {#ident},
                    })
                } else {
                    Err(syn::Error::new(path.span(), "field has no Type?"))
                }
            }
            _ => Err(syn::Error::new(
                path.span(),
                "Array did not get detected properly, found Path",
            )),
        }
    }
}

pub struct TryFromAttrBuilderError {
    pub endianness: Box<Endianness>,
    pub reserve: ReserveFieldOption,
    pub overlap: OverlapOptions,
    pub capture_id: bool,
}

impl TryFromAttrBuilderError {
    pub fn fix(self, bit_range: Range<usize>) -> Attributes {
        Attributes {
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
pub enum AttrBuilderType {
    None,
    Struct(usize),
    Enum(usize, Ident),
    // amount of bits for each element.
    ElementArray(usize, Box<Option<AttrBuilderType>>),
    BlockArray(Box<Option<AttrBuilderType>>),
}

#[derive(Clone, Debug)]
pub enum BuilderRange {
    // a range of bits to use.
    Range(std::ops::Range<usize>),
    // used to pass on the last starting location to another part to figure out.
    LastEnd(usize),
    None,
}

impl Default for BuilderRange {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Debug)]
pub struct AttrBuilder {
    /// name is just so we can give better errors
    pub endianness: Endianness,
    pub bit_range: BuilderRange,
    pub ty: AttrBuilderType,
    pub reserve: ReserveFieldOption,
    pub overlap: OverlapOptions,
    /// This should only ever be true when it the first field in a variant
    /// of an enum.
    pub capture_id: bool,
}

impl AttrBuilder {
    fn new() -> Self {
        Self {
            endianness: Endianness::nested(),
            bit_range: BuilderRange::None,
            ty: AttrBuilderType::None,
            reserve: ReserveFieldOption::NotReserve,
            overlap: OverlapOptions::None,
            capture_id: false,
        }
    }

    pub fn parse(field: &syn::Field, last_field: Option<&FieldInfo>) -> syn::Result<AttrBuilder> {
        let mut builder = AttrBuilder::new();
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
        if let BuilderRange::None = builder.bit_range {
            builder.bit_range = BuilderRange::LastEnd(if let Some(last_value) = last_field {
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
                                get_lit_str(&value.value, ident, Some("endianness = \"big\""))?;
                            builder.endianness = match val.value().to_lowercase().as_str() {
                                "le" | "lsb" | "little" | "lil" => Endianness::little_packed(),
                                "ale" | "little-aliened" | "lilali" => Endianness::little_aligned(),
                                "be" | "msb" | "big" => Endianness::big(),
                                "ne" | "native" | "none" => Endianness::nested(),
                                _ => {
                                    return Err(syn::Error::new(
                                        ident.span(),
                                        "unknown endianness try \"little\", \"big\", or \"none\"",
                                    ));
                                }
                            };
                        }
                        "bit_length" => {
                            if let BuilderRange::None = builder.bit_range {
                                let val = get_lit_int(&value.value, ident, Some("bit_length = 8"))?;
                                match val.base10_parse::<usize>() {
                                    Ok(bit_length) => {
                                        let mut start = 0;
                                        if let Some(last_value) = last_field {
                                            start = last_value.attrs.bit_range.end;
                                        }
                                        builder.bit_range =
                                            BuilderRange::Range(start..start + (bit_length));
                                    }
                                    Err(err) => {
                                        return Err(Error::new(
                                            ident.span(),
                                            format!("bit_length must provide a number that can be parsed as a usize. [{err}]"),
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
                            if let BuilderRange::None = builder.bit_range {
                                let val =
                                    get_lit_int(&value.value, ident, Some("byte_length = 4"))?;
                                match val.base10_parse::<usize>() {
                                    Ok(byte_length) => {
                                        let mut start = 0;
                                        if let Some(last_value) = last_field {
                                            start = last_value.attrs.bit_range.end;
                                        }
                                        builder.bit_range =
                                            BuilderRange::Range(start..start + (byte_length * 8));
                                    }
                                    Err(err) => {
                                        return Err(Error::new(
                                            ident.span(),
                                            format!("byte_length must provide a number that can be parsed as a usize. [{err}]"),
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
                            if !matches!(builder.ty, AttrBuilderType::None) {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    "the type of this field is being assigned to twice.",
                                ));
                            }
                            let val =
                                get_lit_str(&value.value, ident, Some("enum_primitive = \"u8\""))?;
                            let mut ty = Some(match val.value().to_lowercase().as_str() {
                                        "u8" => AttrBuilderType::Enum(1, format_ident!("u8")),
                                        "u16" => {
                                            AttrBuilderType::Enum(2, format_ident!("u16"))
                                        }
                                        "u32" => {
                                            AttrBuilderType::Enum(4, format_ident!("u32"))
                                        }
                                        "u64" => {
                                            AttrBuilderType::Enum(8, format_ident!("u64"))
                                        }
                                        "u128" => {
                                            AttrBuilderType::Enum(16, format_ident!("u128"))
                                        }
                                        _ => {
                                            return Err(syn::Error::new(
                                                ident.span(),
                                                "enum_primitive must provide an unsigned integer rust primitive.  example `enum_primitive = \"u8\"`",
                                            ))
                                        }
                                    });
                            match builder.ty {
                                AttrBuilderType::BlockArray(ref mut sub_ty)
                                | AttrBuilderType::ElementArray(_, ref mut sub_ty) => {
                                    std::mem::swap(&mut ty, sub_ty);
                                }
                                _ => {
                                    builder.ty = ty.unwrap();
                                }
                            }
                        }
                        "struct_size" => {
                            if !matches!(builder.ty, AttrBuilderType::None) {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    "the type of this field is being assigned to twice.",
                                ));
                            }
                            let val = get_lit_int(&value.value, ident, Some("struct_size = 4"))?;
                            let mut ty = Some(match val.base10_parse::<usize>() {
                                Ok(byte_length) => AttrBuilderType::Struct(byte_length),
                                Err(err) => {
                                    return Err(Error::new(
                                        ident.span(),
                                        format!("struct_size must provided a number that can be parsed as a usize. [{err}]"),
                                    ));
                                }
                            });
                            match builder.ty {
                                AttrBuilderType::BlockArray(ref mut sub_ty)
                                | AttrBuilderType::ElementArray(_, ref mut sub_ty) => {
                                    std::mem::swap(&mut ty, sub_ty.as_mut());
                                }
                                _ => {
                                    builder.ty = ty.unwrap();
                                }
                            }
                        }
                        "bits" => {
                            if !matches!(builder.bit_range, BuilderRange::None) {
                                return Err(Error::new(
                                    ident.span(),
                                    "bit_range for field was defined twice",
                                ));
                            }
                            if let Some(val) = get_lit_range(&value.value, ident)? {
                                builder.bit_range = BuilderRange::Range(val);
                            } else if let Ok(val) = get_lit_str(&value.value, ident, None) {
                                let val_string = val.value();
                                let split = val_string.split("..").collect::<Vec<&str>>();
                                if split.len() == 2 {
                                    match (split[0].parse::<usize>(), split[1].parse::<usize>()) {
                                        (Ok(start), Ok(end)) => {
                                            builder.bit_range = BuilderRange::Range(start..end);
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
                                get_lit_int(&value.value, ident, Some("element_bit_length = 10"))?;

                            match val.base10_parse::<usize>() {
                                Ok(bit_length) => {
                                    builder.bit_range = match std::mem::take(&mut builder.bit_range) {
                                    BuilderRange::None => {
                                        builder.ty = match builder.ty {
                                            AttrBuilderType::Struct(_) |
                                            AttrBuilderType::Enum(_, _) => {
                                                AttrBuilderType::ElementArray(bit_length, Box::new(Some(builder.ty.clone())))
                                            }
                                            _ => AttrBuilderType::ElementArray(bit_length, Box::new(None)),
                                        };
                                        if let Some(last_value) = last_field {
                                            BuilderRange::LastEnd(last_value.attrs.bit_range.end)
                                        }else{
                                            BuilderRange::LastEnd(0)
                                        }
                                    }
                                    BuilderRange::Range(range) => {
                                        builder.ty = match builder.ty {
                                            AttrBuilderType::Struct(_) |
                                            AttrBuilderType::Enum(_, _) => {
                                                AttrBuilderType::ElementArray(bit_length, Box::new(Some(builder.ty.clone())))
                                            }
                                            _ => AttrBuilderType::ElementArray(bit_length, Box::new(None)),
                                        };
                                        BuilderRange::Range(range)
                                    }
                                    BuilderRange::LastEnd(_) => return Err(Error::new(
                                        ident.span(),
                                        "found Field bit range no_end while element_bit_length attribute which should never happen",
                                    )),
                                };
                                }
                                Err(err) => {
                                    return Err(Error::new(
                                    ident.span(),
                                    format!("bit_length must provide a number that can be parsed as a usize [{err}]"),
                                ));
                                }
                            }
                        }
                        "element_byte_length" => {
                            let val =
                                get_lit_int(&value.value, ident, Some("element_byte_length = 2"))?;
                            match val.base10_parse::<usize>() {
                                Ok(byte_length) => {
                                    builder.bit_range = match std::mem::take(&mut builder.bit_range) {
                                        BuilderRange::None => {
                                            builder.ty = match builder.ty {
                                                AttrBuilderType::Struct(_) |
                                                AttrBuilderType::Enum(_, _) => {
                                                    AttrBuilderType::ElementArray(byte_length * 8, Box::new(Some(builder.ty.clone())))
                                                }
                                                _ => AttrBuilderType::ElementArray(byte_length * 8, Box::new(None)),
                                            };
                                            if let Some(last_value) = last_field {
                                                BuilderRange::LastEnd(last_value.attrs.bit_range.end)
                                            }else{
                                                BuilderRange::LastEnd(0)
                                            }
                                        }
                                        BuilderRange::Range(range) => {
                                            builder.ty = match builder.ty {
                                                AttrBuilderType::Struct(_) |
                                                AttrBuilderType::Enum(_, _) => {
                                                    AttrBuilderType::ElementArray(byte_length * 8, Box::new(Some(builder.ty.clone())))
                                                }
                                                _ => AttrBuilderType::ElementArray(byte_length * 8, Box::new(None)),
                                            };
                                            BuilderRange::Range(range)
                                        }
                                        BuilderRange::LastEnd(_) => return Err(Error::new(
                                            ident.span(),
                                            "found Field bit range no_end while element_byte_length attribute which should never happen",
                                        )),
                                    };
                                }
                                Err(err) => {
                                    return Err(Error::new(
                                        ident.span(),
                                        format!("byte_length must provide a number that can be parsed as a usize [{err}]"),
                                    ));
                                }
                            }
                        }
                        "block_bit_length" => {
                            let val =
                                get_lit_int(&value.value, ident, Some("block_bit_length = 14"))?;
                            match val.base10_parse::<usize>() {
                                Ok(bit_length) => {
                                    builder.bit_range = match std::mem::take(&mut builder.bit_range) {
                                            BuilderRange::None => {
                                                builder.ty = match builder.ty {
                                                    AttrBuilderType::Struct(_) |
                                                    AttrBuilderType::Enum(_, _) => {
                                                        AttrBuilderType::BlockArray(Box::new(Some(builder.ty.clone())))
                                                    }
                                                    _ => AttrBuilderType::BlockArray(Box::new(None)),
                                                };
                                                if let Some(last_value) = last_field {
                                                    BuilderRange::Range(last_value.attrs.bit_range.end..last_value.attrs.bit_range.end + (bit_length))
                                                }else{
                                                    BuilderRange::Range(0..bit_length)
                                                }
                                            }
                                            BuilderRange::Range(range) => {
                                                builder.ty = match builder.ty {
                                                    AttrBuilderType::Struct(_) |
                                                    AttrBuilderType::Enum(_, _) => {
                                                        AttrBuilderType::BlockArray(Box::new(Some(builder.ty.clone())))
                                                    }
                                                    _ => AttrBuilderType::BlockArray(Box::new(None)),
                                                };
                                                if range.end - range.start == bit_length{
                                                BuilderRange::Range(range)
                                                }else{
                                                    return Err(Error::new(
                                                        ident.span(),
                                                        "size of bit_range provided by (bits, bit_length, or byte_length) does not match array_bit_length",
                                                    ));
                                                }
                                            }
                                            BuilderRange::LastEnd(_) => return Err(Error::new(
                                                    ident.span(),
                                                    "found Field bit range no-end while array_bit_length attribute which should never happen",
                                                )),
                                        };
                                }
                                Err(err) => {
                                    return Err(Error::new(
                                            ident.span(),
                                            format!("array_bit_length must provide a number that can be parsed as a usize [{err}]"),
                                        ));
                                }
                            }
                        }
                        "block_byte_length" => {
                            let val =
                                get_lit_int(&value.value, ident, Some("block_bit_length = 14"))?;
                            match val.base10_parse::<usize>() {
                                Ok(byte_length) => {
                                    builder.bit_range = match std::mem::take(&mut builder.bit_range) {
                                            BuilderRange::None => {
                                                builder.ty = match builder.ty {
                                                    AttrBuilderType::Struct(_) |
                                                    AttrBuilderType::Enum(_, _) => {
                                                        AttrBuilderType::BlockArray(Box::new(Some(builder.ty.clone())))
                                                    }
                                                    _ => AttrBuilderType::BlockArray(Box::new(None)),
                                                };
                                                if let Some(last_value) = last_field {
                                                    BuilderRange::Range(last_value.attrs.bit_range.end..last_value.attrs.bit_range.end + (byte_length * 8))
                                                }else{
                                                    BuilderRange::Range(0..byte_length*8)
                                                }
                                            }
                                            BuilderRange::Range(range) => {
                                                builder.ty = match builder.ty {
                                                    AttrBuilderType::Struct(_) |
                                                    AttrBuilderType::Enum(_, _) => {
                                                        AttrBuilderType::BlockArray(Box::new(Some(builder.ty.clone())))
                                                    }
                                                    _ => AttrBuilderType::BlockArray(Box::new(None)),
                                                };
                                                if range.end - range.start == byte_length * 8{
                                                BuilderRange::Range(range)
                                                }else{
                                                    return Err(Error::new(
                                                        ident.span(),
                                                        "size of bit_range provided by (bits, bit_length, or byte_length) does not match array_byte_length",
                                                    ));
                                                }
                                            }
                                            BuilderRange::LastEnd(_) => return Err(Error::new(
                                                ident.span(),
                                                "found Field bit range no-end while array_byte_length attribute which should never happen",
                                            )),
                                        };
                                }
                                Err(err) => {
                                    return Err(Error::new(
                                            ident.span(),
                                            format!("array_byte_length must provide a number that can be parsed as a usize [{err}]"),
                                        ));
                                }
                            }
                        }
                        "overlapping_bits" => {
                            let val =
                                get_lit_int(&value.value, ident, Some("block_bit_length = 14"))?;
                            match val.base10_parse::<usize>() {
                                Ok(bits) => builder.overlap = OverlapOptions::Allow(bits),
                                Err(err) => {
                                    return Err(Error::new(
                                            ident.span(),
                                            format!("overlapping_bits must provided a number that can be parsed as a usize [{err}]"),
                                        ));
                                }
                            };
                        }
                        _ => {
                            if ident_as_str.as_str() != "doc" {
                                return Err(Error::new(
                                    ident.span(),
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
                        "read_only" => {
                            builder.reserve = ReserveFieldOption::ReadOnly;
                        }
                        "capture_id" => {
                            if let Some(lf) = last_field {
                                if lf.ident().name() == EnumInfo::VARIANT_ID_NAME {
                                    builder.capture_id = true;
                                } else {
                                    return Err(syn::Error::new(ident.span(), "capture_id shall only be used on the first field of a enum variant."));
                                }
                            } else {
                                return Err(syn::Error::new(ident.span(), "capture_id shall only be used on the first field of a enum variant."));
                            }
                        }
                        "redundant" => {
                            builder.overlap = OverlapOptions::Redundant;
                            builder.reserve = ReserveFieldOption::ReadOnly;
                        }
                        _ => {
                            if ident_str.as_str() != "doc" {
                                return Err(Error::new(
                                    ident.span(),
                                    format!("\"{ident_str}\" is not a valid attribute"),
                                ));
                            }
                        }
                    }
                }
            }
            Meta::List(meta_list) => {
                return Err(syn::Error::new(
                    meta_list.span(),
                    "bondrewd does not offer any list attribute for fields",
                ))
            }
        }
        Ok(())
    }
}

impl TryInto<Attributes> for AttrBuilder {
    type Error = TryFromAttrBuilderError;
    fn try_into(self) -> std::result::Result<Attributes, Self::Error> {
        if let BuilderRange::Range(bit_range) = self.bit_range {
            Ok(Attributes {
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
