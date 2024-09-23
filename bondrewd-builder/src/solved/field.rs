use std::ops::Range;

use crate::build::{field::NumberType, ArraySizings};

// Used to make the handling of tuple structs vs named structs easier by removing the need to care.
#[derive(Clone, Debug)]
pub struct DynamicIdent {
    /// name of the value given by bondrewd.
    pub name: String,
    pub ty: DynamicIdentType,
}

#[derive(Clone, Debug)]
pub enum DynamicIdentType {
    /// Named Field
    /// name of the field given by the user.
    Ident(String),
    /// Tuple Struct Field
    /// Index of the field in the tuple struct/enum-variant
    Index(usize),
}

impl DynamicIdent {
    pub fn new_ident(name: String, ident: String) -> Self {
        Self {
            name,
            ty: DynamicIdentType::Ident(ident),
        }
    }
    pub fn new_index(name: String, index: usize) -> Self {
        Self {
            name,
            ty: DynamicIdentType::Index(index),
        }
    }

    pub fn ident(&self) -> String {
        match &self.ty {
            DynamicIdentType::Ident(ident) => ident.to_owned(),
            DynamicIdentType::Index(index) => format!("{index}"),
        }
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

pub struct NewResolverData<'a> {
    pub bit_range: &'a Range<usize>,
    pub name: &'a str,
    pub ty: ResolverType,
    pub byte_order_reversed: bool,
}

pub struct Resolver {
    /// Amount of bits the field uses in bit form.
    pub amount_of_bits: usize,
    /// Amount of bits in the first byte this field has bits in that are not used by this field.
    pub zeros_on_left: usize,
    /// Amount of bits in the first byte this field has bits in that are used by this field.
    pub available_bits_in_first_byte: usize,
    /// the first byte this field is stored in
    pub starting_inject_byte: usize,
    /// the name of the buffer we will use to store the data for the fields value.
    // TODO because the nested ty has the value this is solved from, it may make sense to
    // store only that moving this field into the `ResolverType` variants that do not.
    #[cfg(feature = "derive")]
    pub field_buffer_name: String,
    pub(crate) ty: ResolverType,
    // TODO make sure this happens.
    pub reverse_byte_order: bool,
}

impl Resolver {
    #[must_use]
    pub fn bit_length(&self) -> usize {
        self.amount_of_bits
    }
    #[must_use]
    pub fn starting_inject_byte(&self) -> usize {
        self.starting_inject_byte
    }
    #[must_use]
    pub fn available_bits_in_first_byte(&self) -> usize {
        self.available_bits_in_first_byte
    }
    #[must_use]
    pub fn zeros_on_left(&self) -> usize {
        self.zeros_on_left
    }
    #[must_use]
    pub fn fields_last_bits_index(&self) -> usize {
        self.amount_of_bits.div_ceil(8) - 1
    }
    pub fn spans_multiple_bytes(&self) -> bool {
        self.amount_of_bits > self.available_bits_in_first_byte
    }
    #[must_use]
    #[cfg(feature = "derive")]
    pub fn field_buffer_name(&self) -> &str {
        self.field_buffer_name.as_str()
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
