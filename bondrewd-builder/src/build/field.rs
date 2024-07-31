use super::{BuilderRange, Endianness, OverlapOptions, ReserveFieldOption};

/// Defines how each element of an array should be treated.
#[derive(Debug, Clone)]
pub enum ArrayType {
    /// Each element of the array is considered its own value.
    ///
    /// Good for data, or a collection of datums.
    Element,
    /// All of the useful bits/bytes in the array describe a singular piece of information.
    ///
    /// Good for Strings, or a series of bits/bytes that make 1 datum.
    Block,
}

///
#[derive(Debug, Clone)]
pub(crate) struct ArrayInfo {
    ty: ArrayType,
    /// Each element represents a dimension to the array with the value being the amount of elements
    /// for that dimension.
    ///
    /// # Examples
    /// a single dimensional array would only have 1 value
    /// |      |Element 1|
    /// |:-----|:-------:|
    /// |[u8;4]|        4|
    ///
    /// X dimensional array will have X values, first being the outer-most array size going
    /// to the inner-most.
    ///
    /// |          |Element 1|Element 2|
    /// |:---------|:-------:|:-------:|
    /// |[[u8;4];5]|        5|        4|
    sizings: Vec<usize>,
}

#[derive(Debug)]
pub struct DataBuilder<Id>
where
    Id: Clone + Copy,
{
    /// The name or ident of the field.
    pub(crate) id: Id,
    /// The approximate data type of the field. when solving, this must be
    /// filled.
    pub(crate) ty: DataType,
    pub(crate) endianness: Option<Endianness>,
    /// Size of the rust native type in bytes (should never be zero)
    pub(crate) rust_size: RustByteSize,
    /// Defines if this field is an array or not.
    /// If `None` this data is not in an array and should just be treated as a single value.
    ///
    /// If `Some` than this is an array, NOT a single value. Also Note that the `ty` and `rust_size` only
    /// describe a true data type, which would be the innermost part of an array. The array info
    /// is marly keeping track of the order and magnitude of the array and its dimensions.
    pub(crate) array: Option<ArrayInfo>,
    /// The range of bits that this field will use.
    pub(crate) bit_range: BuilderRange,
    /// Describes when the field should be considered.
    pub(crate) reserve: ReserveFieldOption,
    /// How much you care about the field overlapping other fields.
    pub(crate) overlap: OverlapOptions,
}

#[derive(Debug, Clone, Copy)]
pub enum RustByteSize {
    Unknown,
    One,
    Two,
    Four,
    Eight,
    Sixteen,
}

impl RustByteSize {
    pub fn bytes(&self) -> usize {
        match self {
            RustByteSize::Unknown => 0,
            RustByteSize::One => 1,
            RustByteSize::Two => 2,
            RustByteSize::Four => 4,
            RustByteSize::Eight => 8,
            RustByteSize::Sixteen => 16,
        }
    }
    pub fn bits(&self) -> usize {
        match self {
            RustByteSize::Unknown => 0,
            RustByteSize::One => 8,
            RustByteSize::Two => 16,
            RustByteSize::Four => 32,
            RustByteSize::Eight => 64,
            RustByteSize::Sixteen => 128,
        }
    }
}

impl<Id> DataBuilder<Id>
where
    Id: Clone + Copy,
{
    pub fn new(name: Id, ty: DataType) -> Self {
        Self {
            id: name,
            ty,
            rust_size: RustByteSize::Unknown,
            endianness: None,
            array: None,
            bit_range: BuilderRange::None,
            reserve: ReserveFieldOption::NotReserve,
            overlap: OverlapOptions::None,
        }
    }
    pub fn id(&self) -> &Id {
        &self.id
    }
}

#[derive(Clone, Debug)]
pub enum DataType {
    /// field is a number or primitive. if the endianess is `None`, it will not solve.
    Number(NumberType),
    /// This is a nested structure and does not have a know type. and the name of the struct shall be stored
    /// within.
    #[cfg(feature = "derive")]
    Nested(String),
}

#[derive(Clone, Debug)]
pub enum NumberType {
    /// Floating point numbers
    ///
    /// # Valid
    /// - f32
    /// - f64
    Float,
    /// Unsigned numbers
    ///
    /// # Valid
    /// - u8
    /// - u16
    /// - u32
    /// - u64
    /// - u128
    Unsigned,
    /// Signed numbers
    ///
    /// # Valid
    /// - i8
    /// - i16
    /// - i32
    /// - i64
    /// - i128
    Signed,
}
