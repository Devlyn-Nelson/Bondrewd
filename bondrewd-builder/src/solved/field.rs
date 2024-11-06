use std::ops::Range;

use proc_macro2::Span;
use syn::Ident;

use crate::build::{
    field::{DataType, NumberType, RustByteSize},
    ArraySizings,
};

use super::field_set::{BuiltData, BuiltRangeType, SolvingError};

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
    pub fn from_ident(ident: Ident) -> Self {
        Self {
            bondrewd_name: ident.clone(),
            user_name: DynamicIdentName::Ident(ident),
        }
    }
    /// Returns a `DynamicIdent` for a tuple-struct-field's-index and the [`Span`]`
    /// of its type (so we can display error in a nice place).
    pub fn from_index(index: usize, span: Span) -> Self {
        Self {
            bondrewd_name: Ident::new(&format!("field_{index}"), span),
            user_name: DynamicIdentName::Index(index),
        }
    }
    /// Returns a `DynamicIdent` for a array's to create unique names for a byte_buffer for each element
    /// within an array.
    pub fn from_ident_with_name(ident: Ident, name: Ident) -> Self {
        Self {
            bondrewd_name: name,
            user_name: DynamicIdentName::Ident(ident),
        }
    }
    pub fn ident(&self) -> Ident {
        match &self.user_name {
            DynamicIdentName::Ident(ident) => ident.clone(),
            DynamicIdentName::Index(_) => self.bondrewd_name.clone(),
        }
    }
    pub fn name(&self) -> Ident {
        self.bondrewd_name.clone()
    }
    pub fn span(&self) -> Span {
        self.bondrewd_name.span()
    }
}

