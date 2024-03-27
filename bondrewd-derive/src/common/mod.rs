use self::field::Endianness;

pub mod r#enum;
pub mod field;
pub mod object;
pub mod r#struct;

#[derive(Clone)]
pub enum FieldGrabDirection {
    Msb,
    Lsb,
}

#[derive(Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct AttrInfo {
    /// if false then bit 0 is the Most Significant Bit meaning the first values first bit will start there.
    /// if true then bit 0 is the Least Significant Bit (the last bit in the last byte).
    pub lsb_zero: FieldGrabDirection,
    /// flip all the bytes, like .reverse() for vecs or arrays. but we do that here because we can do
    /// it with no runtime cost.
    pub flip: bool,
    /// When this is used with an Enum, Invalid means
    pub invalid: bool,
    pub dump: bool,
    pub enforcement: StructEnforcement,
    pub default_endianess: Endianness,
    pub fill_bits: Option<usize>,
    // Enum only
    pub id: Option<u128>,
}

impl Default for AttrInfo {
    fn default() -> Self {
        Self {
            lsb_zero: FieldGrabDirection::Msb,
            flip: false,
            enforcement: StructEnforcement::NoRules,
            default_endianess: Endianness::None,
            fill_bits: None,
            id: None,
            invalid: false,
            dump: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StructEnforcement {
    /// there is no enforcement so if bits are unused then it will act like they are a reserve field
    NoRules,
    /// enforce the BIT_SIZE equals BYTE_SIZE * 8
    EnforceFullBytes,
    /// enforce an amount of bits total that need to be used.
    EnforceBitAmount(usize),
}
