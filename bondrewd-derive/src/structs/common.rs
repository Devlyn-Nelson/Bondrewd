use crate::structs::parse::{
    FieldAttrBuilder, FieldAttrBuilderType, FieldBuilderRange, TryFromAttrBuilderError,
};
use proc_macro2::Span;
use quote::quote;
use std::ops::Range;
use syn::parse::Error;
use syn::{DeriveInput, Ident, Lit, Meta, NestedMeta, Type};

/// Returns a u8 mask with provided `num` amount of 1's on the left side (most significant bit)
pub fn get_left_and_mask(num: usize) -> u8 {
    match num {
        8 => 0b11111111,
        7 => 0b11111110,
        6 => 0b11111100,
        5 => 0b11111000,
        4 => 0b11110000,
        3 => 0b11100000,
        2 => 0b11000000,
        1 => 0b10000000,
        _ => 0b00000000,
    }
}

/// Returns a u8 mask with provided `num` amount of 1's on the right side (least significant bit)
pub fn get_right_and_mask(num: usize) -> u8 {
    match num {
        8 => 0b11111111,
        7 => 0b01111111,
        6 => 0b00111111,
        5 => 0b00011111,
        4 => 0b00001111,
        3 => 0b00000111,
        2 => 0b00000011,
        1 => 0b00000001,
        _ => 0b00000000,
    }
}

