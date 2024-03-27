use proc_macro2::Span;
use quote::quote;
use std::ops::Range;
use syn::Ident;

#[derive(Clone, Debug, Copy)]
pub enum Endianness {
    Little,
    Big,
    None,
}

#[derive(Clone, Debug)]
pub struct EndiannessInfo {
    inner: Endianness,
    aligned: bool,
}

impl EndiannessInfo {
    /// Are the bytes aligned to the bytes start and end, otherwise they are packed.
    pub fn is_aligned(&self) -> bool {
        self.aligned
    }
    pub fn has_endianness(&self) -> bool {
        !matches!(self.inner, Endianness::None)
    }
    // If the size provided is 1 or less bytes and endianess is not defined, the endianess will be
    // automatically become big endian which houses common 1 byte logic. if after that the endianess is none
    // `false` will be returned, if big or little endianess `true` will be returned.
    pub fn perhaps_endianness(&mut self, size: usize) -> bool {
        if let Endianness::None = self.inner {
            if size == 1 {
                let mut swap = Endianness::Big;
                std::mem::swap(&mut swap, &mut self.inner);
                true
            } else {
                false
            }
        } else {
            true
        }
    }
    pub fn endianess(&self) -> Endianness {
        self.inner
    }
    pub fn is_big(&self) -> bool {
        matches!(self.inner, Endianness::Big)
    }
    // pub fn is_little(&self) -> bool {
    //     matches!(self.inner, Endianness::Little)
    // }
    // pub fn is_none(&self) -> bool {
    //     matches!(self.inner, Endianness::None)
    // }
    pub fn big() -> Self {
        Self {
            inner: Endianness::Big,
            aligned: false,
        }
    }
    pub fn little() -> Self {
        Self {
            inner: Endianness::Little,
            aligned: false,
        }
    }
    pub fn none() -> Self {
        Self {
            inner: Endianness::None,
            aligned: false,
        }
    }
    pub fn set_aligned(&mut self, align: bool) {
        self.aligned = align;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumberSignage {
    Signed,
    Unsigned,
}

#[derive(Clone, Debug)]
pub enum DataType {
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
    ElementArray(Box<SubInfo>, usize, proc_macro2::TokenStream),
    BlockArray(Box<SubInfo>, usize, proc_macro2::TokenStream),
}

impl DataType {
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
    /// Do not suppress in `from_bytes` or `into_bytes`.
    NotReserve,
    /// User defined, meaning that the field shall not be written-to or read-from on `into_bytes` or
    /// `from_bytes` calls.
    ReserveField,
    /// used with imaginary fields that bondrewd creates, such as fill_bytes or variant_ids.
    /// these typically do not get any standard generated functions.
    FakeField,
    /// User defined, meaning that the field shall not be written to on `into_bytes` calls.
    ReadOnly,
}

impl ReserveFieldOption {
    pub fn wants_write_fns(&self) -> bool {
        match self {
            Self::ReadOnly | Self::FakeField | Self::ReserveField => false,
            Self::NotReserve => true,
        }
    }

    pub fn wants_read_fns(&self) -> bool {
        match self {
            Self::FakeField | Self::ReserveField => false,
            Self::NotReserve | Self::ReadOnly => true,
        }
    }

    pub fn is_fake_field(&self) -> bool {
        match self {
            Self::FakeField => true,
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
pub struct Attributes {
    pub endianness: Box<EndiannessInfo>,
    pub bit_range: Range<usize>,
    pub reserve: ReserveFieldOption,
    pub overlap: OverlapOptions,
    /// This should only ever be true on the Invalid case for enums that what to capture the invalid Id.
    pub capture_id: bool,
}

impl Attributes {
    pub fn bit_length(&self) -> usize {
        self.bit_range.end - self.bit_range.start
    }
}

#[derive(Clone, Debug)]
pub struct SubInfo {
    pub ty: DataType,
}

pub struct ElementSubFieldIter {
    pub outer_ident: Box<DynamicIdent>,
    pub endianness: Box<EndiannessInfo>,
    // this range is elements in the array, not bit range
    pub range: Range<usize>,
    pub starting_bit_index: usize,
    pub ty: DataType,
    pub element_bit_size: usize,
    pub reserve: ReserveFieldOption,
    pub overlap: OverlapOptions,
}

impl Iterator for ElementSubFieldIter {
    type Item = Info;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.range.next() {
            let start = self.starting_bit_index + (index * self.element_bit_size);
            let attrs = Attributes {
                bit_range: start..start + self.element_bit_size,
                endianness: self.endianness.clone(),
                reserve: self.reserve.clone(),
                overlap: self.overlap.clone(),
                capture_id: false,
            };
            let outer_ident = self.outer_ident.ident().clone();
            let name = quote::format_ident!("{}_{}", outer_ident, index);
            let ident = Box::new((outer_ident, name).into());
            Some(Info {
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
    pub outer_ident: Box<DynamicIdent>,
    pub endianness: Box<EndiannessInfo>,
    //array length
    pub length: usize,
    pub starting_bit_index: usize,
    pub ty: DataType,
    pub bit_length: usize,
    pub total_bytes: usize,
    pub reserve: ReserveFieldOption,
    pub overlap: OverlapOptions,
}

impl Iterator for BlockSubFieldIter {
    type Item = Info;
    fn next(&mut self) -> Option<Self::Item> {
        if self.length != 0 {
            let mut ty_size = self.ty.size() * 8;
            if self.bit_length % ty_size != 0 {
                ty_size = self.bit_length % ty_size;
            }
            let start = self.starting_bit_index;
            self.starting_bit_index = start + ty_size;
            let attrs = Attributes {
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
            Some(Info {
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
pub enum DynamicIdent {
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

impl DynamicIdent {
    pub fn ident(&self) -> Ident {
        match self {
            DynamicIdent::Ident { ident, name: _ } => ident.clone(),
            DynamicIdent::Index { index, name } => {
                Ident::new(&format!("field_{index}"), name.span())
            }
        }
    }
    pub fn name(&self) -> Ident {
        match self {
            DynamicIdent::Ident { ident: _, name } | DynamicIdent::Index { index: _, name } => {
                name.clone()
            }
        }
    }
    pub fn span(&self) -> Span {
        match self {
            DynamicIdent::Ident { ident, name: _ } => ident.span(),
            DynamicIdent::Index { index: _, name } => name.span(),
        }
    }
}

impl From<(usize, Span)> for DynamicIdent {
    fn from((value, span): (usize, Span)) -> Self {
        Self::Index {
            index: value,
            name: Ident::new(&format!("field_{value}"), span),
        }
    }
}

impl From<Ident> for DynamicIdent {
    fn from(value: Ident) -> Self {
        Self::Ident {
            ident: value.clone(),
            name: value,
        }
    }
}
impl From<(Ident, Ident)> for DynamicIdent {
    fn from((value, value2): (Ident, Ident)) -> Self {
        Self::Ident {
            ident: value,
            name: value2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Info {
    pub ident: Box<DynamicIdent>,
    pub ty: DataType,
    pub attrs: Attributes,
}

impl Info {
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
    pub fn ident(&self) -> &DynamicIdent {
        &self.ident
    }
    pub fn span(&self) -> Span {
        self.ident.span()
    }
    /// Returns `true` if `self` contains bits that overlap with `other`'s bits.
    pub fn overlapping(&self, other: &Self) -> bool {
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
        if let DataType::ElementArray(ref sub_field, ref array_length, _) = self.ty {
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
        if let DataType::BlockArray(ref sub_field, ref array_length, _) = self.ty {
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
}
