use crate::structs::parse::{
    FieldAttrBuilder, FieldAttrBuilderType, FieldBuilderRange, TryFromAttrBuilderError,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::ops::Range;
use std::str::FromStr;
use syn::parse::Error;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Expr, Fields, Ident, Lit, Meta, Token, Type};

/// Returns a u8 mask with provided `num` amount of 1's on the left side (most significant bit)
pub fn get_left_and_mask(num: usize) -> u8 {
    match num {
        8 => 0b1111_1111,
        7 => 0b1111_1110,
        6 => 0b1111_1100,
        5 => 0b1111_1000,
        4 => 0b1111_0000,
        3 => 0b1110_0000,
        2 => 0b1100_0000,
        1 => 0b1000_0000,
        _ => 0b0000_0000,
    }
}

/// Returns a u8 mask with provided `num` amount of 1's on the right side (least significant bit)
pub fn get_right_and_mask(num: usize) -> u8 {
    match num {
        8 => 0b1111_1111,
        7 => 0b0111_1111,
        6 => 0b0011_1111,
        5 => 0b0001_1111,
        4 => 0b0000_1111,
        3 => 0b0000_0111,
        2 => 0b0000_0011,
        1 => 0b0000_0001,
        _ => 0b0000_0000,
    }
}

/// calculate the starting bit index for a field.
///
/// Returns the index of the byte the first bits of the field
///
/// # Arguments
/// * `amount_of_bits` - amount of bits the field will be after `into_bytes`.
/// * `right_rotation` - amount of bit Rotations to preform on the field. Note if rotation is not needed
///                         to retain all used bits then a shift could be used.
/// * `last_index` - total struct bytes size minus 1.
#[inline]
#[allow(
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
pub fn get_be_starting_index(
    amount_of_bits: usize,
    right_rotation: i8,
    last_index: usize,
) -> Result<usize, String> {
    //println!("be_start_index = [last;{}] - ([aob;{}] - [rs;{}]) / 8", last_index, amount_of_bits, right_rotation);
    let first = ((amount_of_bits as f64 - right_rotation as f64) / 8.0f64).ceil() as usize;
    if last_index < first {
        Err("Failed getting the starting index for big endianness, field's type doesn't fix the bit size".to_string())
    } else {
        Ok(last_index - first)
    }
}

pub struct BitMath {
    pub amount_of_bits: usize,
    pub zeros_on_left: usize,
    pub available_bits_in_first_byte: usize,
    pub starting_inject_byte: usize,
}

impl BitMath {
    pub fn from_field(field: &FieldInfo) -> Result<Self, syn::Error> {
        // get the total number of bits the field uses.
        let amount_of_bits = field.attrs.bit_length();
        // amount of zeros to have for the right mask. (right mask meaning a mask to keep data on the
        // left)
        let zeros_on_left = field.attrs.bit_range.start % 8;
        // NOTE endianness is only for determining how to get the bytes we will apply to the output.
        // calculate how many of the bits will be inside the most significant byte we are adding to.
        if 7 < zeros_on_left {
            return Err(syn::Error::new(
                field.ident.span(),
                "ne 8 - zeros_on_left = underflow",
            ));
        }
        let available_bits_in_first_byte = 8 - zeros_on_left;
        // calculate the starting byte index in the outgoing buffer
        let starting_inject_byte: usize = field.attrs.bit_range.start / 8;
        Ok(Self {
            amount_of_bits,
            zeros_on_left,
            available_bits_in_first_byte,
            starting_inject_byte,
        })
    }

    /// Returns (`amount_of_bits`, `zeros_on_left`, `available_bits_in_first_byte`, `starting_inject_byte`)
    pub fn into_tuple(self) -> (usize, usize, usize, usize) {
        (
            self.amount_of_bits,
            self.zeros_on_left,
            self.available_bits_in_first_byte,
            self.starting_inject_byte,
        )
    }
}

#[derive(Clone, Debug)]
pub enum Endianness {
    Little,
    Big,
    None,
}

impl Endianness {
    fn has_endianness(&self) -> bool {
        !matches!(self, Self::None)
    }
    fn perhaps_endianness(&mut self, size: usize) -> bool {
        if let Self::None = self {
            if size == 1 {
                let mut swap = Self::Big;
                std::mem::swap(&mut swap, self);
                true
            } else {
                false
            }
        } else {
            true
        }
    }
    // pub fn is_little(&self) -> bool {
    //     matches!(self, Self::Little)
    // }
    // pub fn is_big(&self) -> bool {
    //     matches!(self, Self::Big)
    // }
    // pub fn is_none(&self) -> bool {
    //     matches!(self, Self::None)
    // }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumberSignage {
    Signed,
    Unsigned,
}

#[derive(Clone, Debug)]
pub enum FieldDataType {
    Boolean,
    /// first field is byte size for number
    Number(usize, NumberSignage, proc_macro2::TokenStream),
    Float(usize, proc_macro2::TokenStream),
    /// first value is primitive type byte size of enum value in bytes.
    Enum(proc_macro2::TokenStream, usize, proc_macro2::TokenStream),
    /// first field is size in BYTES of the entire struct
    Struct(usize, proc_macro2::TokenStream),
    Char(usize, proc_macro2::TokenStream),
    // array types are Subfield info, array length, ident
    ElementArray(Box<SubFieldInfo>, usize, proc_macro2::TokenStream),
    BlockArray(Box<SubFieldInfo>, usize, proc_macro2::TokenStream),
}

impl FieldDataType {
    /// byte size of actual rust type .
    pub fn size(&self) -> usize {
        match self {
            Self::Number(size, _, _)
            | Self::Float(size, _)
            | Self::Enum(_, size, _)
            | Self::Struct(size, _)
            | Self::Char(size, _) => *size,
            Self::ElementArray(ref fields, size, _) | Self::BlockArray(ref fields, size, _) => {
                fields.ty.size() * size
            }
            Self::Boolean => 1,
        }
    }

