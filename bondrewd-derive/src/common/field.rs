use super::Endianness;
use proc_macro2::Span;
use quote::quote;
use std::ops::Range;
use syn::Ident;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumberSignage {
    Signed,
    Unsigned,
}

#[derive(Clone, Debug)]
pub enum DataType {
    Boolean,
    Number {
        /// The fields rust-type size in bytes.
        size: usize,
        sign: NumberSignage,
        /// Quote containing the original type name.
        type_quote: proc_macro2::TokenStream,
    },
    Float {
        /// The fields rust-type size in bytes.
        size: usize,
        /// Quote containing the original type name.
        type_quote: proc_macro2::TokenStream,
    },
    Enum {
        /// Quote containing the original type name.
        type_quote: proc_macro2::TokenStream,
        /// The fields rust-type size in bytes.
        size: usize,
        /// quote containing the name or ident of the field.
        name_quote: proc_macro2::TokenStream,
    },
    Struct {
        /// The fields rust-type size in bytes.
        size: usize,
        /// quote containing the name or ident of the field.
        type_quote: proc_macro2::TokenStream,
    },
    Char {
        /// The fields rust-type size in bytes.
        size: usize,
        /// Quote containing the original type name.
        type_quote: proc_macro2::TokenStream,
    },
    ElementArray {
        /// Type information for the type contained in the array.
        sub_type: Box<SubInfo>,
        /// Amount of items in the array.
        length: usize,
        /// quote containing the array type and length.
        type_quote: proc_macro2::TokenStream,
    },
    BlockArray {
        /// Type information for the type contained in the array.
        sub_type: Box<SubInfo>,
        /// Amount of items in the array.
        length: usize,
        /// quote containing the array type and length.
        type_quote: proc_macro2::TokenStream,
    },
}