/// calculate the starting bit index for a field.
///
/// Returns the index of the byte the first bits of the field
///
/// # Arguments
/// * `amount_of_bits` - amount of bits the field will be after into_bytes.
/// * `right_rotation` - amount of bit Rotations to preform on the field. Note if rotation is not needed
///                         to retain all used bits then a shift could be used.
/// * `last_index` - total struct bytes size minus 1.
#[inline]
pub fn get_be_starting_index(
    amount_of_bits: usize,
    right_rotation: i8,
    last_index: usize,
) -> Result<usize, String> {
    //println!("be_start_index = [last;{}] - ([aob;{}] - [rs;{}]) / 8", last_index, amount_of_bits, right_rotation);
    let first = ((amount_of_bits as f64 - right_rotation as f64) / 8.0f64).ceil() as usize;
    if last_index < first {
        Err("the be_starting_index subtract underflow".to_string())
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
        if 8 < zeros_on_left {
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

    /// (amount_of_bits, zeros_on_left, available_bits_in_first_byte, starting_inject_byte)
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
        if let Self::None = self {
            false
        } else {
            true
        }
    }
}

#[derive(Clone, Debug)]
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
            Self::Number(ref size, _, _) => size.clone(),
            Self::Float(ref size, _) => size.clone(),
            Self::Enum(_, ref size, _) => size.clone(),
            Self::Struct(ref size, _) => size.clone(),
            Self::Char(ref size, _) => size.clone(),
            Self::ElementArray(ref fields, ref length, _) => fields.ty.size() * length,
            Self::BlockArray(ref fields, size, _) => fields.ty.size() * size,
            Self::Boolean => 1,
        }
    }

    pub fn type_quote(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Number(_, _, ref ident) => ident.clone(),
            Self::Float(_, ref ident) => ident.clone(),
            Self::Enum(_, _, ref ident) => ident.clone(),
            Self::Struct(_, ref ident) => ident.clone(),
            Self::Char(_, ref ident) => ident.clone(),
            Self::ElementArray(_, _, ref ident) => ident.clone(),
            Self::BlockArray(_, _, ident) => ident.clone(),
            Self::Boolean => quote! {bool},
        }
    }
    pub fn is_number(&self) -> bool {
        // TODO put Arrays in here
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
    pub fn parse(
        ty: &syn::Type,
        attrs: &mut FieldAttrBuilder,
        ident: &Ident,
        default_endianess: &Endianness,
    ) -> syn::Result<FieldDataType> {
        let data_type = match ty {
            Type::Path(ref path) => Self::parse_path(&path.path, attrs, ident.span())?,
            Type::Array(ref array_path) => {
                // arrays must use a literal for length, because its would be hard any other way.
                if let syn::Expr::Lit(ref lit_expr) = array_path.len {
                    if let syn::Lit::Int(ref lit_int) = lit_expr.lit {
                        if let Ok(array_length) = lit_int.base10_parse::<usize>() {
                            match attrs.ty {
                                FieldAttrBuilderType::ElementArray(element_bit_size) => {
                                    attrs.bit_range = match std::mem::take(&mut attrs.bit_range) {
                                        FieldBuilderRange::Range(range) => {
                                            if range.end < range.start {
                                                return Err(syn::Error::new(
                                                    ident.span(),
                                                    "range end is less than range start",
                                                ));
                                            }
                                            if range.end - range.start
                                                != element_bit_size * array_length
                                            {
                                                return Err(
                                                    syn::Error::new(
                                                        ident.span(),
                                                        "Element arrays bit range didn't match (element bit size * array length)"
                                                    )
                                                );
                                            }
                                            FieldBuilderRange::Range(range)
                                        }
                                        FieldBuilderRange::LastEnd(last_end) => {
                                            FieldBuilderRange::Range(
                                                last_end
                                                    ..last_end + (array_length * element_bit_size),
                                            )
                                        }
                                        _ => {
                                            return Err(syn::Error::new(
                                                ident.span(),
                                                "failed getting Range for element array",
                                            ));
                                        }
                                    };
                                    let mut sub_attrs = attrs.clone();
                                    if let Type::Array(_) = array_path.elem.as_ref() {
                                    } else {
                                        sub_attrs.ty = FieldAttrBuilderType::None;
                                    }
                                    let sub_ty = Self::parse(
                                        &array_path.elem,
                                        &mut sub_attrs,
                                        &ident,
                                        default_endianess,
                                    )?;

                                    let type_ident = &sub_ty.type_quote();
                                    FieldDataType::ElementArray(
                                        Box::new(SubFieldInfo { ty: sub_ty }),
                                        array_length,
                                        quote! {[#type_ident;#array_length]},
                                    )
                                }
                                FieldAttrBuilderType::BlockArray => {
                                    let mut sub_attrs = attrs.clone();
                                    if let Type::Array(_) = array_path.elem.as_ref() {
                                    } else {
                                        sub_attrs.ty = FieldAttrBuilderType::None;
                                    }

                                    let sub_ty = Self::parse(
                                        &array_path.elem,
                                        &mut sub_attrs,
                                        &ident,
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
                                _ => {
                                    return Err(Error::new(
                                        array_path.bracket_token.span,
                                        "Please Use array-bit-length (Bit-Block) or element-bit-length (List of nameless Fields of the same type) for defining array packing behavior",
                                    ));
                                }
                            }
                        } else {
                            return Err(Error::new(
                                array_path.bracket_token.span,
                                "failed parsing array length as literal integer",
                            ));
                        }
                    } else {
                        return Err(Error::new(array_path.bracket_token.span, "Couldn't determine Array length, literal array lengths must be an integer"));
                    }
                } else {
                    return Err(Error::new(
                        array_path.bracket_token.span,
                        "Couldn't determine Array length, must be literal",
                    ));
                }
            }
            _ => {
                return Err(Error::new(ident.span(), "Unsupported field type"));
            }
        };
        // if the type is a number and its endianess is None (numbers should have endianess) then we
        // apply the structs default (which might also be None)
        if data_type.is_number() {
            if !attrs.endianness.has_endianness() {
                if default_endianess.has_endianness() {
                    attrs.endianness = Box::new(default_endianess.clone());
                } else {
                    return Err(Error::new(ident.span(), "field without defined endianess found, please set endianess of struct or fields"));
                }
            }
        }

        Ok(data_type)
    }

    fn parse_path(
        path: &syn::Path,
        attrs: &mut FieldAttrBuilder,
        field_span: Span,
    ) -> syn::Result<FieldDataType> {
        // TODO added attribute consideration for recognizing structs and enums.
        // TODO impl enum logic.
        // TODO impl struct logic
        match attrs.ty {
            FieldAttrBuilderType::None => {
                if let Some(last_segment) = path.segments.last() {
                    let type_quote = &last_segment.ident;
                    let field_type_name = last_segment.ident.to_string();
                    match field_type_name.as_str() {
                        "bool" => match attrs.bit_range {
                            FieldBuilderRange::LastEnd(start) => {
                                attrs.bit_range =
                                    FieldBuilderRange::Range(start.clone()..start + 1);
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
                            NumberSignage::Signed,
                            quote! {#type_quote},
                        )),
                        "i16" => Ok(FieldDataType::Number(
                            2,
                            NumberSignage::Signed,
                            quote! {#type_quote},
                        )),
                        "f32" => Ok(FieldDataType::Float(4, quote! {#type_quote})),
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
                        "f64" => Ok(FieldDataType::Float(8, quote! {#type_quote})),
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
                            format!("usize and isize are not supported due to ambiguous sizing"),
                        )),
                        _ => Err(Error::new(
                            field_span,
                            format!("unknown primitive type [{}]", field_type_name),
                        )),
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
pub struct FieldAttrs {
    pub endianness: Box<Endianness>,
    pub bit_range: Range<usize>,
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
    pub outer_ident: Box<Ident>,
    pub endianness: Box<Endianness>,
    // this range is elements in the array, not bit range
    pub range: Range<usize>,
    pub starting_bit_index: usize,
    pub ty: FieldDataType,
    pub outer_name: proc_macro2::TokenStream,
    pub element_bit_size: usize,
}

impl Iterator for ElementSubFieldIter {
    type Item = FieldInfo;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.range.next() {
            let start = self.starting_bit_index + (index * self.element_bit_size);
            let attrs = FieldAttrs {
                bit_range: start..start + self.element_bit_size,
                endianness: self.endianness.clone(),
            };
            let mut name = self.outer_name.clone();
            name = quote! {#name[#index]};
            Some(FieldInfo {
                ident: self.outer_ident.clone(),
                attrs,
                name,
                ty: self.ty.clone(),
            })
        } else {
            None
        }
    }
}

pub struct BlockSubFieldIter {
    pub outer_ident: Box<Ident>,
    pub endianness: Box<Endianness>,
    //array length
    pub length: usize,
    pub starting_bit_index: usize,
    pub ty: FieldDataType,
    pub outer_name: proc_macro2::TokenStream,
    pub bit_length: usize,
}

impl Iterator for BlockSubFieldIter {
    type Item = FieldInfo;
    fn next(&mut self) -> Option<Self::Item> {
        if self.length != 0 {
            self.length -= 1;
            let mut ty_size = self.ty.size() * 8;
            if ty_size > self.bit_length {
                ty_size = self.bit_length;
            }
            let start = if self.length > 0 {
                self.starting_bit_index
                    + (self.bit_length % ty_size)
                    + ((self.length - 1) * ty_size)
            } else {
                self.starting_bit_index
            };
            let attrs = FieldAttrs {
                bit_range: start..start + ty_size,
                endianness: self.endianness.clone(),
            };
            self.bit_length -= ty_size;
            let index = self.length;
            let mut name = self.outer_name.clone();
            name = quote! {#name[#index]};
            Some(FieldInfo {
                ident: self.outer_ident.clone(),
                attrs,
                name,
                ty: self.ty.clone(),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub name: proc_macro2::TokenStream,
    pub ident: Box<Ident>,
    pub ty: FieldDataType,
    pub attrs: FieldAttrs,
}

impl FieldInfo {
    fn overlapping(&self, other: &Self) -> bool {
        // check that self's start is not within other's range
        if self.attrs.bit_range.start >= other.attrs.bit_range.start {
            if self.attrs.bit_range.start == other.attrs.bit_range.start
                || self.attrs.bit_range.start < other.attrs.bit_range.end
            {
                return true;
            }
        }
        // check that other's start is not within self's range
        if other.attrs.bit_range.start >= self.attrs.bit_range.start {
            if other.attrs.bit_range.start == self.attrs.bit_range.start
                || other.attrs.bit_range.start < self.attrs.bit_range.end
            {
                return true;
            }
        }
        if self.attrs.bit_range.end > other.attrs.bit_range.start {
            if self.attrs.bit_range.end <= other.attrs.bit_range.end {
                return true;
            }
        }
        if other.attrs.bit_range.end > self.attrs.bit_range.start {
            if other.attrs.bit_range.end <= self.attrs.bit_range.end {
                return true;
            }
        }
        false
    }

    #[inline]
    pub fn bit_size(&self) -> usize {
        self.attrs.bit_range.end - self.attrs.bit_range.start
    }

    #[inline]
    pub fn struct_byte_size(&self) -> usize {
        self.ty.size()
    }

    pub fn get_element_iter(&self) -> Result<ElementSubFieldIter, syn::Error> {
        if let FieldDataType::ElementArray(ref sub_field, ref array_length, _) = self.ty {
            Ok(ElementSubFieldIter {
                outer_name: self.name.clone(),
                outer_ident: self.ident.clone(),
                endianness: self.attrs.endianness.clone(),
                element_bit_size: (self.attrs.bit_range.end - self.attrs.bit_range.start)
                    / array_length,
                starting_bit_index: self.attrs.bit_range.start,
                range: 0..*array_length,
                ty: sub_field.ty.clone(),
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
                outer_name: self.name.clone(),
                outer_ident: self.ident.clone(),
                endianness: self.attrs.endianness.clone(),
                bit_length,
                starting_bit_index: self.attrs.bit_range.start,
                length: array_length.clone(),
                ty: sub_field.ty.clone(),
            })
        } else {
            Err(syn::Error::new(
                self.ident.span(),
                "This field was trying to get used like an array",
            ))
        }
    }

    pub fn from_syn_field(field: &syn::Field, struct_info: &StructInfo) -> syn::Result<Self> {
        let ident: Box<Ident> = if let Some(ref name) = field.ident {
            Box::new(name.clone())
        } else {
            return Err(Error::new(Span::call_site(), "all fields must be named"));
        };
        // parse all attrs. which will also give us the bit locations
        let mut attrs_builder =
            FieldAttrBuilder::parse(&field, struct_info.fields.last(), ident.clone())?;
        // check the field for supported types.
        let data_type = FieldDataType::parse(
            &field.ty,
            &mut attrs_builder,
            &ident,
            &struct_info.default_endianess,
        )?;

        let attr_result: std::result::Result<FieldAttrs, TryFromAttrBuilderError> =
            attrs_builder.try_into();

        let attrs = match attr_result {
            Ok(attr) => attr,
            Err(fix_me) => {
                let mut start = 0;
                if let Some(last_value) = struct_info.fields.last() {
                    start = last_value.attrs.bit_range.end;
                }
                fix_me.fix(start..start + (data_type.size() * 8))
            }
        };

        // construct the field we are parsed.
        let new_field = FieldInfo {
            name: quote! {#ident},
            ident: ident.clone(),
            ty: data_type,
            attrs,
        };
        // check to verify there are no overlapping bit ranges from previously parsed fields.
        for (parsed_field, i) in struct_info.fields.iter().zip(0..struct_info.fields.len()) {
            if parsed_field.overlapping(&new_field) {
                return Err(Error::new(
                    Span::call_site(),
                    format!("fields {} and {} overlap", i, struct_info.fields.len()),
                ));
            }
        }

        Ok(new_field)
    }
}

pub enum StructEnforcement {
    /// there is no enforcement so if bits are unused then it will act like they are a reserve field
    NoRules,
    /// enforce the BIT_SIZE equals BYTE_SIZE * 8
    EnforceFullBytes,
    /// enforce an amount of bits total that need to be used.
    EnforceBitAmount(usize),
}

pub struct StructInfo {
    pub name: Ident,
    /// if false then bit 0 is the Most Significant Bit meaning the first values first bit will start there.
    /// if true then bit 0 is the Least Significant Bit (the last bit in the last byte).
    pub lsb_zero: bool,
    /// flip all the bytes, like .reverse() for vecs or arrays. but we do that here because we can do
    /// it with no runtime cost.
    pub flip: bool,
    pub enforcement: StructEnforcement,
    pub fields: Vec<FieldInfo>,
    pub default_endianess: Endianness,
}

impl StructInfo {
    pub fn total_bits(&self) -> usize {
        let mut total: usize = 0;
        for field in self.fields.iter() {
            total += field.attrs.bit_length();
        }
        total
    }

    pub fn total_bytes(&self) -> usize {
        (self.total_bits() as f64 / 8.0f64).ceil() as usize
    }
    fn parse_struct_attrs_meta(info: &mut StructInfo, meta: Meta) -> Result<(), syn::Error> {
        match meta {
            Meta::NameValue(value) => {
                if value.path.is_ident("read_from") {
                    if let Lit::Str(val) = value.lit {
                        match val.value().as_str() {
                            "lsb0" => info.lsb_zero = true,
                            "msb0" => info.lsb_zero = false,
                            _ => return Err(Error::new(
                                val.span(),
                                "Expected literal str \"lsb0\" or \"msb0\" for read_from attribute.",
                            )),
                        }
                    }
                } else if value.path.is_ident("default_endianness") {
                    if let Lit::Str(val) = value.lit {
                        match val.value().as_str() {
                            "le" | "lsb" | "little" | "lil" => {
                                info.default_endianess = Endianness::Little
                            }
                            "be" | "msb" | "big" => info.default_endianess = Endianness::Big,
                            "ne" | "native" => info.default_endianess = Endianness::None,
                            _ => {}
                        }
                    }
                } else if value.path.is_ident("enforce_bytes") {
                    if let Lit::Int(val) = value.lit {
                        match val.base10_parse::<usize>() {
                            Ok(value) => {
                                info.enforcement = StructEnforcement::EnforceBitAmount(value * 8);
                            }
                            Err(err) => {
                                return Err(syn::Error::new(
                                    info.name.span(),
                                    format!("failed parsing enforce_bytes value [{}]", err),
                                ))
                            }
                        }
                    }
                } else if value.path.is_ident("enforce_bits") {
                    if let Lit::Int(val) = value.lit {
                        match val.base10_parse::<usize>() {
                            Ok(value) => {
                                info.enforcement = StructEnforcement::EnforceBitAmount(value);
                            }
                            Err(err) => {
                                return Err(syn::Error::new(
                                    info.name.span(),
                                    format!("failed parsing enforce_bytes value [{}]", err),
                                ))
                            }
                        }
                    }
                }
            }
            Meta::Path(value) => {
                if let Some(ident) = value.get_ident() {
                    match ident.to_string().as_str() {
                        "flip" => {
                            info.flip = true;
                        }
                        "enforce_full_bytes" => {
                            info.enforcement = StructEnforcement::EnforceFullBytes;
                        }
                        _ => {}
                    }
                }
            }
            Meta::List(meta_list) => {
                if meta_list.path.is_ident("bondrewd") {
                    for nested_meta in meta_list.nested {
                        match nested_meta {
                            NestedMeta::Meta(meta) => {
                                Self::parse_struct_attrs_meta(info, meta)?;
                            }
                            NestedMeta::Lit(_) => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub fn parse(input: &DeriveInput) -> syn::Result<StructInfo> {
        // get the struct, error out if not a struct
        let data = match input.data {
            syn::Data::Struct(ref data) => data,
            _ => {
                return Err(Error::new(Span::call_site(), "input must be a struct"));
            }
        };
        let mut info = StructInfo {
            name: input.ident.clone(),
            lsb_zero: false,
            flip: false,
            enforcement: StructEnforcement::NoRules,
            fields: Default::default(),
            default_endianess: Endianness::None,
        };
        for attr in input.attrs.iter() {
            let meta = attr.parse_meta()?;
            Self::parse_struct_attrs_meta(&mut info, meta)?;
        }
        // get the list of fields in syn form, error out if unit struct (because they have no data, and
        // data packing/analysis don't seem necessary)
        let fields = match data.fields {
            syn::Fields::Named(ref named_fields) => named_fields.named.iter().map(|x| x.clone()).collect::<Vec<syn::Field>>(),
            syn::Fields::Unnamed(ref fields) => fields.unnamed.iter().map(|x| x.clone()).collect::<Vec<syn::Field>>(),
            syn::Fields::Unit => return Err(Error::new(data.struct_token.span, "Packing a Unit Struct (Struct with no data) seems pointless to me, so i didn't write code for it.")),
        };

        // figure out what the field are and what/where they should be in byte form.
        let mut bit_size = 0;
        for ref field in fields {
            let parsed_field = FieldInfo::from_syn_field(field, &info)?;
            bit_size += parsed_field.bit_size();
            info.fields.push(parsed_field);
        }

        match info.enforcement {
            StructEnforcement::NoRules => {}
            StructEnforcement::EnforceFullBytes => {
                if bit_size % 8 != 0 {
                    return Err(syn::Error::new(
                        info.name.span(),
                        "BIT_SIZE modulus 8 is not zero",
                    ));
                }
            }
            StructEnforcement::EnforceBitAmount(expected_total_bits) => {
                if bit_size != expected_total_bits {
                    return Err(syn::Error::new(
                        info.name.span(),
                        format!(
                            "Bit Size Enforcement [{} != {}]",
                            expected_total_bits, bit_size
                        ),
                    ));
                }
            }
        }

        if info.lsb_zero {
            for ref mut field in info.fields.iter_mut() {
                field.attrs.bit_range = (bit_size - field.attrs.bit_range.end)
                    ..(bit_size - field.attrs.bit_range.start);
            }
            info.fields.reverse();
        }
        Ok(info)
    }
}