    pub fn type_quote(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Number(_, _, ref ident)
            | Self::Float(_, ref ident)
            | Self::Enum(_, _, ref ident)
            | Self::Struct(_, ref ident)
            | Self::Char(_, ref ident)
            | Self::ElementArray(_, _, ref ident)
            | Self::BlockArray(_, _, ref ident) => ident.clone(),
            Self::Boolean => quote! {bool},
        }
    }
    pub fn is_number(&self) -> bool {
        match self {
            Self::Enum(_, _, _) | Self::Number(_, _, _) | Self::Float(_, _) | Self::Char(_, _) => {
                true
            }
            Self::Boolean | Self::Struct(_, _) => false,
            Self::ElementArray(ref ty, _, _) | Self::BlockArray(ref ty, _, _) => {
                ty.as_ref().ty.is_number()
            }
        }
    }
    fn get_element_bit_length(&self) -> usize {
        match self {
            Self::Boolean => 1,
            Self::Char(_, _) => 32,
            Self::Number(ref size, _, _)
            | Self::Enum(_, ref size, _)
            | Self::Float(ref size, _)
            | Self::Struct(ref size, _) => size * 8,
            Self::BlockArray(sub, _, _) | Self::ElementArray(sub, _, _) => {
                sub.as_ref().ty.get_element_bit_length()
            }
        }
    }
    #[allow(clippy::too_many_lines)]
    pub fn parse(
        ty: &syn::Type,
        attrs: &mut FieldAttrBuilder,
        span: Span,
        default_endianess: &Endianness,
    ) -> syn::Result<FieldDataType> {
        let data_type = match ty {
            Type::Path(ref path) => match attrs.ty {
                FieldAttrBuilderType::Struct(ref size) => FieldDataType::Struct(
                    *size,
                    if let Some(last_segment) = path.path.segments.last() {
                        let asdf = &last_segment.ident;
                        quote! {#asdf}
                    } else {
                        return Err(syn::Error::new(span, "field has no Type?"));
                    },
                ),
                FieldAttrBuilderType::Enum(ref size, ref prim) => FieldDataType::Enum(
                    quote! {#prim},
                    *size,
                    if let Some(last_segment) = path.path.segments.last() {
                        let asdf = &last_segment.ident;
                        quote! {#asdf}
                    } else {
                        return Err(syn::Error::new(span, "field has no Type?"));
                    },
                ),
                _ => Self::parse_path(&path.path, attrs, span)?,
            },
            Type::Array(ref array_path) => {
                // arrays must use a literal for length, because its would be hard any other way.
                if let syn::Expr::Lit(ref lit_expr) = array_path.len {
                    if let syn::Lit::Int(ref lit_int) = lit_expr.lit {
                        if let Ok(array_length) = lit_int.base10_parse::<usize>() {
                            match attrs.ty {
                                FieldAttrBuilderType::ElementArray(
                                    ref element_bit_size,
                                    ref sub,
                                ) => {
                                    attrs.bit_range = match std::mem::take(&mut attrs.bit_range) {
                                        FieldBuilderRange::Range(ref range) => {
                                            if range.end < range.start {
                                                return Err(syn::Error::new(
                                                    span,
                                                    "range end is less than range start",
                                                ));
                                            }
                                            if range.end - range.start
                                                != *element_bit_size * array_length
                                            {
                                                return Err(
                                                    syn::Error::new(
                                                        span,
                                                        "Element arrays bit range didn't match (element bit size * array length)"
                                                    )
                                                );
                                            }
                                            FieldBuilderRange::Range(range.clone())
                                        }
                                        FieldBuilderRange::LastEnd(ref last_end) => {
                                            FieldBuilderRange::Range(
                                                *last_end
                                                    ..last_end + (array_length * *element_bit_size),
                                            )
                                        }
                                        FieldBuilderRange::None => {
                                            return Err(syn::Error::new(
                                                span,
                                                "failed getting Range for element array",
                                            ));
                                        }
                                    };
                                    let mut sub_attrs = attrs.clone();
                                    if let Type::Array(_) = array_path.elem.as_ref() {
                                    } else if let Some(ref ty) = sub.as_ref() {
                                        sub_attrs.ty = ty.clone();
                                    } else {
                                        sub_attrs.ty = FieldAttrBuilderType::None;
                                    }
                                    let sub_ty = Self::parse(
                                        &array_path.elem,
                                        &mut sub_attrs,
                                        span,
                                        default_endianess,
                                    )?;

                                    let type_ident = &sub_ty.type_quote();
                                    FieldDataType::ElementArray(
                                        Box::new(SubFieldInfo { ty: sub_ty }),
                                        array_length,
                                        quote! {[#type_ident;#array_length]},
                                    )
                                }
                                FieldAttrBuilderType::BlockArray(_) => {
                                    let mut sub_attrs = attrs.clone();
                                    if let Type::Array(_) = array_path.elem.as_ref() {
                                    } else {
                                        sub_attrs.ty = FieldAttrBuilderType::None;
                                    }

                                    let sub_ty = Self::parse(
                                        &array_path.elem,
                                        &mut sub_attrs,
                                        span,
                                        default_endianess,
                                    )?;
                                    attrs.endianness = sub_attrs.endianness;
                                    let type_ident = &sub_ty.type_quote();
                                    FieldDataType::BlockArray(
                                        Box::new(SubFieldInfo { ty: sub_ty }),
                                        array_length,
                                        quote! {[#type_ident;#array_length]},
                                    )
                                }
                                FieldAttrBuilderType::Enum(_, _)
                                | FieldAttrBuilderType::Struct(_) => {
                                    let mut sub_attrs = attrs.clone();
                                    if let Type::Array(_) = array_path.elem.as_ref() {
                                    } else {
                                        sub_attrs.ty = attrs.ty.clone();
                                    }

                                    let sub_ty = Self::parse(
                                        &array_path.elem,
                                        &mut sub_attrs,
                                        span,
                                        default_endianess,
                                    )?;
                                    attrs.endianness = sub_attrs.endianness;
                                    let type_ident = &sub_ty.type_quote();
                                    FieldDataType::BlockArray(
                                        Box::new(SubFieldInfo { ty: sub_ty }),
                                        array_length,
                                        quote! {[#type_ident;#array_length]},
                                    )
                                }
                                FieldAttrBuilderType::None => {
                                    let mut sub_attrs = attrs.clone();
                                    if let Type::Array(_) = array_path.elem.as_ref() {
                                    } else {
                                        sub_attrs.ty = FieldAttrBuilderType::None;
                                    }
                                    let sub_ty = Self::parse(
                                        &array_path.elem,
                                        &mut sub_attrs,
                                        span,
                                        default_endianess,
                                    )?;
                                    attrs.bit_range = match std::mem::take(&mut attrs.bit_range) {
                                        FieldBuilderRange::Range(ref range) => {
                                            if range.end < range.start {
                                                return Err(syn::Error::new(
                                                    span,
                                                    "range end is less than range start",
                                                ));
                                            }
                                            if range.end - range.start % array_length != 0 {
                                                return Err(
                                                    syn::Error::new(
                                                        span,
                                                        "Array Inference failed because given total bit_length does not split up evenly between elements"
                                                    )
                                                );
                                            }
                                            FieldBuilderRange::Range(range.clone())
                                        }
                                        FieldBuilderRange::LastEnd(ref last_end) => {
                                            let element_bit_length =
                                                sub_ty.get_element_bit_length();
                                            FieldBuilderRange::Range(
                                                *last_end
                                                    ..last_end
                                                        + (array_length * element_bit_length),
                                            )
                                        }
                                        FieldBuilderRange::None => {
                                            return Err(syn::Error::new(
                                                span,
                                                "failed getting Range for element array",
                                            ));
                                        }
                                    };
                                    let type_ident = &sub_ty.type_quote();
                                    FieldDataType::ElementArray(
                                        Box::new(SubFieldInfo { ty: sub_ty }),
                                        array_length,
                                        quote! {[#type_ident;#array_length]},
                                    )
                                }
                            }
                        } else {
                            return Err(Error::new(
                                array_path.bracket_token.span.span(),
                                "failed parsing array length as literal integer",
                            ));
                        }
                    } else {
                        return Err(Error::new(array_path.bracket_token.span.span(), "Couldn't determine Array length, literal array lengths must be an integer"));
                    }
                } else {
                    return Err(Error::new(
                        array_path.bracket_token.span.span(),
                        "Couldn't determine Array length, must be literal",
                    ));
                }
            }
            _ => {
                return Err(Error::new(span, "Unsupported field type"));
            }
        };
        // if the type is a number and its endianess is None (numbers should have endianess) then we
        // apply the structs default (which might also be None)
        if data_type.is_number() && !attrs.endianness.perhaps_endianness(data_type.size()) {
            if default_endianess.has_endianness() {
                attrs.endianness = default_endianess.clone();
            } else if data_type.size() == 1 {
                let mut big = Endianness::Big;
                std::mem::swap(&mut attrs.endianness, &mut big);
            } else {
                let mut little = Endianness::Little;
                std::mem::swap(&mut attrs.endianness, &mut little);
                // return Err(Error::new(ident.span(), "field without defined endianess found, please set endianess of struct or fields"));
            }
        }

        Ok(data_type)
    }
    #[allow(clippy::too_many_lines)]
    fn parse_path(
        path: &syn::Path,
        attrs: &mut FieldAttrBuilder,
        field_span: Span,
    ) -> syn::Result<FieldDataType> {
        match attrs.ty {
            FieldAttrBuilderType::None => {
                if let Some(last_segment) = path.segments.last() {
                    let type_quote = &last_segment.ident;
                    let field_type_name = last_segment.ident.to_string();
                    match field_type_name.as_str() {
                        "bool" => match attrs.bit_range {
                            #[allow(clippy::range_plus_one)]
                            FieldBuilderRange::LastEnd(start) => {
                                attrs.bit_range = FieldBuilderRange::Range(start..start + 1);
                                Ok(FieldDataType::Boolean)
                            }
                            _ => Ok(FieldDataType::Boolean),
                        },
                        "u8" => Ok(FieldDataType::Number(
                            1,
                            NumberSignage::Unsigned,
                            quote! {#type_quote},
                        )),
                        "i8" => Ok(FieldDataType::Number(
                            1,
                            NumberSignage::Signed,
                            quote! {#type_quote},
                        )),
                        "u16" => Ok(FieldDataType::Number(
                            2,
                            NumberSignage::Unsigned,
                            quote! {#type_quote},
                        )),
                        "i16" => Ok(FieldDataType::Number(
                            2,
                            NumberSignage::Signed,
                            quote! {#type_quote},
                        )),
                        "f32" => {
                            if let FieldBuilderRange::Range(ref span) = attrs.bit_range {
                                if 32 != span.end - span.start {
                                    return Err(syn::Error::new(field_span, format!("f32 must be full sized, if this is a problem for you open an issue.. provided bit length = {}.", span.end - span.start)));
                                }
                            }
                            Ok(FieldDataType::Float(4, quote! {#type_quote}))
                        }
                        "u32" => Ok(FieldDataType::Number(
                            4,
                            NumberSignage::Unsigned,
                            quote! {#type_quote},
                        )),
                        "i32" => Ok(FieldDataType::Number(
                            4,
                            NumberSignage::Signed,
                            quote! {#type_quote},
                        )),
                        "char" => Ok(FieldDataType::Char(4, quote! {#type_quote})),
                        "f64" => {
                            if let FieldBuilderRange::Range(ref span) = attrs.bit_range {
                                if 64 != span.end - span.start {
                                    return Err(syn::Error::new(field_span, format!("f64 must be full sized, if this is a problem for you open an issue. provided bit length = {}.", span.end - span.start)));
                                }
                            }
                            Ok(FieldDataType::Float(8, quote! {#type_quote}))
                        }
                        "u64" => Ok(FieldDataType::Number(
                            8,
                            NumberSignage::Unsigned,
                            quote! {#type_quote},
                        )),
                        "i64" => Ok(FieldDataType::Number(
                            8,
                            NumberSignage::Signed,
                            quote! {#type_quote},
                        )),
                        "u128" => Ok(FieldDataType::Number(
                            16,
                            NumberSignage::Unsigned,
                            quote! {#type_quote},
                        )),
                        "i128" => Ok(FieldDataType::Number(
                            16,
                            NumberSignage::Signed,
                            quote! {#type_quote},
                        )),
                        "usize" | "isize" => Err(Error::new(
                            field_span,
                            "usize and isize are not supported due to ambiguous sizing".to_string(),
                        )),
                        _ => {
                            Ok(FieldDataType::Struct(
                                match attrs.bit_range {
                                    FieldBuilderRange::Range(ref range) => {
                                        (range.end - range.start).div_ceil(8)
                                    }
                                    FieldBuilderRange::LastEnd(_) | FieldBuilderRange::None => {
                                        return Err(Error::new(
                                            field_span,
                                            format!("unknown primitive type. If this type is a Bitfield as well you need to define the bit_length because bondrewd has no way to determine the size of another struct at compile time. [{field_type_name}]"),
                                        ));
                                    }
                                },
                                quote! {#type_quote},
                            ))
                            // Err(Error::new(
                            //     field_span,
                            //     format!("unknown primitive type [{}]", field_type_name),
                            // ))
                        }
                    }
                } else {
                    Err(syn::Error::new(field_span, "field has no Type?"))
                }
            }
            FieldAttrBuilderType::Struct(size) => {
                if let Some(ident) = path.get_ident() {
                    Ok(FieldDataType::Struct(size, quote! {#ident}))
                } else {
                    Err(syn::Error::new(field_span, "field has no Type?"))
                }
            }
            FieldAttrBuilderType::Enum(size, ref type_ident) => {
                if let Some(ident) = path.get_ident() {
                    Ok(FieldDataType::Enum(
                        quote! {#type_ident},
                        size,
                        quote! {#ident},
                    ))
                } else {
                    Err(syn::Error::new(field_span, "field has no Type?"))
                }
            }
            _ => Err(syn::Error::new(
                field_span,
                "Array did not get detected properly, found Path",
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ReserveFieldOption {
    NotReserve,
    ReserveField,
    FakeReserveField,
    ReadOnly,
}

impl ReserveFieldOption {
    pub fn write_field(&self) -> bool {
        match self {
            Self::ReadOnly | Self::FakeReserveField | Self::ReserveField => false,
            Self::NotReserve => true,
        }
    }

    pub fn read_field(&self) -> bool {
        match self {
            Self::FakeReserveField | Self::ReserveField => false,
            Self::NotReserve | Self::ReadOnly => true,
        }
    }

    pub fn is_fake_field(&self) -> bool {
        match self {
            Self::FakeReserveField => true,
            Self::ReserveField | Self::NotReserve | Self::ReadOnly => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum OverlapOptions {
    None,
    Allow(usize),
    Redundant,
}

impl OverlapOptions {
    pub fn enabled(&self) -> bool {
        !matches!(self, Self::None)
    }
    pub fn is_redundant(&self) -> bool {
        matches!(self, Self::Redundant)
    }
}

#[derive(Clone, Debug)]
pub struct FieldAttrs {
    pub endianness: Box<Endianness>,
    pub bit_range: Range<usize>,
    pub reserve: ReserveFieldOption,
    pub overlap: OverlapOptions,
    /// This should only ever be true on the Invalid case for enums that what to capture the invalid Id.
    pub capture_id: bool,
}

impl FieldAttrs {
    pub fn bit_length(&self) -> usize {
        self.bit_range.end - self.bit_range.start
    }
}

#[derive(Clone, Debug)]
pub struct SubFieldInfo {
    pub ty: FieldDataType,
}

pub struct ElementSubFieldIter {
    pub outer_ident: Box<FieldIdent>,
    pub endianness: Box<Endianness>,
    // this range is elements in the array, not bit range
    pub range: Range<usize>,
    pub starting_bit_index: usize,
    pub ty: FieldDataType,
    pub element_bit_size: usize,
    pub reserve: ReserveFieldOption,
    pub overlap: OverlapOptions,
}

impl Iterator for ElementSubFieldIter {
    type Item = FieldInfo;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.range.next() {
            let start = self.starting_bit_index + (index * self.element_bit_size);
            let attrs = FieldAttrs {
                bit_range: start..start + self.element_bit_size,
                endianness: self.endianness.clone(),
                reserve: self.reserve.clone(),
                overlap: self.overlap.clone(),
                capture_id: false,
            };
            let outer_ident = self.outer_ident.ident().clone();
            let name = quote::format_ident!("{}_{}", outer_ident, index);
            let ident = Box::new((outer_ident, name).into());
            Some(FieldInfo {
                ident,
                attrs,
                ty: self.ty.clone(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct BlockSubFieldIter {
    pub outer_ident: Box<FieldIdent>,
    pub endianness: Box<Endianness>,
    //array length
    pub length: usize,
    pub starting_bit_index: usize,
    pub ty: FieldDataType,
    pub bit_length: usize,
    pub total_bytes: usize,
    pub reserve: ReserveFieldOption,
    pub overlap: OverlapOptions,
}

impl Iterator for BlockSubFieldIter {
    type Item = FieldInfo;
    fn next(&mut self) -> Option<Self::Item> {
        if self.length != 0 {
            let mut ty_size = self.ty.size() * 8;
            if self.bit_length % ty_size != 0 {
                ty_size = self.bit_length % ty_size;
            }
            let start = self.starting_bit_index;
            self.starting_bit_index = start + ty_size;
            let attrs = FieldAttrs {
                bit_range: start..(start + ty_size),
                endianness: self.endianness.clone(),
                reserve: self.reserve.clone(),
                overlap: self.overlap.clone(),
                capture_id: false,
            };
            self.bit_length -= ty_size;
            let index = self.total_bytes - self.length;
            let outer_ident = self.outer_ident.ident().clone();
            let name = quote::format_ident!("{}_{}", outer_ident, index);
            let ident = Box::new((outer_ident, name).into());
            self.length -= 1;
            Some(FieldInfo {
                ident,
                attrs,
                ty: self.ty.clone(),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub enum FieldIdent {
    Ident {
        /// name of the field given by the user.
        ident: Ident,
        /// name of the value given by bondrewd.
        name: Ident,
    },
    Index {
        /// Index of the field in the tuple struct/enum-variant
        index: usize,
        /// name of the value given by bondrewd.
        name: Ident,
    },
}

impl FieldIdent {
    pub fn ident(&self) -> Ident {
        match self {
            FieldIdent::Ident { ident, name: _ } => ident.clone(),
            FieldIdent::Index { index, name } => Ident::new(&format!("field_{index}"), name.span()),
        }
    }
    pub fn name(&self) -> Ident {
        match self {
            FieldIdent::Ident { ident: _, name } | FieldIdent::Index { index: _, name } => {
                name.clone()
            }
        }
    }
    pub fn span(&self) -> Span {
        match self {
            FieldIdent::Ident { ident, name: _ } => ident.span(),
            FieldIdent::Index { index: _, name } => name.span(),
        }
    }
}

impl From<(usize, Span)> for FieldIdent {
    fn from((value, span): (usize, Span)) -> Self {
        Self::Index {
            index: value,
            name: Ident::new(&format!("field_{value}"), span),
        }
    }
}

impl From<Ident> for FieldIdent {
    fn from(value: Ident) -> Self {
        Self::Ident {
            ident: value.clone(),
            name: value,
        }
    }
}
impl From<(Ident, Ident)> for FieldIdent {
    fn from((value, value2): (Ident, Ident)) -> Self {
        Self::Ident {
            ident: value,
            name: value2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub ident: Box<FieldIdent>,
    pub ty: FieldDataType,
    pub attrs: FieldAttrs,
}

impl FieldInfo {
    // pub fn right_shift(&self, math: &BitMath) -> i8 {
    //     match *self.attrs.endianness {
    //         Endianness::Little => {
    //             let mut bits_needed_in_msb = math.amount_of_bits % 8;
    //             if bits_needed_in_msb == 0 {
    //                 bits_needed_in_msb = 8;
    //             }
    //             let right_shift: i8 =
    //                 (bits_needed_in_msb as i8) - ((math.available_bits_in_first_byte % 8) as i8);
    //             if right_shift == 8 {
    //                 0
    //             } else {
    //                 right_shift
    //             }
    //         }
    //         #[allow(clippy::cast_possible_truncation)]
    //         Endianness::Big => {
    //             let mut right_shift: i8 = ((math.amount_of_bits % 8) as i8)
    //                 - ((math.available_bits_in_first_byte % 8) as i8);
    //             // TODO this right_shift modification is a fix because left shifts in be number are broken.
    //             // this exists in both from and into bytes for big endian. right shift should not be mut.
    //             while right_shift < 0 {
    //                 right_shift += 8;
    //             }
    //             right_shift
    //         }
    //         #[allow(clippy::cast_possible_truncation)]
    //         Endianness::None => 8_i8 - ((math.available_bits_in_first_byte % 8) as i8),
    //     }
    // }
    pub fn ident(&self) -> &FieldIdent {
        &self.ident
    }
    pub fn span(&self) -> Span {
        self.ident.span()
    }
    fn overlapping(&self, other: &Self) -> bool {
        if self.attrs.overlap.enabled() || other.attrs.overlap.enabled() {
            return false;
        }
        // check that self's start is not within other's range
        if self.attrs.bit_range.start >= other.attrs.bit_range.start
            && (self.attrs.bit_range.start == other.attrs.bit_range.start
                || self.attrs.bit_range.start < other.attrs.bit_range.end)
        {
            return true;
        }
        // check that other's start is not within self's range
        if other.attrs.bit_range.start >= self.attrs.bit_range.start
            && (other.attrs.bit_range.start == self.attrs.bit_range.start
                || other.attrs.bit_range.start < self.attrs.bit_range.end)
        {
            return true;
        }
        if self.attrs.bit_range.end > other.attrs.bit_range.start
            && self.attrs.bit_range.end <= other.attrs.bit_range.end
        {
            return true;
        }
        if other.attrs.bit_range.end > self.attrs.bit_range.start
            && other.attrs.bit_range.end <= self.attrs.bit_range.end
        {
            return true;
        }
        false
    }

    #[inline]
    // this returns how many bits of the fields pertain to total structure bits.
    // where as attrs.bit_length() give you bits the fields actually needs.
    pub fn bit_size(&self) -> usize {
        if self.attrs.overlap.is_redundant() {
            0
        } else {
            let minus = if let OverlapOptions::Allow(skip) = self.attrs.overlap {
                skip
            } else {
                0
            };
            (self.attrs.bit_range.end - self.attrs.bit_range.start) - minus
        }
    }

    #[inline]
    pub fn struct_byte_size(&self) -> usize {
        self.ty.size()
    }

    pub fn get_element_iter(&self) -> Result<ElementSubFieldIter, syn::Error> {
        if let FieldDataType::ElementArray(ref sub_field, ref array_length, _) = self.ty {
            Ok(ElementSubFieldIter {
                outer_ident: self.ident.clone(),
                endianness: self.attrs.endianness.clone(),
                element_bit_size: (self.attrs.bit_range.end - self.attrs.bit_range.start)
                    / array_length,
                starting_bit_index: self.attrs.bit_range.start,
                range: 0..*array_length,
                ty: sub_field.ty.clone(),
                overlap: self.attrs.overlap.clone(),
                reserve: self.attrs.reserve.clone(),
            })
        } else {
            Err(syn::Error::new(
                self.ident.span(),
                "This field was trying to get used like an array",
            ))
        }
    }

    pub fn get_block_iter(&self) -> Result<BlockSubFieldIter, syn::Error> {
        if let FieldDataType::BlockArray(ref sub_field, ref array_length, _) = self.ty {
            let bit_length = self.attrs.bit_range.end - self.attrs.bit_range.start;
            Ok(BlockSubFieldIter {
                outer_ident: self.ident.clone(),
                endianness: self.attrs.endianness.clone(),
                bit_length,
                starting_bit_index: self.attrs.bit_range.start,
                length: *array_length,
                ty: sub_field.ty.clone(),
                total_bytes: *array_length,
                reserve: self.attrs.reserve.clone(),
                overlap: self.attrs.overlap.clone(),
            })
        } else {
            Err(syn::Error::new(
                self.ident.span(),
                "This field was trying to get used like an array",
            ))
        }
    }

    pub fn from_syn_field(
        field: &syn::Field,
        fields: &Vec<FieldInfo>,
        attrs: &AttrInfo,
    ) -> syn::Result<Self> {
        let ident: FieldIdent = if let Some(ref name) = field.ident {
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
        let mut attrs_builder = FieldAttrBuilder::parse(field, last_relevant_field, ident.span())?;
        // check the field for supported types.
        let data_type = FieldDataType::parse(
            &field.ty,
            &mut attrs_builder,
            ident.span(),
            &attrs.default_endianess,
        )?;

        let attr_result: std::result::Result<FieldAttrs, TryFromAttrBuilderError> =
            attrs_builder.try_into();

        let attrs = match attr_result {
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

#[derive(Debug, Clone)]
pub enum StructEnforcement {
    /// there is no enforcement so if bits are unused then it will act like they are a reserve field
    NoRules,
    /// enforce the BIT_SIZE equals BYTE_SIZE * 8
    EnforceFullBytes,
    /// enforce an amount of bits total that need to be used.
    EnforceBitAmount(usize),
}

#[derive(Clone)]
pub enum IdPosition {
    Leading,
    Trailing,
}

#[derive(Clone)]
pub struct AttrInfo {
    /// if false then bit 0 is the Most Significant Bit meaning the first values first bit will start there.
    /// if true then bit 0 is the Least Significant Bit (the last bit in the last byte).
    pub lsb_zero: bool,
    /// flip all the bytes, like .reverse() for vecs or arrays. but we do that here because we can do
    /// it with no runtime cost.
    pub flip: bool,
    pub enforcement: StructEnforcement,
    pub default_endianess: Endianness,
    pub fill_bits: Option<usize>,
    // Enum only
    pub id: Option<u128>,
    pub invalid: bool,
}

impl Default for AttrInfo {
    fn default() -> Self {
        Self {
            lsb_zero: false,
            flip: false,
            enforcement: StructEnforcement::NoRules,
            default_endianess: Endianness::None,
            fill_bits: None,
            id: None,
            invalid: false,
        }
    }
}

#[derive(Clone)]
pub struct StructInfo {
    pub name: Ident,
    pub attrs: AttrInfo,
    pub fields: Vec<FieldInfo>,
    pub vis: syn::Visibility,
    pub tuple: bool,
}

impl StructInfo {
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

pub struct EnumInfo {
    pub name: Ident,
    pub variants: Vec<StructInfo>,
    pub attrs: EnumAttrInfo,
    pub vis: syn::Visibility,
}

impl EnumInfo {
    pub const VARIANT_ID_NAME: &'static str = "variant_id";
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
    pub fn id_ident(&self) -> syn::Result<TokenStream> {
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
    pub fn generate_id_field(&self) -> syn::Result<FieldInfo> {
        let e = match &self.attrs.attrs.default_endianess {
            Endianness::None | Endianness::Little => Endianness::Little,
            Endianness::Big => Endianness::Big,
        };
        Ok(FieldInfo {
            ident: Box::new(format_ident!("{}", EnumInfo::VARIANT_ID_NAME).into()),
            ty: FieldDataType::Number(
                self.attrs.id_bits.div_ceil(8),
                NumberSignage::Unsigned,
                self.id_ident()?,
            ),
            attrs: FieldAttrs {
                endianness: Box::new(e),
                bit_range: 0..self.attrs.id_bits,
                reserve: ReserveFieldOption::NotReserve,
                overlap: OverlapOptions::None,
                capture_id: false,
            },
        })
    }
}

#[derive(Clone)]
pub struct EnumAttrInfoBuilder {
    pub id_bits: Option<usize>,
    pub id_position: IdPosition,
    pub total_bit_size: Option<usize>,
    pub payload_bit_size: Option<usize>,
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

impl Default for EnumAttrInfoBuilder {
    fn default() -> Self {
        Self {
            id_bits: None,
            id_position: IdPosition::Leading,
            total_bit_size: None,
            payload_bit_size: None,
        }
    }
}

pub enum ObjectInfo {
    Struct(StructInfo),
    Enum(EnumInfo),
}

/// `id_bits` is the amount of bits the enum's id takes.
fn get_id_type(id_bits: usize, span: Span) -> syn::Result<TokenStream> {
    match id_bits {
        0..=8 => Ok(quote! {u8}),
        9..=16 => Ok(quote! {u16}),
        17..=32 => Ok(quote! {u32}),
        33..=64 => Ok(quote! {u64}),
        65..=128 => Ok(quote! {u128}),
        _ => Err(syn::Error::new(span, "id size is invalid")),
    }
}

impl ObjectInfo {
    #[cfg(feature = "dyn_fns")]
    pub fn vis(&self) -> &syn::Visibility {
        match self {
            ObjectInfo::Struct(s) => &s.vis,
            ObjectInfo::Enum(e) => &e.vis,
        }
    }
    pub fn name(&self) -> Ident {
        match self {
            ObjectInfo::Struct(s) => s.name.clone(),
            ObjectInfo::Enum(e) => e.name.clone(),
        }
    }
    fn parse_struct_attrs(
        attrs: &[Attribute],
        attrs_info: &mut AttrInfo,
        is_variant: bool,
    ) -> syn::Result<()> {
        for attr in attrs {
            let span = attr.pound_token.span();
            if attr.path().is_ident("bondrewd") {
                let nested =
                    attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                for meta in &nested {
                    Self::parse_struct_attrs_meta(span, attrs_info, meta, is_variant)?;
                }
            }
        }
        Ok(())
    }

    fn parse_enum_attrs(
        attrs: &[Attribute],
        attrs_info: &mut AttrInfo,
        enum_attrs_info: &mut EnumAttrInfoBuilder,
    ) -> syn::Result<()> {
        for attr in attrs {
            let span = attr.pound_token.span();
            if attr.path().is_ident("bondrewd") {
                let nested =
                    attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                for meta in &nested {
                    Self::parse_enum_attrs_meta(span, attrs_info, enum_attrs_info, meta)?;
                }
            }
            // let meta = attr.parse_meta()?;
            // Self::parse_enum_attrs_meta(span, attrs_info, enum_attrs_info, &attr)?;
        }
        Ok(())
    }
    // Parses the Expression, looking for a literal number expression
    fn parse_lit_discriminant_expr(input: &Expr) -> syn::Result<u128> {
        match input {
            Expr::Lit(ref lit) => match lit.lit {
                Lit::Int(ref i) => Ok(i.base10_parse()?),
                _ => Err(syn::Error::new(
                    input.span(),
                    "non-integer literals for custom discriminant are illegal.",
                )),
            },
            _ => Err(syn::Error::new(
                input.span(),
                "non-literal expressions for custom discriminant are illegal.",
            )),
        }
    }
    #[allow(clippy::too_many_lines)]
    pub fn parse(input: &DeriveInput) -> syn::Result<Self> {
        // get the struct, error out if not a struct
        let mut attrs = AttrInfo::default();
        let name = input.ident.clone();
        match input.data {
            syn::Data::Struct(ref data) => {
                let tuple = matches!(data.fields, syn::Fields::Unnamed(_));
                Self::parse_struct_attrs(&input.attrs, &mut attrs, false)?;
                let fields = Self::parse_fields(&name, &data.fields, &attrs, None, tuple)?;
                Ok(Self::Struct(StructInfo {
                    name,
                    attrs,
                    fields,
                    vis: input.vis.clone(),
                    tuple,
                }))
            }
            syn::Data::Enum(ref data) => {
                let mut enum_attrs = EnumAttrInfoBuilder::default();
                Self::parse_enum_attrs(&input.attrs, &mut attrs, &mut enum_attrs)?;
                let mut variants: Vec<StructInfo> = Vec::default();
                let (id_field_type, id_bits) = {
                    let id_bits = if let Some(id_bits) = enum_attrs.id_bits {
                        id_bits
                    } else if let (Some(payload_size), Some(total_size)) =
                        (enum_attrs.payload_bit_size, enum_attrs.total_bit_size)
                    {
                        total_size - payload_size
                    } else {
                        return Err(syn::Error::new(
                            data.enum_token.span(),
                            "Must define the length of the id use #[bondrewd(id_bit_length = AMOUNT_OF_BITS)]",
                        ));
                    };
                    (
                        FieldDataType::Number(
                            id_bits.div_ceil(8),
                            NumberSignage::Unsigned,
                            get_id_type(id_bits, name.span())?,
                        ),
                        id_bits,
                    )
                };
                let id_field = FieldInfo {
                    ident: Box::new(format_ident!("{}", EnumInfo::VARIANT_ID_NAME).into()),
                    ty: id_field_type,
                    attrs: FieldAttrs {
                        endianness: Box::new(attrs.default_endianess.clone()),
                        // this need to accommodate tailing ids, currently this locks the
                        // id field to the first field read from the starting point of reading.
                        // TODO make sure this gets corrected if the id size is unknown.
                        bit_range: 0..id_bits,
                        reserve: ReserveFieldOption::FakeReserveField,
                        overlap: OverlapOptions::None,
                        capture_id: false,
                    },
                };
                for variant in &data.variants {
                    let tuple = matches!(variant.fields, syn::Fields::Unnamed(_));
                    let mut attrs = attrs.clone();
                    if let Some((_, ref expr)) = variant.discriminant {
                        let parsed = Self::parse_lit_discriminant_expr(expr)?;
                        attrs.id = Some(parsed);
                    }
                    Self::parse_struct_attrs(&variant.attrs, &mut attrs, true)?;
                    let variant_name = variant.ident.clone();
                    // TODO currently we always add the id field, but some people might want the id to be a
                    // field in the variant. this would no longer need to insert the id as a "fake-field".
                    let fields = Self::parse_fields(
                        &variant_name,
                        &variant.fields,
                        &attrs,
                        Some(id_field.clone()),
                        tuple,
                    )?;
                    variants.push(StructInfo {
                        name: variant_name,
                        attrs,
                        fields,
                        vis: input.vis.clone(),
                        tuple,
                    });
                }
                // detect and fix variants without ids and verify non conflict.
                let mut used_ids: Vec<u128> = Vec::default();
                let mut unassigned_indices: Vec<usize> = Vec::default();
                let mut invalid_index: Option<usize> = None;
                let mut largest = 0;
                for (i, variant) in variants.iter().enumerate() {
                    if let Some(ref value) = variant.attrs.id {
                        if used_ids.contains(value) {
                            return Err(Error::new(
                                variant.name.span(),
                                "variant identifier used twice.",
                            ));
                        }
                        used_ids.push(*value);
                    } else {
                        unassigned_indices.push(i);
                    }
                    if variant.attrs.invalid {
                        if invalid_index.is_none() {
                            invalid_index = Some(i);
                        } else {
                            return Err(Error::new(
                                variant.name.span(),
                                "second catch invalid variant found. only 1 is currently allowed.",
                            ));
                        }
                    }
                    // verify the size doesn't go over set size.
                    let size = variant.total_bits();
                    if largest < size {
                        largest = size;
                    }
                    if let Some(bit_size) = enum_attrs.payload_bit_size {
                        if bit_size < size - variant.fields[0].attrs.bit_length() {
                            return Err(Error::new(
                                variant.name.span(),
                                format!("variant is larger than defined payload_size of enum. defined size: {bit_size}. variant size: {}", size- variant.fields[0].attrs.bit_length()),
                            ));
                        }
                    } else if let (Some(bit_size), Some(id_size)) =
                        (enum_attrs.total_bit_size, enum_attrs.id_bits)
                    {
                        if bit_size - id_size < size - variant.fields[0].attrs.bit_length() {
                            return Err(Error::new(
                                variant.name.span(),
                                format!("variant with id is larger than defined total_size of enum. defined size: {}. calculated size: {}", bit_size - id_size, size),
                            ));
                        }
                    }
                }
                if !unassigned_indices.is_empty() {
                    let mut current_guess: u128 = 0;
                    for i in unassigned_indices {
                        while used_ids.contains(&current_guess) {
                            current_guess += 1;
                        }
                        variants[i].attrs.id = Some(current_guess);
                        used_ids.push(current_guess);
                        current_guess += 1;
                    }
                }
                if let Some(ii) = invalid_index {
                    let var = variants.remove(ii);
                    variants.push(var);
                }
                // find minimal id size from largest id value
                used_ids.sort_unstable();
                let min_id_size = if let Some(last_id) = used_ids.last() {
                    let mut x = *last_id;
                    // find minimal id size from largest id value
                    let mut n = 0;
                    while x != 0 {
                        x >>= 1;
                        n += 1;
                    }
                    n
                } else {
                    return Err(Error::new(
                        data.enum_token.span(),
                        "found no variants and could not determine size of id".to_string(),
                    ));
                };
                let enum_attrs = match (enum_attrs.payload_bit_size, enum_attrs.total_bit_size) {
                    (Some(payload), None) => {
                        if let Some(id) = enum_attrs.id_bits {
                            EnumAttrInfo {
                                payload_bit_size: payload,
                                id_bits: id,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        } else {
                            EnumAttrInfo {
                                payload_bit_size: payload,
                                id_bits: min_id_size,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        }
                    }
                    (None, Some(total)) => {
                        if let Some(id) = enum_attrs.id_bits {
                            EnumAttrInfo {
                                payload_bit_size: total - id,
                                id_bits: id,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        } else if largest < total {
                            let id = total - largest;
                            EnumAttrInfo {
                                payload_bit_size: largest,
                                id_bits: id,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        } else {
                            return Err(Error::new(
                                data.enum_token.span(),
                                "specified total is not smaller than the largest payload size, meaning there is not room the the variant id.".to_string(),
                            ));
                        }
                    }
                    (Some(payload), Some(total)) => {
                        if let Some(id) = enum_attrs.id_bits {
                            if payload + id != total {
                                return Err(Error::new(
                                    data.enum_token.span(),
                                    format!("total_size, payload_size, and id_size where all specified but id_size ({id}) + payload_size ({payload}) is not equal to total_size ({total})"),
                                ));
                            }
                            if payload < largest {
                                return Err(Error::new(
                                    data.enum_token.span(),
                                    "detected a variant over the maximum defined size.".to_string(),
                                ));
                            }
                            EnumAttrInfo {
                                id_bits: id,
                                id_position: enum_attrs.id_position,
                                payload_bit_size: payload,
                                attrs: attrs.clone(),
                            }
                        } else {
                            EnumAttrInfo {
                                payload_bit_size: largest,
                                id_bits: min_id_size,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        }
                    }
                    _ => {
                        if let Some(id) = enum_attrs.id_bits {
                            EnumAttrInfo {
                                id_bits: id,
                                id_position: enum_attrs.id_position,
                                payload_bit_size: largest,
                                attrs: attrs.clone(),
                            }
                        } else {
                            EnumAttrInfo {
                                payload_bit_size: largest,
                                id_bits: min_id_size,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        }
                    }
                };
                if enum_attrs.id_bits < min_id_size {
                    return Err(Error::new(
                        data.enum_token.span(),
                        "the bit size being used is less than required to describe each variant"
                            .to_string(),
                    ));
                }
                if enum_attrs.payload_bit_size + enum_attrs.id_bits < largest {
                    return Err(Error::new(
                        data.enum_token.span(),
                        "the payload size being used is less than largest variant".to_string(),
                    ));
                }
                // let id_field_ty = FieldDataType::Number(
                //     enum_attrs.id_bits,
                //     NumberSignage::Unsigned,
                //     get_id_type(enum_attrs.id_bits, name.span())?,
                // );
                // add fill_bits if needed.
                for v in &mut variants {
                    let first_bit = v.total_bits();
                    if first_bit < largest {
                        let fill_bytes_size = (largest - first_bit).div_ceil(8);
                        let ident = quote::format_ident!("fill_bits");
                        v.fields.push(FieldInfo {
                            ident: Box::new(ident.into()),
                            attrs: FieldAttrs {
                                bit_range: first_bit..largest,
                                endianness: Box::new(Endianness::Big),
                                reserve: ReserveFieldOption::FakeReserveField,
                                overlap: OverlapOptions::None,
                                capture_id: false,
                            },
                            ty: FieldDataType::BlockArray(
                                Box::new(SubFieldInfo {
                                    ty: FieldDataType::Number(
                                        1,
                                        NumberSignage::Unsigned,
                                        quote! {u8},
                                    ),
                                }),
                                fill_bytes_size,
                                quote! {[u8;#fill_bytes_size]},
                            ),
                        });
                    }
                }
                Ok(Self::Enum(EnumInfo {
                    name,
                    variants,
                    attrs: enum_attrs,
                    vis: input.vis.clone(),
                }))
            }
            syn::Data::Union(_) => Err(Error::new(Span::call_site(), "input can not be a union")),
        }
    }
    pub fn total_bits(&self) -> usize {
        match self {
            Self::Struct(s) => s.total_bits(),
            Self::Enum(info) => info.total_bits(),
        }
    }
    pub fn total_bytes(&self) -> usize {
        self.total_bits().div_ceil(8)
    }
    fn parse_enum_attrs_meta(
        span: Span,
        info: &mut AttrInfo,
        enum_info: &mut EnumAttrInfoBuilder,
        meta: &Meta,
    ) -> Result<(), syn::Error> {
        match meta {
            Meta::NameValue(value) => {
                if let Expr::Lit(ref lit) = value.value {
                    if value.path.is_ident("id_bit_length") {
                        if let Lit::Int(ref val) = lit.lit {
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    if value > 128 {
                                        return Err(syn::Error::new(
                                            span,
                                            "Maximum id bits is 128.",
                                        ));
                                    }
                                    enum_info.id_bits = Some(value);
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        span,
                                        format!("failed parsing id_bits value [{err}]"),
                                    ))
                                }
                            }
                        }
                    } else if value.path.is_ident("id_byte_length") {
                        if let Lit::Int(ref val) = lit.lit {
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    if value > 16 {
                                        return Err(syn::Error::new(
                                            span,
                                            "Maximum id bytes is 16.",
                                        ));
                                    }
                                    enum_info.id_bits = Some(value * 8);
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        span,
                                        format!("failed parsing id_bytes value [{err}]"),
                                    ))
                                }
                            }
                        }
                    } else if value.path.is_ident("payload_bit_length") {
                        if let Lit::Int(ref val) = lit.lit {
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    enum_info.payload_bit_size = Some(value);
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        span,
                                        format!("failed parsing payload_bits value [{err}]"),
                                    ))
                                }
                            }
                        }
                    } else if value.path.is_ident("payload_byte_length") {
                        if let Lit::Int(ref val) = lit.lit {
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    enum_info.payload_bit_size = Some(value * 8);
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        span,
                                        format!("failed parsing payload_bytes value [{err}]"),
                                    ))
                                }
                            }
                        }
                    }
                }
            }
            Meta::Path(value) => {
                if let Some(ident) = value.get_ident() {
                    match ident.to_string().as_str() {
                        "id_tail" => {
                            enum_info.id_position = IdPosition::Trailing;
                        }
                        "id_head" => {
                            enum_info.id_position = IdPosition::Leading;
                        }
                        _ => {}
                    }
                }
            }
            Meta::List(_meta_list) => {}
        }
        Self::parse_struct_attrs_meta(span, info, meta, false)?;
        if let StructEnforcement::EnforceBitAmount(bits) = info.enforcement {
            enum_info.total_bit_size = Some(bits);
            info.enforcement = StructEnforcement::NoRules;
        }
        Ok(())
    }
    #[allow(clippy::too_many_lines)]
    fn parse_struct_attrs_meta(
        span: Span,
        info: &mut AttrInfo,
        meta: &Meta,
        is_variant: bool,
    ) -> Result<(), syn::Error> {
        match meta {
            Meta::NameValue(ref value) => {
                if is_variant && value.path.is_ident(EnumInfo::VARIANT_ID_NAME) {
                    if let Expr::Lit(ref lit) = value.value {
                        if let Lit::Int(ref val) = lit.lit {
                            match val.base10_parse::<u128>() {
                                Ok(value) => {
                                    if info.id.is_none() {
                                        info.id = Some(value);
                                    } else {
                                        return Err(syn::Error::new(
                                            span,
                                            "must not have 2 ids defined.",
                                        ));
                                    }
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        span,
                                        format!("failed parsing id value [{err}]"),
                                    ))
                                }
                            }
                        } else {
                            return Err(syn::Error::new(
                                span,
                                format!(
                                    "improper usage of {}, must use literal integer ex. `{} = 0`",
                                    EnumInfo::VARIANT_ID_NAME,
                                    EnumInfo::VARIANT_ID_NAME
                                ),
                            ));
                        }
                    }
                } else if value.path.is_ident("read_from") {
                    if let Expr::Lit(ref lit) = value.value {
                        if let Lit::Str(ref val) = lit.lit {
                            match val.value().as_str() {
                            "lsb0" => info.lsb_zero = true,
                            "msb0" => info.lsb_zero = false,
                            _ => return Err(Error::new(
                                val.span(),
                                "Expected literal str \"lsb0\" or \"msb0\" for read_from attribute.",
                            )),
                        }
                        } else {
                            return Err(syn::Error::new(
                            span,
                            "improper usage of read_from, must use string ex. `read_from = \"lsb0\"`",
                        ));
                        }
                    }
                } else if value.path.is_ident("default_endianness") {
                    if let Expr::Lit(ref lit) = value.value {
                        if let Lit::Str(ref val) = lit.lit {
                            match val.value().as_str() {
                                "le" | "lsb" | "little" | "lil" => {
                                    info.default_endianess = Endianness::Little;
                                }
                                "be" | "msb" | "big" => info.default_endianess = Endianness::Big,
                                "ne" | "native" => info.default_endianess = Endianness::None,
                                _ => {}
                            }
                        } else {
                            return Err(syn::Error::new(
                            span,
                            "improper usage of default_endianness, must use string ex. `default_endianness = \"be\"`",
                        ));
                        }
                    }
                } else if value.path.is_ident("enforce_bytes") {
                    if let Expr::Lit(ref lit) = value.value {
                        if let Lit::Int(ref val) = lit.lit {
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    info.enforcement =
                                        StructEnforcement::EnforceBitAmount(value * 8);
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        span,
                                        format!("failed parsing enforce_bytes value [{err}]"),
                                    ))
                                }
                            }
                        } else {
                            return Err(syn::Error::new(
                            span,
                            "improper usage of enforce_bytes, must use literal integer ex. `enforce_bytes = 5`",
                        ));
                        }
                    }
                } else if value.path.is_ident("enforce_bits") {
                    if let Expr::Lit(ref lit) = value.value {
                        if let Lit::Int(ref val) = lit.lit {
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    info.enforcement = StructEnforcement::EnforceBitAmount(value);
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        span,
                                        format!("failed parsing enforce_bits value [{err}]"),
                                    ))
                                }
                            }
                        } else {
                            return Err(syn::Error::new(
                            span,
                            "improper usage of enforce_bits, must use literal integer ex. `enforce_bits = 5`",
                        ));
                        }
                    }
                } else if value.path.is_ident("fill_bytes") {
                    if let Expr::Lit(ref lit) = value.value {
                        if let Lit::Int(ref val) = lit.lit {
                            match val.base10_parse::<usize>() {
                                Ok(value) => {
                                    if info.fill_bits.is_none() {
                                        info.fill_bits = Some(value * 8);
                                    } else {
                                        return Err(syn::Error::new(
                                            span,
                                            "multiple fill_bits values".to_string(),
                                        ));
                                    }
                                }
                                Err(err) => {
                                    return Err(syn::Error::new(
                                        span,
                                        format!("failed parsing fill_bits value [{err}]"),
                                    ))
                                }
                            }
                        } else {
                            return Err(syn::Error::new(
                            span,
                            "improper usage of fill_bytes, must use literal integer ex. `fill_bytes = 5`",
                        ));
                        }
                    }
                }
            }
            Meta::Path(ref value) => {
                if let Some(ident) = value.get_ident() {
                    match ident.to_string().as_str() {
                        "reverse" => {
                            info.flip = true;
                        }
                        "enforce_full_bytes" => {
                            info.enforcement = StructEnforcement::EnforceFullBytes;
                        }
                        "invalid" => {
                            info.invalid = true;
                        }
                        _ => {}
                    }
                }
            }
            Meta::List(ref _meta_list) => {
                // if meta_list.path.is_ident("bondrewd") {
                //     for nested_meta in meta_list.nested.iter() {
                //         match nested_meta {
                //             NestedMeta::Meta(ref meta) => {
                //                 Self::parse_struct_attrs_meta(span, info, meta, is_variant)?;
                //             }
                //             NestedMeta::Lit(_) => {}
                //         }
                //     }
                // }
            }
        }
        Ok(())
    }
    #[allow(clippy::too_many_lines)]
    pub fn parse_fields(
        name: &Ident,
        fields: &Fields,
        attrs: &AttrInfo,
        first_field: Option<FieldInfo>,
        tuple: bool,
    ) -> syn::Result<Vec<FieldInfo>> {
        let (mut parsed_fields, is_enum) = if let Some(f) = first_field {
            (vec![f], true)
        } else {
            (Vec::default(), false)
        };
        // get the list of fields in syn form, error out if unit struct (because they have no data, and
        // data packing/analysis don't seem necessary)
        let fields = match fields {
            syn::Fields::Named(ref named_fields) => Some(
                named_fields
                    .named
                    .iter()
                    .cloned()
                    .collect::<Vec<syn::Field>>(),
            ),
            syn::Fields::Unnamed(ref fields) => {
                Some(fields.unnamed.iter().cloned().collect::<Vec<syn::Field>>())
            }
            syn::Fields::Unit => {
                if parsed_fields.first().is_none() {
                    return Err(Error::new(name.span(), "Packing a Unit Struct (Struct with no data) seems pointless to me, so i didn't write code for it."));
                }
                None
            }
        };

        // figure out what the field are and what/where they should be in byte form.
        let mut bit_size = if let Some(id_field) = parsed_fields.first() {
            id_field.bit_size()
        } else {
            0
        };
        if let Some(fields) = fields {
            for (i, ref field) in fields.iter().enumerate() {
                let mut parsed_field = FieldInfo::from_syn_field(field, &parsed_fields, attrs)?;
                if parsed_field.attrs.capture_id {
                    if is_enum {
                        if i == 0 {
                            match (&parsed_fields[0].ty, &mut parsed_field.ty) {
                                (FieldDataType::Number(_, ref bon_sign, ref bon_ty), FieldDataType::Number(_, ref user_sign, ref user_ty)) => {
                                    if parsed_fields[0].attrs.bit_range != parsed_field.attrs.bit_range {
                                        parsed_field.attrs.bit_range = parsed_fields[0].attrs.bit_range.clone();
                                    }
                                    if bon_sign != user_sign {
                                        return Err(Error::new(field.span(), format!("capture_id field must be unsigned. bondrewd will enforce the type as {bon_ty}")));
                                    }else if bon_ty.to_string() != user_ty.to_string() {
                                        return Err(Error::new(field.span(), format!("capture_id field currently must be {bon_ty} in this instance, because bondrewd makes an assumption about the id type. changing this would be difficult")));
                                    }
                                    let old_id = parsed_fields.remove(0);
                                    if tuple {
                                        parsed_field.ident = old_id.ident;
                                    }
                                }
                                (FieldDataType::Number(_bon_bits, _bon_sign, bon_ty), _) => return Err(Error::new(field.span(), format!("capture_id field must be an unsigned number. detected type is {bon_ty}."))),
                                _ => return Err(Error::new(field.span(), "an error with bondrewd has occurred, the id field should be a number but bondrewd did not use a number for the id.")),
                            }
                        } else {
                            return Err(Error::new(
                                field.span(),
                                "capture_id attribute must be the first field.",
                            ));
                        }
                    } else {
                        return Err(Error::new(
                            field.span(),
                            "capture_id attribute is intended for enum variants only.",
                        ));
                    }
                } else {
                    bit_size += parsed_field.bit_size();
                }
                parsed_fields.push(parsed_field);
            }
        }

        match attrs.enforcement {
            StructEnforcement::NoRules => {}
            StructEnforcement::EnforceFullBytes => {
                if bit_size % 8 != 0 {
                    return Err(syn::Error::new(
                        name.span(),
                        "BIT_SIZE modulus 8 is not zero",
                    ));
                }
            }
            StructEnforcement::EnforceBitAmount(expected_total_bits) => {
                if bit_size != expected_total_bits {
                    return Err(syn::Error::new(
                        name.span(),
                        format!(
                            "Bit Enforcement failed because bondrewd detected {bit_size} total bits used by defined fields, but the bit enforcement attribute is defined as {expected_total_bits} bits.",
                        ),
                    ));
                }
            }
        }

        // add reserve for fill bytes. this happens after bit enforcement because bit_enforcement is for checking user code.
        if let Some(fill_bits) = attrs.fill_bits {
            let first_bit = if let Some(last_range) = parsed_fields.iter().last() {
                last_range.attrs.bit_range.end
            } else {
                0_usize
            };
            let fill_bytes_size = (fill_bits - first_bit).div_ceil(8);
            let ident = quote::format_ident!("bondrewd_fill_bits");
            parsed_fields.push(FieldInfo {
                ident: Box::new(ident.into()),
                attrs: FieldAttrs {
                    bit_range: first_bit..fill_bits,
                    endianness: Box::new(Endianness::Big),
                    reserve: ReserveFieldOption::FakeReserveField,
                    overlap: OverlapOptions::None,
                    capture_id: false,
                },
                ty: FieldDataType::BlockArray(
                    Box::new(SubFieldInfo {
                        ty: FieldDataType::Number(1, NumberSignage::Unsigned, quote! {u8}),
                    }),
                    fill_bytes_size,
                    quote! {[u8;#fill_bytes_size]},
                ),
            });
        }

        if attrs.lsb_zero {
            for ref mut field in &mut parsed_fields {
                field.attrs.bit_range = (bit_size - field.attrs.bit_range.end)
                    ..(bit_size - field.attrs.bit_range.start);
            }
            parsed_fields.reverse();
        }

        Ok(parsed_fields)
    }
}