impl DataType {
    /// returns the byte size of actual rust type .
    pub fn size(&self) -> usize {
        match self {
            Self::Number { size, .. }
            | Self::Float { size, .. }
            | Self::Enum { size, .. }
            | Self::Struct { size, .. }
            | Self::Char { size, .. } => *size,
            Self::ElementArray {
                ref sub_type,
                length,
                ..
            }
            | Self::BlockArray {
                ref sub_type,
                length,
                ..
            } => sub_type.ty.size() * length,
            Self::Boolean => 1,
        }
    }
    /// a quote of the field's rust type
    pub fn type_quote(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Number { type_quote, .. }
            | Self::Float { type_quote, .. }
            | Self::Enum { type_quote, .. }
            | Self::Struct { type_quote, .. }
            | Self::Char { type_quote, .. }
            | Self::ElementArray { type_quote, .. }
            | Self::BlockArray { type_quote, .. } => type_quote.clone(),
            Self::Boolean => quote! {bool},
        }
    }
    /// Returns `true` if `self` a rust primitive number (u32, f64, etc.. ), a `char` or an array that
    /// has a base type of those.
    pub fn is_number(&self) -> bool {
        match self {
            Self::Enum { .. } | Self::Number { .. } | Self::Float { .. } | Self::Char { .. } => {
                true
            }
            Self::Boolean | Self::Struct { .. } => false,
            Self::ElementArray { sub_type, .. } | Self::BlockArray { sub_type, .. } => {
                sub_type.as_ref().ty.is_number()
            }
        }
    }
    /// returns the bit size, or in the case of arrays the inner most type's bit size.
    pub fn get_element_bit_length(&self) -> usize {
        match self {
            Self::Boolean => 1,
            Self::Char { .. } => 32,
            Self::Number { size, .. }
            | Self::Enum { size, .. }
            | Self::Float { size, .. }
            | Self::Struct { size, .. } => size * 8,
            Self::BlockArray { sub_type, .. } | Self::ElementArray { sub_type, .. } => {
                sub_type.as_ref().ty.get_element_bit_length()
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
    /// used for enum variant id field
    EnumId,
    /// User defined, meaning that the field shall not be written to on `into_bytes` calls.
    ReadOnly,
}

impl ReserveFieldOption {
    pub fn wants_write_fns(&self) -> bool {
        match self {
            Self::EnumId | Self::ReadOnly | Self::FakeField | Self::ReserveField => false,
            Self::NotReserve => true,
        }
    }

    pub fn wants_read_fns(&self) -> bool {
        match self {
            Self::EnumId | Self::FakeField | Self::ReserveField => false,
            Self::NotReserve | Self::ReadOnly => true,
        }
    }

    pub fn count_bits(&self) -> bool {
        match self {
            Self::FakeField => false,
            Self::EnumId | Self::ReserveField | Self::NotReserve | Self::ReadOnly => true,
        }
    }

    pub fn is_fake_field(&self) -> bool {
        match self {
            Self::EnumId | Self::FakeField => true,
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
    pub endianness: Box<Endianness>,
    pub bit_range: Range<usize>,
    pub reserve: ReserveFieldOption,
    pub overlap: OverlapOptions,
    /// This should only ever be true on the Invalid case for enums that what to capture the invalid Id.
    pub capture_id: bool,
}

impl Attributes {
    /// Returns the amount of bits the field should occupy in byte form.
    pub fn bit_length(&self) -> usize {
        self.bit_range.end - self.bit_range.start
    }
}

/// This type exists for the express purpose of stopping rust from complaining about
/// putting an enum type as one of its own fields. This is needed for arrays so...
#[derive(Clone, Debug)]
pub struct SubInfo {
    pub ty: DataType,
}

pub struct ElementSubFieldIter {
    pub outer_ident: Box<DynamicIdent>,
    pub endianness: Box<Endianness>,
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
    pub endianness: Box<Endianness>,
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

/// Used to make the handling of tuple structs vs named structs easier by removing the need to care.
#[derive(Clone, Debug)]
pub enum DynamicIdent {
    /// Named Field
    Ident {
        /// name of the field given by the user.
        ident: Ident,
        /// name of the value given by bondrewd.
        name: Ident,
    },
    /// Tuple Struct Field
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
            DynamicIdent::Ident { ident: _, name } => name.span(),
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

/// Stores information about a field and how it shall be represented in byte form.
#[derive(Clone, Debug)]
pub struct Info {
    /// The name of the field.
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
    // this returns how many bits of the fields pertain to total structure bits.
    // where as attrs.bit_length() give you bits the fields actually needs.
    pub fn bit_size_no_fill(&self) -> usize {
        if !self.attrs.reserve.count_bits() {
            return 0;
        }
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
    pub fn byte_size(&self) -> usize {
        self.ty.size()
    }

    pub fn get_element_iter(&self) -> Result<ElementSubFieldIter, syn::Error> {
        if let DataType::ElementArray {
            sub_type: ref sub_field,
            length: ref array_length,
            ..
        } = self.ty
        {
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
                "This field was trying to get used like an element array",
            ))
        }
    }

    pub fn get_block_iter(&self) -> Result<BlockSubFieldIter, syn::Error> {
        if let DataType::BlockArray {
            sub_type: ref sub_field,
            length: ref array_length,
            ..
        } = self.ty
        {
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
                "This field was trying to get used like a block array",
            ))
        }
    }
}

pub struct Solved {
    // TODO START_HERE make a solved bondrewd field that is used for generation and future bondrewd-builder
    // Basically we need to removed all usages of `FieldInfo` in `gen` and allow `Info` to be an
    // active builder we can use for bondrewd builder, then solve. bondrewd-derive would then
    // use `Solved` for its information and `bondrewd-builder` would use a `Solved` runtime api to
    // access bondrewd's bit-engine at runtime.
    //
    // Also the `fill_bits` that make enums variants expand to the largest variant size currently get added
    // after the byte-order-reversal. This would make it so the `Object` could: parse all of the variants
    // one at a at, until a solve function is called, which then grabs the largest variant, does a
    // auto-fill-bits operation on variants that need it, THEN solve the byte-order for all of them,
    // Each quote maker (multi-byte-le, single-byte-ne, there are 6 total) will become a FieldHandler
    // that can be used at runtime or be used by bondrewd-derive to construct its quotes.
}
