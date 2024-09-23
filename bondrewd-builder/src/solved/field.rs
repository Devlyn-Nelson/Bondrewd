use std::ops::Range;

use crate::build::field::NumberType;

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

pub enum ResolverType {
    Standard(NumberType),
    Alternate(NumberType),
    Nested(String),
}
