use std::{ops::Range, str::FromStr};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    build::{
        field::{DataType, NumberType, RustByteSize},
        ArraySizings, OverlapOptions, ReserveFieldOption,
    },
    derive::GeneratedQuotes,
};

use super::field_set::{BuiltData, BuiltRangeType};

// Used to make the handling of tuple structs vs named structs easier by removing the need to care.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DynamicIdent {
    /// name of the value given by bondrewd.
    pub bondrewd_name: Ident,
    /// Original data from the user
    pub user_name: DynamicIdentName,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicIdentName {
    /// Named Field
    /// name of the field given by the user.
    Ident(Ident),
    /// Tuple Struct Field
    /// Index of the field in the tuple struct/enum-variant
    Index(usize),
}

impl DynamicIdent {
    /// Returns a `DynamicIdent` for a user-defined-field-name.
    #[must_use]
    pub fn from_ident(ident: Ident) -> Self {
        Self {
            bondrewd_name: ident.clone(),
            user_name: DynamicIdentName::Ident(ident),
        }
    }
    /// Returns a `DynamicIdent` for a tuple-struct-field's-index and the [`Span`]`
    /// of its type (so we can display error in a nice place).
    #[must_use]
    pub fn from_index(index: usize, span: Span) -> Self {
        Self {
            bondrewd_name: Ident::new(&format!("field_{index}"), span),
            user_name: DynamicIdentName::Index(index),
        }
    }
    /// Returns a `DynamicIdent` for a array's to create unique names for a `byte_buffer` for each element
    /// within an array.
    #[must_use]
    pub fn from_ident_with_name(ident: Ident, name: Ident) -> Self {
        Self {
            bondrewd_name: name,
            user_name: DynamicIdentName::Ident(ident),
        }
    }
    #[must_use]
    pub fn ident(&self) -> Ident {
        match &self.user_name {
            DynamicIdentName::Ident(ident) => ident.clone(),
            DynamicIdentName::Index(_) => self.bondrewd_name.clone(),
        }
    }
    #[must_use]
    pub fn name(&self) -> Ident {
        self.bondrewd_name.clone()
    }
    #[must_use]
    pub fn span(&self) -> Span {
        self.bondrewd_name.span()
    }
}

impl From<(usize, Span)> for DynamicIdent {
    fn from((index, span): (usize, Span)) -> Self {
        Self::from_index(index, span)
    }
}

impl From<&Ident> for DynamicIdent {
    fn from(ident: &Ident) -> Self {
        Self::from_ident(ident.clone())
    }
}
impl From<(&Ident, &Ident)> for DynamicIdent {
    fn from((ident, name): (&Ident, &Ident)) -> Self {
        Self::from_ident_with_name(ident.clone(), name.clone())
    }
}

impl From<Ident> for DynamicIdent {
    fn from(ident: Ident) -> Self {
        Self::from_ident(ident)
    }
}
impl From<(Ident, Ident)> for DynamicIdent {
    fn from((ident, name): (Ident, Ident)) -> Self {
        Self::from_ident_with_name(ident, name)
    }
}

#[derive(Debug)]
pub struct SolvedData {
    pub resolver: Resolver,
}

