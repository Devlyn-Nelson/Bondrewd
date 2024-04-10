use std::{fmt::Debug, ops::Deref};

use quote::ToTokens;

pub mod r#enum;
pub mod field;
pub mod object;
pub mod r#struct;

#[derive(Clone)]
pub struct Visibility(pub syn::Visibility);

impl Deref for Visibility {
    type Target = syn::Visibility;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_token_stream().to_string())
    }
}

#[derive(Clone, Default, Debug)]
pub enum FieldOrder {
    #[default]
    FirstToLast,
    LastToFirst,
}
#[derive(Clone, Debug, Copy)]
pub enum EndiannessMode {
    Alternative,
    Standard,
    Nested,
}

#[derive(Clone, Default, Debug)]
pub enum ByteOrder {
    #[default]
    FirstToLast,
    LastToFirst,
}

impl ByteOrder {
    pub fn is_reversed(&self) -> bool {
        matches!(self, Self::LastToFirst)
    }
}

/// This mess determines the endianness of a field.
///
/// # Bit and Byte order
/// `field_order` determines the alignment of bits (left or right). `byte_order`
/// determines the order that bytes will be used. Combining these values in `Standard`
/// `mode` aligned little endian where as having them both as `false` in the same `mode`
/// is big endian. Now for a truth table show fields: A of 3 bit, B of 6 bits, and C of 7.
/// the arrays will be bits represented like so `[07,06,05,04,03,02,01] [15,14,13,12,11,10,09,08]`.
///
/// |`field_order = `    |`false`              | `true`              |
/// |:-------------------|:-------------------:|:-------------------:|
/// |`byte_order = false`|[AAABBBBB] [BCCCCCCC]|[CCCCCCCB] [BBBBBAAA]|
/// |`byte_order = true` |[BCCCCCCC] [AAABBBBB]|[BBBBBAAA] [CCCCCCCB]|
///
/// `field_order` and `bytes_order` both have a reverse flag that are XOR with
/// with `field_order` and `byte_order` being `false` in `FirstToLast`.
///
/// |`field_order = `             | false | true  |
/// |:----------------------------|:-----:|:-----:|
/// |`reverse_field_order = false`| false | true  |
/// |`reverse_field_order = true` | true  | false |
///
/// # Mode vs Endian
/// Endianess is basically really confusing when being applied at the bit level.
///
/// Multiple ways of processing little endian are used in industry. I don't know which
/// is actually correct but offer both because I already needed them both to successfully
/// do my job that i get paid for.
///
/// One thing to consider is because modes where originally created to create big-endian and
/// packed-little-endian functions which is why when Endianness was redefined internally in bondrewd
/// the table in [Bit and Byte order](#bit-and-byte-order) are reversed for `Alternative`. `Standard` is a
/// Big Endian processing strategy and `Alternative` is a Packed Little Endian processing strategy and due
/// to weird bit math, "Standard" mode produces "Aligned Little Endian" when both `byte_order` and
/// `field_order` are `true`, look at the truth table in [Bit and Byte order](#bit-and-byte-order).
/// So because in "Alternate" is little endian we XOR `true` with the results of `byte_order` and
/// `field_order`, see [`Endianness::is_byte_order_reversed`] and upcoming truth table for how logic
/// gets resolved if you are confused.
///
/// |`mode = `            | Standard | Alternative |
/// |:--------------------|:--------:|:-----------:|
/// |`field_order = false`| false    | true        |
/// |`field_order = true` | true     | false       |
///
/// ## But whats what?
/// |`(is_field_order_reversed(),is_byte_order_reversed())`| Standard                | Alternative          |
/// |:-----------------------------------------------------|:-----------------------:|:--------------------:|
/// | (false, false)                                       | Big Endian              | Packed Little Endian |
/// | (true, true)                                         | Aligned Little Endian   | Idk                  |
/// | (true, false)                                        | Idk                     | Idk                  |
/// | (false, true)                                        | Idk                     | Idk                  |
///
/// # Packed... Aligned... ???
/// These are my made up names for them.
/// ### Packed
/// bit are left aligned
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields, Clone)]
/// #[bondrewd(default_endianness = "ple")]
/// struct Packed {
///     #[bondrewd(bit_length = 9)]
///     number: u16,
/// }
///
/// fn main() {
///     assert_eq!(Packed::BIT_SIZE, 9);
///     assert_eq!(Packed::BYTE_SIZE, 2);
///     let ex = Packed { number: u16::MAX };
///
///     let bytes = ex.clone().into_bytes();
///     assert_eq!(bytes, [0b11111111, 0b10000000]);
/// }
/// ```
/// ### Aligned
/// bit are right aligned
/// ```
/// use bondrewd::*;
/// #[derive(Bitfields)]
/// #[bondrewd(default_endianness = "ale", fill_bits)]
/// struct Aligned {
///     #[bondrewd(bit_length = 9)]
///     number: u16,
/// }
/// assert_eq!(Aligned::BIT_SIZE, 9);
/// assert_eq!(Aligned::BYTE_SIZE, 2);
/// let ex = Aligned { number: u16::MAX };
///
/// let bytes = ex.into_bytes();
/// assert_eq!(bytes, [0b11111111, 0b00000001]);
/// ```
#[derive(Clone, Debug)]
pub struct Endianness {
    mode: EndiannessMode,
    byte_order: ByteOrder,
    reverse_byte_order: bool,
    field_order: FieldOrder,
    reverse_field_order: bool,
}

