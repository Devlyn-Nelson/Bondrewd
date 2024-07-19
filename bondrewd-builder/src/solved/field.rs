use std::{fmt::Display, ops::Range};

use crate::build::field::NumberType;

use super::field_set::{BuiltData, SolvingError};

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
    pub(crate) fn get_resolver<Id: Clone + Copy + Display + PartialEq>(
        field: BuiltData<Id>,
    ) -> Result<Self, SolvingError> {
        let bit_range = &field.bit_range;

        let bit_length = bit_range.end - bit_range.start;
        let spans_multiple_bytes = (bit_range.start / 8) != (bit_range.end / 8);
        let name = format!("{}", field.id);
        let endianness = &field.endianness;
        let resolver = match &field.ty {
            crate::build::field::DataType::None => return Err(SolvingError::NoTypeProvided(name)),
            crate::build::field::DataType::Number(ty) => match endianness.mode() {
                crate::build::EndiannessMode::Alternative => {
                    if spans_multiple_bytes {
                        Resolver::multi_alt(
                            bit_range,
                            name.as_str(),
                            endianness.is_byte_order_reversed(),
                            ty.clone(),
                        )
                    } else {
                        Resolver::single_alt(
                            bit_range,
                            name.as_str(),
                            endianness.is_byte_order_reversed(),
                            ty.clone(),
                        )
                    }
                }
                crate::build::EndiannessMode::Standard => {
                    if spans_multiple_bytes {
                        Resolver::multi_standard(
                            bit_range,
                            name.as_str(),
                            endianness.is_byte_order_reversed(),
                            ty.clone(),
                        )
                    } else {
                        Resolver::single_standard(
                            bit_range,
                            name.as_str(),
                            endianness.is_byte_order_reversed(),
                            ty.clone(),
                        )
                    }
                }
            },
            #[cfg(feature = "derive")]
            crate::build::field::DataType::Nested(struct_name) => {
                let Some(e) = endianness else {
                    return Err(SolvingError::NoEndianness(name));
                };
                if spans_multiple_bytes {
                    Resolver::multi_nested(
                        &bit_range,
                        name.as_str(),
                        e.is_byte_order_reversed(),
                        struct_name,
                    )
                } else {
                    Resolver::single_nested(
                        &bit_range,
                        name.as_str(),
                        e.is_byte_order_reversed(),
                        struct_name,
                    )
                }
            }
        };
        if let Some(resolver) = resolver {
            Ok(resolver)
        } else {
            Err(SolvingError::ResolverUnderflow(name))
        }
    }
    /// If this returns `None` a zeros on left underflow was detected.
    fn single_nested<S: Into<String>>(
        bit_range: &Range<usize>,
        field_name: &str,
        bytes_reversed: bool,
        set_name: S,
    ) -> Option<Self> {
        Self::new(NewResolverData {
            bit_range,
            name: field_name,
            byte_order_reversed: bytes_reversed,
            ty: ResolverType::NestedSingle(set_name.into()),
        })
    }
    /// If this returns `None` a zeros on left underflow was detected.
    fn multi_nested<S: Into<String>>(
        bit_range: &Range<usize>,
        field_name: &str,
        bytes_reversed: bool,
        set_name: S,
    ) -> Option<Self> {
        Self::new(NewResolverData {
            bit_range,
            name: field_name,
            byte_order_reversed: bytes_reversed,
            ty: ResolverType::NestedMultiple(set_name.into()),
        })
    }
    /// If this returns `None` a zeros on left underflow was detected.
    fn single_standard(
        bit_range: &Range<usize>,
        field_name: &str,
        bytes_reversed: bool,
        data_ty: NumberType,
    ) -> Option<Self> {
        Self::new(NewResolverData {
            bit_range,
            name: field_name,
            byte_order_reversed: bytes_reversed,
            ty: ResolverType::StandardSingle(data_ty),
        })
    }
    /// If this returns `None` a zeros on left underflow was detected.
    fn multi_standard(
        bit_range: &Range<usize>,
        field_name: &str,
        bytes_reversed: bool,
        data_ty: NumberType,
    ) -> Option<Self> {
        Self::new(NewResolverData {
            bit_range,
            name: field_name,
            byte_order_reversed: bytes_reversed,
            ty: ResolverType::StandardMultiple(data_ty),
        })
    }
    /// If this returns `None` a zeros on left underflow was detected.
    fn single_alt(
        bit_range: &Range<usize>,
        field_name: &str,
        bytes_reversed: bool,
        data_ty: NumberType,
    ) -> Option<Self> {
        Self::new(NewResolverData {
            bit_range,
            name: field_name,
            byte_order_reversed: bytes_reversed,
            ty: ResolverType::AlternateSingle(data_ty),
        })
    }
    /// If this returns `None` a zeros on left underflow was detected.
    fn multi_alt(
        bit_range: &Range<usize>,
        field_name: &str,
        bytes_reversed: bool,
        data_ty: NumberType,
    ) -> Option<Self> {
        Self::new(NewResolverData {
            bit_range,
            name: field_name,
            byte_order_reversed: bytes_reversed,
            ty: ResolverType::AlternateMultiple(data_ty),
        })
    }
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
    #[cfg(feature = "derive")]
    pub fn field_buffer_name(&self) -> &str {
        self.field_buffer_name.as_str()
    }
    #[must_use]
    pub fn fields_last_bits_index(&self) -> usize {
        self.amount_of_bits.div_ceil(8) - 1
    }
    /// If this returns `None` a zeros on left underflow was detected.
    fn new(data: NewResolverData) -> Option<Self> {
        let bit_range = data.bit_range;
        let name = data.name;
        let ty = data.ty;
        let byte_order_reversed = data.byte_order_reversed;
        // get the total number of bits the field uses.
        let amount_of_bits = bit_range.end - bit_range.start;
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
            let starting_inject_byte: usize = bit_range.start / 8;
            // if let Some(flip) = byte_order_reversed {
            //     starting_inject_byte = *flip - starting_inject_byte;
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
                #[cfg(feature = "derive")]
                field_buffer_name,
                ty,
            })
        }
    }
}

pub enum ResolverType {
    StandardSingle(NumberType),
    StandardMultiple(NumberType),
    AlternateSingle(NumberType),
    AlternateMultiple(NumberType),
    NestedSingle(String),
    NestedMultiple(String),
}