impl SolvedData {
    pub fn attr_capture_id(&self) -> bool {
        self.resolver.is_captured_id
    }
    pub fn attr_reserve(&self) -> &ReserveFieldOption {
        &self.resolver.reserve
    }
    #[must_use]
    pub fn bit_length(&self) -> usize {
        self.resolver.bit_length()
    }
    #[must_use]
    pub fn bit_range(&self) -> &Range<usize> {
        self.resolver.bit_range()
    }
    pub fn read(&self) {
        todo!(
            "Solved should use generation information to perform runtime getting/setting of bits"
        );
    }
    pub fn write(&self) {
        todo!(
            "Solved should use generation information to perform runtime getting/setting of bits"
        );
    }
    /// This will return a [`FieldQuotes`] which contains the code that goes into functions like:
    /// - `read_field`
    /// - `write_field`
    /// - `write_slice_field`
    /// - `StructChecked::read_field`
    ///
    /// More code, and the functions themselves, will be wrapped around this to insure it is safe.
    pub fn get_quotes(&self) -> syn::Result<GeneratedQuotes> {
        match self.resolver.ty.as_ref() {
            ResolverType::Nested {
                ty_ident,
                rust_size,
            } => self.get_ne_quotes(),
            ResolverType::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            } => match resolver_strategy.ty {
                ResolverPrimitiveStrategyTy::Standard => self.get_be_quotes(),
                ResolverPrimitiveStrategyTy::Alternate => self.get_le_quotes(),
            },
            ResolverType::Array {
                sub_ty,
                array_ty,
                sizings,
            } => match sub_ty {
                ResolverSubType::Primitive {
                    number_ty,
                    resolver_strategy,
                    rust_size,
                } => match resolver_strategy.ty {
                    ResolverPrimitiveStrategyTy::Standard => self.get_be_quotes(),
                    ResolverPrimitiveStrategyTy::Alternate => self.get_le_quotes(),
                },
                ResolverSubType::Nested {
                    ty_ident,
                    rust_size,
                } => self.get_ne_quotes(),
            },
        }
    }
    fn get_le_quotes(&self) -> Result<GeneratedQuotes, syn::Error> {
        let (read, write, clear) = {
            let read = self.resolver.get_read_quote(Resolver::get_read_le_quote)?;
            let (write, clear) = self
                .resolver
                .get_write_quote(Resolver::get_write_le_quote, false)?;
            (read, write, clear)
        };
        Ok(GeneratedQuotes {
            read,
            write,
            zero: clear,
        })
    }
    fn get_ne_quotes(&self) -> Result<GeneratedQuotes, syn::Error> {
        let (read, write, clear) = {
            // generate
            let read = self.resolver.get_read_quote(Resolver::get_read_ne_quote)?;
            let (write, clear) = self
                .resolver
                .get_write_quote(Resolver::get_write_ne_quote, false)?;
            (read, write, clear)
        };
        Ok(GeneratedQuotes {
            read,
            write,
            zero: clear,
        })
    }
    fn get_be_quotes(&self) -> Result<GeneratedQuotes, syn::Error> {
        let (read, write, clear) = {
            // generate
            let read = self.resolver.get_read_quote(Resolver::get_read_be_quote)?;
            let (write, clear) = self
                .resolver
                .get_write_quote(Resolver::get_write_be_quote, false)?;
            (read, write, clear)
        };
        Ok(GeneratedQuotes {
            read,
            write,
            zero: clear,
        })
    }
    pub fn from_built(mut pre_field: BuiltData, struct_bit_size: usize) -> Self {
        let flip = if pre_field.endianness.is_byte_order_reversed() {
            Some(struct_bit_size.div_ceil(8) - 1)
        } else {
            None
        };
        // TODO do auto_fill process. which just adds a implied reserve fields to structures that have a
        // bit size which has a non-zero remainder when divided by 8 (amount of bit in a byte). This shall
        // happen before byte_order_reversal and field_order_reversal
        //
        // Reverse field order
        if pre_field.endianness.is_field_order_reversed() && struct_bit_size != 0 {
            let reverse_val = struct_bit_size;
            let old_field_range = pre_field.bit_range.range().clone();
            let new_start = reverse_val - old_field_range.end;
            let new_end = reverse_val - old_field_range.start;
            // println!("{struct_bit_size} start {}, end {}", old_field_range.start, old_field_range.end);
            // println!("\tstart {new_start}, end {new_end}");
            pre_field.bit_range.bit_range = new_start..new_end;
        }
        // get the total number of bits the field uses.
        let amount_of_bits = pre_field.bit_range.range().end - pre_field.bit_range.range().start;
        // amount of zeros to have for the right mask. (right mask meaning a mask to keep data on the
        // left)
        let start = pre_field.bit_range.range().start;
        let mut zeros_on_left = start % 8;
        if 7 < zeros_on_left {
            // TODO if don't think this error is possible, and im wondering why it is being checked for
            // in the first place.
            // return Err(SolvingError::ResolverUnderflow(format!(
            //     "field \"{}\" would have had left shift underflow, report this at \
            //         https://github.com/Devlyn-Nelson/Bondrewd",
            //     pre_field.id.ident(),
            // )));
            zeros_on_left %= 8;
        }
        let available_bits_in_first_byte = 8 - zeros_on_left;
        // calculate the starting byte index in the outgoing buffer
        let starting_inject_byte: usize = start / 8;
        // NOTE endianness is only for determining how to get the bytes we will apply to the output.
        // calculate how many of the bits will be inside the most significant byte we are adding to.
        // if pre_field.endianness.is_byte_order_reversed() {
        //     let struct_byte_length = bit_size / 8;
        //     starting_inject_byte = struct_byte_length - starting_inject_byte;
        // }
        let sub_ty = match &pre_field.ty {
            DataType::Number(number_type, rust_byte_size) => {
                let resolver_strategy = ResolverPrimitiveStrategy {
                    ty: if pre_field.endianness.is_alternative() {
                        ResolverPrimitiveStrategyTy::Alternate
                    } else {
                        ResolverPrimitiveStrategyTy::Standard
                    },
                    fn_quote: pre_field.endianness.from_byte_endianness_fn_quote(),
                };
                ResolverSubType::Primitive {
                    number_ty: *number_type,
                    resolver_strategy,
                    rust_size: *rust_byte_size,
                }
            }
            DataType::Nested {
                ident,
                rust_byte_size,
            } => ResolverSubType::Nested {
                ty_ident: ident.clone(),
                rust_size: *rust_byte_size,
            },
        };
        let ty = Box::new(match &pre_field.bit_range.ty {
            BuiltRangeType::SingleElement => sub_ty.into(),
            BuiltRangeType::BlockArray(vec) => ResolverType::Array {
                array_ty: ResolverArrayType::Block,
                sizings: vec.clone(),
                sub_ty,
            },
            BuiltRangeType::ElementArray(vec) => ResolverType::Array {
                array_ty: ResolverArrayType::Element,
                sizings: vec.clone(),
                sub_ty,
            },
        });
        let resolver = Resolver {
            data: Box::new(ResolverData {
                bit_range: pre_field.bit_range.range().clone(),
                flip,
                zeros_on_left,
                available_bits_in_first_byte,
                starting_inject_byte,
                field_name: pre_field.id,
            }),
            ty,
            reserve: pre_field.reserve,
            is_captured_id: pre_field.is_captured_id,
            overlap: pre_field.overlap,
        };
        SolvedData { resolver }
    }
}

