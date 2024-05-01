// Used to make the handling of tuple structs vs named structs easier by removing the need to care.
#[derive(Clone, Debug)]
pub enum DynamicIdent {
    /// Named Field
    Ident {
        /// name of the field given by the user.
        ident: String,
        /// name of the value given by bondrewd.
        name: String,
    },
    /// Tuple Struct Field
    Index {
        /// Index of the field in the tuple struct/enum-variant
        index: usize,
        /// name of the value given by bondrewd.
        name: String,
    },
}

pub struct SolvedData {
    pub resolver: Resolver,
}

impl SolvedData {
    pub fn bit_length(&self) -> u8 {
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

pub enum Resolver {
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
    StandardSingle(StandardSingle),
    StandardMultiple(StandardMultiple),
    AlternateSingle(AlternateSingle),
    AlternateMultiple(AlternateMultiple),
    NestedSingle(NestedSingle),
    NestedMultiple(NestedMultiple),
}

impl Resolver {
    pub fn bit_length(&self) -> u8 {
        match self {
            Resolver::StandardSingle(thing) => thing.amount_of_bits,
            Resolver::StandardMultiple(thing) => thing.amount_of_bits,
            Resolver::AlternateSingle(thing) => thing.amount_of_bits,
            Resolver::AlternateMultiple(thing) => thing.amount_of_bits,
            Resolver::NestedSingle(thing) => thing.amount_of_bits,
            Resolver::NestedMultiple(thing) => thing.amount_of_bits,
        }
    }
}

pub struct StandardSingle {
    /// Amount of bits the field uses in bit form.
    pub amount_of_bits: u8,
    /// Amount of bits in the first byte this field has bits in that are not used by this field.
    pub zeros_on_left: usize,
    /// Amount of bits in the first byte this field has bits in that are used by this field.
    pub available_bits_in_first_byte: usize,
    /// the first byte this field is stored in
    pub starting_inject_byte: usize,
    /// the name of the buffer we will use to store the data for the fields value.
    pub field_buffer_name: String,
    /// if the structure is flipped. (reverse the bytes order)
    pub flip: Option<usize>,
}
pub struct StandardMultiple {
    /// Amount of bits the field uses in bit form.
    pub amount_of_bits: u8,
    /// Amount of bits in the first byte this field has bits in that are not used by this field.
    pub zeros_on_left: usize,
    /// Amount of bits in the first byte this field has bits in that are used by this field.
    pub available_bits_in_first_byte: usize,
    /// the first byte this field is stored in
    pub starting_inject_byte: usize,
    /// the name of the buffer we will use to store the data for the fields value.
    pub field_buffer_name: String,
    /// if the structure is flipped. (reverse the bytes order)
    pub flip: Option<usize>,
}

pub struct AlternateSingle {
    /// Amount of bits the field uses in bit form.
    pub amount_of_bits: u8,
    /// Amount of bits in the first byte this field has bits in that are not used by this field.
    pub zeros_on_left: usize,
    /// Amount of bits in the first byte this field has bits in that are used by this field.
    pub available_bits_in_first_byte: usize,
    /// the first byte this field is stored in
    pub starting_inject_byte: usize,
    /// the name of the buffer we will use to store the data for the fields value.
    pub field_buffer_name: String,
    /// if the structure is flipped. (reverse the bytes order)
    pub flip: Option<usize>,
}
pub struct AlternateMultiple {
    /// Amount of bits the field uses in bit form.
    pub amount_of_bits: u8,
    /// Amount of bits in the first byte this field has bits in that are not used by this field.
    pub zeros_on_left: usize,
    /// Amount of bits in the first byte this field has bits in that are used by this field.
    pub available_bits_in_first_byte: usize,
    /// the first byte this field is stored in
    pub starting_inject_byte: usize,
    /// the name of the buffer we will use to store the data for the fields value.
    pub field_buffer_name: String,
    /// if the structure is flipped. (reverse the bytes order)
    pub flip: Option<usize>,
}

pub struct NestedSingle {
    /// Amount of bits the field uses in bit form.
    pub amount_of_bits: u8,
    /// Amount of bits in the first byte this field has bits in that are not used by this field.
    pub zeros_on_left: usize,
    /// Amount of bits in the first byte this field has bits in that are used by this field.
    pub available_bits_in_first_byte: usize,
    /// the first byte this field is stored in
    pub starting_inject_byte: usize,
    /// the name of the buffer we will use to store the data for the fields value.
    pub field_buffer_name: String,
    /// if the structure is flipped. (reverse the bytes order)
    pub flip: Option<usize>,
}
pub struct NestedMultiple {
    /// Amount of bits the field uses in bit form.
    pub amount_of_bits: u8,
    /// Amount of bits in the first byte this field has bits in that are not used by this field.
    pub zeros_on_left: usize,
    /// Amount of bits in the first byte this field has bits in that are used by this field.
    pub available_bits_in_first_byte: usize,
    /// the first byte this field is stored in
    pub starting_inject_byte: usize,
    /// the name of the buffer we will use to store the data for the fields value.
    pub field_buffer_name: String,
    /// if the structure is flipped. (reverse the bytes order)
    pub flip: Option<usize>,
}
