use std::ops::Range;

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

pub struct Resolver {
    /// Amount of bits the field uses in bit form.
    pub amount_of_bits: u8,
    /// Amount of bits in the first byte this field has bits in that are not used by this field.
    pub zeros_on_left: usize,
    /// Amount of bits in the first byte this field has bits in that are used by this field.
    pub available_bits_in_first_byte: usize,
    /// the first byte this field is stored in
    pub starting_inject_byte: usize,
    /// the name of the buffer we will use to store the data for the fields value.
    // TODO because the nested ty has the value this is solved from, it may make sense to
    // store only that moving this field into the `ResolverType` variants that do not.
    pub field_buffer_name: String,
    ty: ResolverType,
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
    /// If this returns `None` a zeros on left underflow was detected.
    pub(crate) fn single_nested<S: Into<String>>(
        bit_range: &Range<usize>,
        field_name: &str,
        set_name: S,
    ) -> Option<Self> {
        Self::new(
            bit_range,
            field_name,
            ResolverType::NestedSingle(set_name.into()),
        )
    }
    /// If this returns `None` a zeros on left underflow was detected.
    pub(crate) fn multi_nested<S: Into<String>>(
        bit_range: &Range<usize>,
        field_name: &str,
        set_name: S,
    ) -> Option<Self> {
        Self::new(
            bit_range,
            field_name,
            ResolverType::NestedMultiple(set_name.into()),
        )
    }
    /// If this returns `None` a zeros on left underflow was detected.
    pub(crate) fn single_standard(bit_range: &Range<usize>, field_name: &str) -> Option<Self> {
        Self::new(bit_range, field_name, ResolverType::StandardSingle)
    }
    /// If this returns `None` a zeros on left underflow was detected.
    pub(crate) fn multi_standard(bit_range: &Range<usize>, field_name: &str) -> Option<Self> {
        Self::new(bit_range, field_name, ResolverType::StandardMultiple)
    }
    /// If this returns `None` a zeros on left underflow was detected.
    pub(crate) fn single_alt(bit_range: &Range<usize>, field_name: &str) -> Option<Self> {
        Self::new(bit_range, field_name, ResolverType::AlternateSingle)
    }
    /// If this returns `None` a zeros on left underflow was detected.
    pub(crate) fn multi_alt(bit_range: &Range<usize>, field_name: &str) -> Option<Self> {
        Self::new(bit_range, field_name, ResolverType::AlternateMultiple)
    }
    pub fn bit_length(&self) -> u8 {
        self.amount_of_bits
    }
    pub fn starting_inject_byte(&self) -> usize {
        self.starting_inject_byte
    }
    pub fn available_bits_in_first_byte(&self) -> usize {
        self.available_bits_in_first_byte
    }
    pub fn zeros_on_left(&self) -> usize {
        self.zeros_on_left
    }
    pub fn field_buffer_name(&self) -> &str {
        self.field_buffer_name.as_str()
    }
    pub fn fields_last_bits_index(&self) -> usize {
        (self.amount_of_bits as usize).div_ceil(8) - 1
    }
    /// If this returns `None` a zeros on left underflow was detected.
    fn new(bit_range: &Range<usize>, name: &str, ty: ResolverType) -> Option<Self> {
        // get the total number of bits the field uses.
        let amount_of_bits = (bit_range.end - bit_range.start) as u8;
        // amount of zeros to have for the right mask. (right mask meaning a mask to keep data on the
        // left)
        let zeros_on_left = bit_range.start % 8;
        // NOTE endianness is only for determining how to get the bytes we will apply to the output.
        // calculate how many of the bits will be inside the most significant byte we are adding to.
        if 7 < zeros_on_left {
            // ne 8 - zeros_on_left = underflow
            None
        } else {
            let available_bits_in_first_byte = 8 - zeros_on_left;
            // calculate the starting byte index in the outgoing buffer
            let mut starting_inject_byte: usize = bit_range.start / 8;
            // TODO this code needs to be done to the range before it enters this function.
            // let flip = if let Some(flip) = flip {
            //     starting_inject_byte = flip - starting_inject_byte;
            //     Some(flip)
            // } else {
            //     None
            // };
            // make a name for the buffer that we will store the number in byte form
            let field_buffer_name = format!("{name}_bytes");

            Some(Self {
                amount_of_bits,
                zeros_on_left,
                available_bits_in_first_byte,
                starting_inject_byte,
                field_buffer_name,
                ty,
            })
        }
    }
}

pub enum ResolverType {
    StandardSingle,
    StandardMultiple,
    AlternateSingle,
    AlternateMultiple,
    NestedSingle(String),
    NestedMultiple(String),
}
