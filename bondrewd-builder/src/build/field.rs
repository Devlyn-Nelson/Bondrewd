use crate::solved::field::DynamicIdent;

use super::{BuilderRange, Endianness, OverlapOptions, ReserveFieldOption};

#[derive(Debug)]
pub struct DataBuilder {
    /// The name or ident of the field.
    pub(crate) id: DynamicIdent,
    /// The approximate data type of the field. when solving, this must be
    /// filled.
    pub(crate) ty: DataType,
    /// Describes the properties of which techniques to use for bit extraction
    /// and modifications the inputs that they can have. When None, we are expecting
    /// either a Nested Type or the get it from the default.
    pub(crate) endianness: Option<Endianness>,
    /// The range of bits that this field will use.
    /// TODO this should become a new Range system that allows dynamic start and/or end bit-indices.
    pub(crate) bit_range: BuilderRange,
    /// Describes when the field should be considered.
    pub(crate) reserve: ReserveFieldOption,
    /// How much you care about the field overlapping other fields.
    pub(crate) overlap: OverlapOptions,
}

#[derive(Debug, Clone, Copy)]
pub enum RustByteSize {
    One,
    Two,
    Four,
    Eight,
    Sixteen,
}

impl RustByteSize {
    #[must_use] pub fn bytes(&self) -> usize {
        match self {
            RustByteSize::One => 1,
            RustByteSize::Two => 2,
            RustByteSize::Four => 4,
            RustByteSize::Eight => 8,
            RustByteSize::Sixteen => 16,
        }
    }
    #[must_use] pub fn bits(&self) -> usize {
        match self {
            RustByteSize::One => 8,
            RustByteSize::Two => 16,
            RustByteSize::Four => 32,
            RustByteSize::Eight => 64,
            RustByteSize::Sixteen => 128,
        }
    }
}

impl DataType {
    #[must_use] pub fn rust_size(&self) -> usize {
        match self {
            DataType::Number(number_type, rust_byte_size) => rust_byte_size.bytes(),
            DataType::Nested {
                ident,
                rust_byte_size,
            } => *rust_byte_size,
        }
    }
}

#[derive(Clone, Debug)]
pub enum DataType {
    /// field is a number or primitive. if the endianess is `None`, it will not solve.
    Number(NumberType, RustByteSize),
    /// This is a nested structure and does not have a know type. and the name of the struct shall be stored
    /// within.
    Nested {
        ident: String,
        rust_byte_size: usize,
    },
}

#[derive(Debug, Clone, Copy)]
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
    /// Just `Char`
    Char,
    /// Boolean types
    Bool,
}

impl DataBuilder {
    #[must_use] pub fn new(name: DynamicIdent, ty: DataType) -> Self {
        Self {
            id: name,
            ty,
            endianness: None,
            bit_range: BuilderRange::None,
            reserve: ReserveFieldOption::NotReserve,
            overlap: OverlapOptions::None,
        }
    }
    #[must_use] pub fn id(&self) -> &DynamicIdent {
        &self.id
    }

    pub fn set_endianess(&mut self, e: Endianness) {
        self.endianness = Some(e);
    }
}