impl From<(usize, Span)> for DynamicIdent {
    fn from((index, span): (usize, Span)) -> Self {
        Self::from_index(index, span)
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

pub struct SolvedData {
    pub resolver: Resolver,
}

impl SolvedData {
    #[must_use]
    pub fn bit_length(&self) -> usize {
        self.resolver.bit_length()
    }
    pub fn generate_fn_quotes(&self) {
        todo!("Solved should get all of the generation code, without needing the Info structures.");
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
}

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

pub struct Resolver {
    pub(crate) data: Box<ResolverData>,
    pub(crate) ty: Box<ResolverType>,
}

impl Resolver {
    pub fn ident(&self) -> Ident {
        self.data.field_name.ident()
    }
    pub fn name(&self) -> Ident {
        self.data.field_name.name()
    }
    #[must_use]
    pub fn bit_length(&self) -> usize {
        self.data.bit_length()
    }
    #[must_use]
    pub fn starting_inject_byte(&self) -> usize {
        self.data.starting_inject_byte
    }
    #[must_use]
    pub fn available_bits_in_first_byte(&self) -> usize {
        self.data.available_bits_in_first_byte
    }
    #[must_use]
    pub fn zeros_on_left(&self) -> usize {
        self.data.zeros_on_left
    }
    #[must_use]
    pub fn fields_last_bits_index(&self) -> usize {
        self.bit_length().div_ceil(8) - 1
    }
    pub fn spans_multiple_bytes(&self) -> bool {
        self.bit_length() > self.data.available_bits_in_first_byte
    }
    #[must_use]
    pub fn field_buffer_name(&self) -> String {
        format!("{}_bytes", &self.data.field_name.name())
    }
}

#[derive(Debug, Clone)]
pub enum ResolverPrimitiveStrategy {
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
        ty_ident: String,
        rust_size: usize,
    },
    Array {
        sub_ty: ResolverSubType,
        array_ty: ResolverArrayType,
        sizings: ArraySizings,
    },
}

impl ResolverType {
    pub fn rust_size(&self) -> usize {
        match self  {
            ResolverType::Primitive { number_ty, resolver_strategy, rust_size } => rust_size.bytes(),
            ResolverType::Nested { ty_ident, rust_size } => *rust_size,
            ResolverType::Array { sub_ty, array_ty, sizings } => {
                let mut size = match sub_ty {
                    ResolverSubType::Primitive { number_ty, resolver_strategy, rust_size } => rust_size.bytes(),
                    ResolverSubType::Nested { ty_ident, rust_size } => *rust_size,
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
        ty_ident: String,
        rust_size: usize,
    },
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

impl From<BuiltData> for SolvedData {
    fn from(mut pre_field: BuiltData) -> Self {
        // TODO do auto_fill process. which just adds a implied reserve fields to structures that have a
        // bit size which has a non-zero remainder when divided by 8 (amount of bit in a byte). This shall
        // happen before byte_order_reversal and field_order_reversal
        //
        // Reverse field order
        let bit_size = pre_field.bit_range.bit_size();
        if pre_field.endianness.is_field_order_reversed() {
            let old_field_range = pre_field.bit_range.range().clone();
            pre_field.bit_range.bit_range =
                (bit_size - old_field_range.end)..(bit_size - old_field_range.start);
        }
        // get the total number of bits the field uses.
        let amount_of_bits = pre_field.bit_range.range().end - pre_field.bit_range.range().start;
        // amount of zeros to have for the right mask. (right mask meaning a mask to keep data on the
        // left)
        let mut zeros_on_left = pre_field.bit_range.range().start % 8;
        if 7 < zeros_on_left {
            // TODO if don't think this error is possible, and im wondering why it is being checked for
            // in the first place.
            // return Err(SolvingError::ResolverUnderflow(format!(
            //     "field \"{}\" would have had left shift underflow, report this at \
            //         https://github.com/Devlyn-Nelson/Bondrewd",
            //     pre_field.id.ident(),
            // )));
            zeros_on_left = zeros_on_left % 8;
        }
        let available_bits_in_first_byte = 8 - zeros_on_left;
        // calculate the starting byte index in the outgoing buffer
        let mut starting_inject_byte: usize = pre_field.bit_range.range().start / 8;
        // NOTE endianness is only for determining how to get the bytes we will apply to the output.
        // calculate how many of the bits will be inside the most significant byte we are adding to.
        // if pre_field.endianness.is_byte_order_reversed() {
        //     let struct_byte_length = bit_size / 8;
        //     starting_inject_byte = struct_byte_length - starting_inject_byte;
        // }

        let sub_ty = match pre_field.ty {
            DataType::Number(number_type, rust_byte_size) => {
                let resolver_strategy = if pre_field.endianness.is_alternative() {
                    ResolverPrimitiveStrategy::Alternate
                } else {
                    ResolverPrimitiveStrategy::Standard
                };
                ResolverSubType::Primitive {
                    number_ty: number_type,
                    resolver_strategy,
                    rust_size: rust_byte_size,
                }
            }
            DataType::Nested {
                ident,
                rust_byte_size,
            } => ResolverSubType::Nested {
                ty_ident: ident,
                rust_size: rust_byte_size,
            },
        };
        let ty = Box::new(match pre_field.bit_range.ty {
            BuiltRangeType::SingleElement => sub_ty.into(),
            BuiltRangeType::BlockArray(vec) => ResolverType::Array {
                array_ty: ResolverArrayType::Block,
                sizings: vec,
                sub_ty: sub_ty,
            },
            BuiltRangeType::ElementArray(vec) => ResolverType::Array {
                array_ty: ResolverArrayType::Element,
                sizings: vec,
                sub_ty: sub_ty,
            },
        });
        let resolver = Resolver {
            data: Box::new(ResolverData {
                bit_range: pre_field.bit_range.range().clone(),
                flip: if pre_field.endianness.is_byte_order_reversed() {
                    Some(pre_field.ty.rust_size() - 1)
                } else {
                    None
                },
                zeros_on_left,
                available_bits_in_first_byte,
                starting_inject_byte,
                field_name: pre_field.id,
            }),
            ty,
        };
        let new_field = SolvedData { resolver };
        new_field
    }
}
