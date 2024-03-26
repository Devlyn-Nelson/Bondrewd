use crate::parse::field::FieldAttrBuilder;
use proc_macro2::Span;
use quote::quote;
use std::ops::Range;
use syn::parse::Error;
use syn::spanned::Spanned;
use syn::Ident;

use super::AttrInfo;

#[derive(Clone, Debug)]
pub enum Endianness {
    Little,
    Big,
    None,
}

impl Endianness {
    pub fn has_endianness(&self) -> bool {
        !matches!(self, Self::None)
    }
    // If the size provided is 1 or less bytes and endianess is not defined, the endianess will be
    // automatically become big endian which houses common 1 byte logic. if after that the endianess is none
    // `false` will be returned, if big or little endianess `true` will be returned.
    pub fn perhaps_endianness(&mut self, size: usize) -> bool {
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
    pub fn get_element_bit_length(&self) -> usize {
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

    /// `fields` should be all previous fields that have been parsed already.
    pub fn from_syn_field(
        field: &syn::Field,
        fields: &[FieldInfo],
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

        let attrs: FieldAttrs = match attrs_builder.try_into() {
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