#[derive(Debug)]
pub struct ResolverData {
    /// Amount of bits in the first byte this field has bits in that are not used by this field.
    pub zeros_on_left: usize,
    /// Amount of bits in the first byte this field has bits in that are used by this field.
    pub available_bits_in_first_byte: usize,
    /// the first byte this field is stored in
    pub starting_inject_byte: usize,
    /// if the structure is flipped. (reverse the bytes order)
    pub flip: Option<usize>,
    pub field_name: DynamicIdent,
    pub bit_range: Range<usize>,
}

impl ResolverData {
    #[must_use]
    pub fn available_bits_in_first_byte(&self) -> usize {
        self.available_bits_in_first_byte
    }
}

#[derive(Debug)]
pub struct Resolver {
    pub(crate) data: Box<ResolverData>,
    pub(crate) ty: Box<ResolverType>,
    pub(crate) reserve: ReserveFieldOption,
    pub(crate) is_captured_id: bool,
    pub(crate) overlap: OverlapOptions,
}

impl Resolver {
    #[must_use]
    pub fn ident(&self) -> Ident {
        self.data.field_name.ident()
    }
    #[must_use]
    pub fn name(&self) -> Ident {
        self.data.field_name.name()
    }
    #[must_use]
    pub fn bit_length(&self) -> usize {
        self.data.bit_length()
    }
    #[must_use]
    pub fn bit_range(&self) -> &Range<usize> {
        self.data.bit_range()
    }
    #[must_use]
    pub fn starting_inject_byte(&self) -> usize {
        self.data.starting_inject_byte()
    }
    #[must_use]
    pub fn available_bits_in_first_byte(&self) -> usize {
        self.data.available_bits_in_first_byte()
    }
    #[must_use]
    pub fn zeros_on_left(&self) -> usize {
        self.data.zeros_on_left
    }
    #[must_use]
    pub fn fields_last_bits_index(&self) -> usize {
        self.bit_length().div_ceil(8) - 1
    }
    #[must_use]
    pub fn spans_multiple_bytes(&self) -> bool {
        self.bit_length() > self.data.available_bits_in_first_byte()
    }
    #[must_use]
    pub fn field_buffer_name(&self) -> String {
        format!("{}_bytes", &self.data.field_name.name())
    }
    #[must_use]
    pub fn field_buffer_ident(&self) -> Ident {
        format_ident!("{}", self.field_buffer_name())
    }
    pub fn get_resolved_ty(&self) -> ResolverSubType {
        match self.ty.as_ref() {
            ResolverType::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            } => ResolverSubType::Primitive {
                number_ty: *number_ty,
                resolver_strategy: resolver_strategy.clone(),
                rust_size: *rust_size,
            },
            ResolverType::Nested {
                ty_ident,
                rust_size,
            } => ResolverSubType::Nested {
                ty_ident: ty_ident.clone(),
                rust_size: *rust_size,
            },
            ResolverType::Array {
                sub_ty,
                array_ty: _,
                sizings: _,
            } => sub_ty.clone(),
        }
    }

    // this returns how many bits of the fields pertain to total structure bits.
    // where as attrs.bit_length() give you bits the fields actually needs.
    pub fn bit_size(&self) -> usize {
        if self.overlap.is_redundant() {
            0
        } else {
            let minus = if let OverlapOptions::Allow(skip) = self.overlap {
                skip
            } else {
                0
            };
            self.bit_length() - minus
        }
    }

    #[inline]
    // this returns how many bits of the fields pertain to total structure bits.
    // where as attrs.bit_length() give you bits the fields actually needs.
    pub fn bit_size_no_fill(&self) -> usize {
        if self.reserve.count_bits() && !self.is_captured_id {
            self.bit_size()
        } else {
            0
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolverPrimitiveStrategy {
    pub ty: ResolverPrimitiveStrategyTy,
    pub fn_quote: TokenStream,
}

#[derive(Debug, Clone)]
pub enum ResolverPrimitiveStrategyTy {
    Standard,
    Alternate,
}

#[derive(Debug, Clone)]
pub enum ResolverArrayType {
    Element,
    Block,
}

#[derive(Debug, Clone)]
pub enum ResolverType {
    Primitive {
        number_ty: NumberType,
        resolver_strategy: ResolverPrimitiveStrategy,
        rust_size: RustByteSize,
    },
    Nested {
        ty_ident: Ident,
        rust_size: usize,
    },
    Array {
        sub_ty: ResolverSubType,
        array_ty: ResolverArrayType,
        sizings: ArraySizings,
    },
}

pub fn get_number_type_ident(number_ty: &NumberType, bits: usize) -> syn::Result<TokenStream> {
    let span = Span::call_site();
    let pre = match number_ty {
        NumberType::Float => "f",
        NumberType::Unsigned => "u",
        NumberType::Signed => "i",
        NumberType::Char => return Ok(quote! {char}),
        NumberType::Bool => return Ok(quote! {bool}),
    };
    let thing = format!("{pre}{bits}");
    Ok(TokenStream::from_str(&thing)?)
}

impl ResolverType {
    pub fn is_nested(&self) -> bool {
        matches!(
            self,
            Self::Nested {
                ty_ident: _,
                rust_size: _
            }
        )
    }
    #[must_use]
    pub fn get_type_quote(&self) -> syn::Result<TokenStream> {
        match self {
            ResolverType::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            } => get_number_type_ident(number_ty, rust_size.bits()),
            ResolverType::Nested {
                ty_ident,
                rust_size,
            } => Ok(quote! {#ty_ident}),
            ResolverType::Array {
                sub_ty,
                array_ty,
                sizings,
            } => {
                let ty = sub_ty.get_type_ident();
                let mut ty = quote! {#ty};
                for size in sizings {
                    ty = quote! {[#ty;#size]};
                }
                Ok(ty)
            }
        }
    }
    #[must_use]
    pub fn rust_size(&self) -> usize {
        match self {
            ResolverType::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            } => rust_size.bytes(),
            ResolverType::Nested {
                ty_ident,
                rust_size,
            } => *rust_size,
            ResolverType::Array {
                sub_ty,
                array_ty,
                sizings,
            } => {
                let mut size = match sub_ty {
                    ResolverSubType::Primitive {
                        number_ty,
                        resolver_strategy,
                        rust_size,
                    } => rust_size.bytes(),
                    ResolverSubType::Nested {
                        ty_ident,
                        rust_size,
                    } => *rust_size,
                };
                for sizing in sizings {
                    size *= sizing;
                }
                size
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolverSubType {
    Primitive {
        number_ty: NumberType,
        resolver_strategy: ResolverPrimitiveStrategy,
        rust_size: RustByteSize,
    },
    Nested {
        ty_ident: Ident,
        rust_size: usize,
    },
}
impl ResolverSubType {
    #[must_use]
    pub fn get_type_ident(&self) -> Ident {
        let span = Span::call_site();
        match self {
            Self::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            } => {
                let pre = match number_ty {
                    NumberType::Float => "f",
                    NumberType::Unsigned => "u",
                    NumberType::Signed => "i",
                    NumberType::Char => return Ident::new("char", span),
                    NumberType::Bool => return Ident::new("bool", span),
                };
                let size = rust_size.bits();
                Ident::new(&format!("{pre}{size}"), span)
            }
            Self::Nested {
                ty_ident,
                rust_size,
            } => format_ident!("{ty_ident}"),
        }
    }
}

impl From<ResolverSubType> for ResolverType {
    fn from(value: ResolverSubType) -> Self {
        match value {
            ResolverSubType::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            } => ResolverType::Primitive {
                number_ty,
                resolver_strategy,
                rust_size,
            },
            ResolverSubType::Nested {
                ty_ident,
                rust_size,
            } => ResolverType::Nested {
                ty_ident,
                rust_size,
            },
        }
    }
}