impl Endianness {
    pub fn set_byte_order(&mut self, new: ByteOrder) {
        self.byte_order = new;
    }
    pub fn is_byte_order_reversed(&self) -> bool {
        self.is_alternative() ^ (self.reverse_byte_order ^ self.byte_order.is_reversed())
    }
    pub fn reverse_byte_order(&mut self) {
        self.reverse_byte_order = !self.reverse_byte_order;
    }
    pub fn set_reverse_field_order(&mut self, new: bool) {
        self.reverse_field_order = new;
    }
    pub fn is_field_order_reversed(&self) -> bool {
        let r = self.reverse_field_order ^ matches!(self.field_order, FieldOrder::LastToFirst);
        if self.is_alternative() {
            !r
        } else {
            r
        }
    }
    /// Are the bytes aligned to the bytes start and end, otherwise they are packed.
    pub fn has_endianness(&self) -> bool {
        !matches!(self.mode, EndiannessMode::Nested)
    }
    // If the size provided is 1 or less bytes and endianess is not defined, the endianess will be
    // automatically become big endian which houses common 1 byte logic. if after that the endianess is none
    // `false` will be returned, if big or little endianess `true` will be returned.
    pub fn perhaps_endianness(&mut self, size: usize) -> bool {
        if let EndiannessMode::Nested = self.mode {
            if size == 1 {
                let mut swap = EndiannessMode::Standard;
                std::mem::swap(&mut swap, &mut self.mode);
                true
            } else {
                false
            }
        } else {
            true
        }
    }
    pub fn mode(&self) -> EndiannessMode {
        self.mode
    }
    pub fn set_mode(&mut self, mode: EndiannessMode) {
        self.mode = mode;
    }
    pub fn is_standard(&self) -> bool {
        matches!(self.mode, EndiannessMode::Standard)
    }
    pub fn is_alternative(&self) -> bool {
        matches!(self.mode, EndiannessMode::Alternative)
    }
    // pub fn is_little(&self) -> bool {
    //     matches!(self.inner, Endianness::Little)
    // }
    // pub fn is_none(&self) -> bool {
    //     matches!(self.inner, Endianness::None)
    // }
    pub fn big() -> Self {
        Self {
            mode: EndiannessMode::Standard,
            byte_order: ByteOrder::FirstToLast,
            reverse_byte_order: false,
            field_order: FieldOrder::FirstToLast,
            reverse_field_order: false,
        }
    }
    pub fn little_packed() -> Self {
        Self {
            mode: EndiannessMode::Alternative,
            byte_order: ByteOrder::LastToFirst,
            reverse_byte_order: false,
            field_order: FieldOrder::LastToFirst,
            reverse_field_order: false,
        }
    }
    pub fn little_aligned() -> Self {
        Self {
            mode: EndiannessMode::Standard,
            byte_order: ByteOrder::LastToFirst,
            reverse_byte_order: false,
            field_order: FieldOrder::LastToFirst,
            reverse_field_order: false,
        }
    }
    pub fn nested() -> Self {
        Self {
            mode: EndiannessMode::Nested,
            byte_order: ByteOrder::FirstToLast,
            reverse_byte_order: false,
            field_order: FieldOrder::FirstToLast,
            reverse_field_order: false,
        }
    }
}

impl Default for Endianness {
    fn default() -> Self {
        Self::nested()
    }
}

#[derive(Clone, Debug)]
pub enum FillBits {
    /// Does not fill bytes.
    None,
    /// Fills a specific amount of bits.
    Bits(usize),
    /// Fills bits up until the total is a multiple of 8.
    Auto,
}

impl FillBits {
    pub fn is_none(&self) -> bool {
        matches!(self,Self::None)
    }
}

#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct AttrInfo {
    /// flip all the bytes, like .reverse() for vecs or arrays. but we do that here because we can do
    /// it with no runtime cost.
    pub dump: bool,
    pub enforcement: StructEnforcement,
    pub default_endianess: Endianness,
    pub fill_bits: FillBits,
    // Enum only
    pub id: Option<u128>,
    /// When this is used with an Enum, Invalid means
    pub invalid: bool,
}

impl Default for AttrInfo {
    fn default() -> Self {
        Self {
            enforcement: StructEnforcement::NoRules,
            default_endianess: Endianness::default(),
            fill_bits: FillBits::None,
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
