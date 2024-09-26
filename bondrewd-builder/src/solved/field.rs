use proc_macro2::Span;
use syn::Ident;

use crate::build::{field::NumberType, ArraySizings};

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
    // TODO make sure this happens.
    pub reverse_byte_order: bool,
    /// Amount of bits the field uses in bit form.
    pub amount_of_bits: usize,
    /// Amount of bits in the first byte this field has bits in that are not used by this field.
    pub zeros_on_left: usize,
    /// Amount of bits in the first byte this field has bits in that are used by this field.
    pub available_bits_in_first_byte: usize,
    /// the first byte this field is stored in
    pub starting_inject_byte: usize,
    pub field_name: DynamicIdent,
}

pub struct Resolver {
    pub(crate) data: ResolverData,
    pub(crate) ty: ResolverType,
}

impl Resolver {
    #[must_use]
    pub fn bit_length(&self) -> usize {
        self.data.amount_of_bits
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
        self.data.amount_of_bits.div_ceil(8) - 1
    }
    pub fn spans_multiple_bytes(&self) -> bool {
        self.data.amount_of_bits > self.data.available_bits_in_first_byte
    }
    #[must_use]
    pub fn field_buffer_name(&self) -> String {
        format!("{}_bytes", &self.data.field_name.name())
    }
    pub fn bit_range_start(&self) -> usize {
        (self.starting_inject_byte() * 8) + (8 - self.available_bits_in_first_byte())
    }
}

pub enum ResolverPrimitiveStrategy {
    Standard,
    Alternate,
}

pub enum ResolverArrayType {
    Element,
    Block,
}

pub enum ResolverType {
    Primitive {
        number_ty: NumberType,
        resolver_strategy: ResolverPrimitiveStrategy,
    },
    Nested {
        ty_ident: String,
    },
    Array {
        sub_ty: ResolverSubType,
        array_ty: ResolverArrayType,
        sizings: ArraySizings,
    },
}

pub enum ResolverSubType {
    Primitive {
        number_ty: NumberType,
        resolver_strategy: ResolverPrimitiveStrategy,
    },
    Nested {
        ty_ident: String,
    },
}

impl From<ResolverSubType> for ResolverType {
    fn from(value: ResolverSubType) -> Self {
        match value {
            ResolverSubType::Primitive {
                number_ty,
                resolver_strategy,
            } => ResolverType::Primitive {
                number_ty,
                resolver_strategy,
            },
            ResolverSubType::Nested { ty_ident } => ResolverType::Nested { ty_ident },
        }
    }
}
