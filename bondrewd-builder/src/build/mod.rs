pub mod field;
pub mod field_set;

use quote::ToTokens;
use std::{
    fmt::Debug,
    ops::{Deref, Range},
};
use syn::{Expr, Ident, Lit, LitInt, LitStr};

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

#[derive(Clone, Debug)]
pub enum BuilderRange {
    /// A range of bits to use. solve this is easy, but note that it is an exclusive range, meaning the
    /// end is NOT included.
    Range(std::ops::Range<usize>),
    /// Used to pass on the last starting location to the next field that is added to a set. this when solved
    /// will tell bondrewd this can be resolved by being the bits starting from the end of the previous field
    /// to the last bit needed for the field.
    LastEnd(usize),
    /// Will not solve, must be another variant.
    None,
}

impl BuilderRange {
    /// This is intended for use in `bondrewd-derive`.
    ///
    /// Tries to extract a range from a `&Expr`. there is no need to check the type of expr.
    /// If the Result returns `Err` then a parsing error occurred and should be reported as an error to user.
    /// If `Ok(None)`, no error but `expr` was not valid for housing a range.
    pub fn range_from_expr(expr: &Expr, ident: &Ident) -> syn::Result<Option<Self>> {
        if let Some(lit) = get_lit_range(expr, ident)? {
            Ok(Some(Self::Range(lit)))
        } else {
            Ok(None)
        }
    }
}

impl Default for BuilderRange {
    fn default() -> Self {
        Self::None
    }
}

/// The order fields shall be traversed when solving. please read [`Endianness`] for more information.
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
}

/// The order byte indices shall be traversed when solving. please read [`Endianness`] for more information.
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

/// [`Endianness`] is complicated, and allowing users to mess will it is hard. so we need a way
/// to allow the user to show intent to deviate from convention, while not blowing up how the backend works.
/// this is that way read [`Endianness`].
#[derive(Clone, Debug)]
enum UserDefinedReversal {
    Set(bool),
    Unset,
}

impl Default for UserDefinedReversal {
    fn default() -> Self {
        Self::Unset
    }
}

impl UserDefinedReversal {
    /// returns `true` if the user has defined to switch the order.
    pub fn get(&self) -> bool {
        match self {
            UserDefinedReversal::Set(out) => *out,
            UserDefinedReversal::Unset => false,
        }
    }
    /// Overwrites `self` with a user defined reversal and returns `None` if this instance had not
    /// already been set. If `Some` is returned, this has been writing to twice.
    pub fn set(&mut self, reverse: bool) -> Option<bool> {
        let new = Self::Set(reverse);
        let old = std::mem::replace(self, new);
        match old {
            UserDefinedReversal::Set(out) => Some(out),
            UserDefinedReversal::Unset => None,
        }
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
/// To clarify: Bit Endianess exists but standards for them are very obscure and don't have much structure.
///
/// Multiple ways of processing little endian are used in industry. I don't know which
/// is actually correct but offer both because I already needed them both to successfully
/// do my job that i get paid for.
///
/// One thing to consider is because modes where originally created to accomplish big-endian and
/// packed-little-endian functions which is why when Endianness was redefined internally
/// the table in [Bit and Byte order](#bit-and-byte-order) is reversed for `Alternative` (to make
/// non-reversed bit/byte order always mean big endian). `Standard` is a Big Endian processing strategy
/// and `Alternative` is a Packed Little Endian processing strategy and due to weird bit math, "Standard"
/// mode produces "Aligned Little Endian" when both `byte_order` and `field_order` are `true`, look at
/// the truth table in [Bit and Byte order](#bit-and-byte-order).
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
/// These are my made up names for the two common "little endian" bit field strategies.
/// ### Aligned
/// bit are right aligned
/// > Note that we are using `fill_bits` here otherwise only 9 bits will be considered when flipping the order.
/// ```
/// use bondrewd_test as bondrewd;
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
/// ### Packed
/// bits are left aligned
/// > `fill_bits` does nothing to this structure but is included for comparison with Aligned example because
/// > if you don't use `fill_bits` in the Aligned example you will actually get the same output as this
/// > Packed example due to bondrewd handling structures that do not use a multiple of 8 bits.
/// ```
/// use bondrewd_test as bondrewd;
/// use bondrewd::*;
/// #[derive(Bitfields, Clone)]
/// #[bondrewd(default_endianness = "ple", fill_bits)]
/// struct Packed {
///     #[bondrewd(bit_length = 9)]
///     number: u16,
/// }
///
/// assert_eq!(Packed::BIT_SIZE, 9);
/// assert_eq!(Packed::BYTE_SIZE, 2);
/// let ex = Packed { number: u16::MAX };
///
/// let bytes = ex.clone().into_bytes();
/// assert_eq!(bytes, [0b11111111, 0b10000000]);
/// ```
#[derive(Clone, Debug)]
pub struct Endianness {
    /// Describes what type (standard/big-endian or alternative/little-endian) of number resolver to
    /// use for accessing bits after solving.
    mode: EndiannessMode,
    /// # Waring
    /// This needs to be highly controlled and should only be set by functions that understand how the
    /// `mode`, `byte_order`, and `field_order` together determine endianness, and this shall not be otherwise
    /// tampered with via user defined order switching. `reverse_byte_order` is where a chaotic non-standard
    /// byte order reversals shall be defined.
    ///
    /// # What it do
    /// Defines the order that byte indices will be assigned for bit placement during the solving process.
    /// either starting from 0 or the largest byte index.
    byte_order: ByteOrder,
    /// This is a user defined byte order reversal
    reverse_byte_order: UserDefinedReversal,
    /// # Waring
    /// This needs to be highly controlled and should only be set by functions that understand how the
    /// `mode`, `byte_order`, and `field_order` together determine endianness, and this shall not be otherwise
    /// tampered with via user defined order switching. `reverse_field_order` is where a chaotic non-standard
    /// field order reversals shall be defined.
    ///
    /// # What it do
    /// Defines the order that bit indices will be assigned for bit placement during the solving process.
    /// either starting the first field get the first assigned bits or the last fields does.
    field_order: FieldOrder,
    /// This is a user defined field order reversal
    reverse_field_order: UserDefinedReversal,
}

impl Endianness {
    /// Defines if the the order that byte indices shall be assigned in the solving process shall be reversed.
    ///
    /// # Warning
    /// The default of this changes depending on the mode, and the docs for how byte_order
    /// effect things in [`Endianness`] should be read before using this.
    #[inline]
    pub fn set_reverse_byte_order(&mut self, new: bool) -> Option<bool> {
        self.reverse_byte_order.set(new)
    }
    /// This gets called during the solving process to determine the
    /// byte_order, and should be avoided by users. Just build it right without checking, you got this.
    pub fn is_byte_order_reversed(&self) -> bool {
        self.is_alternative() ^ (self.reverse_byte_order.get() ^ self.byte_order.is_reversed())
    }
    /// Reverses the current order that byte indices shall be assigned in the solving process.
    ///
    /// # Warning
    /// The default of this changes depending on the mode, and the docs for how byte_order
    /// effect things in [`Endianness`] should be read before using this.
    pub fn reverse_byte_order(&mut self) -> Option<bool> {
        self.set_reverse_byte_order(!self.reverse_byte_order.get())
    }
    /// Defines if the order that fields will receive bit indices during the solving process shall be reversed.
    ///
    /// # Warning
    /// The default of this changes depending on the mode, and the docs for how field_order
    /// effect things in [`Endianness`] should be read before using this.
    pub fn set_reverse_field_order(&mut self, new: bool) -> Option<bool> {
        self.reverse_field_order.set(new)
    }
    /// This gets called during the solving process to determine the
    /// field_order, and should be avoided by users. Just build it right without checking, you got this.
    pub fn is_field_order_reversed(&self) -> bool {
        let r =
            self.reverse_field_order.get() ^ matches!(self.field_order, FieldOrder::LastToFirst);
        if self.is_alternative() {
            !r
        } else {
            r
        }
    }
    /// Reverses the current order that fields will receive bit indices during the solving process.
    ///
    /// # Warning
    /// The default of this changes depending on the mode, and the docs for how field_order
    /// effect things in [`Endianness`] should be read before using this.
    pub fn reverse_field_order(&mut self, new: bool) -> Option<bool> {
        self.set_reverse_field_order(!self.reverse_field_order.get())
    }
    /// Returns the `mode`.
    pub fn mode(&self) -> EndiannessMode {
        self.mode
    }
    /// Sets the `mode`.
    pub fn set_mode(&mut self, mode: EndiannessMode) {
        self.mode = mode;
    }
    /// Returns `true` if `mode` is [`EndiannessMode::Alternative`].
    pub fn is_alternative(&self) -> bool {
        matches!(self.mode, EndiannessMode::Alternative)
    }
    /// Returns a new `Self` that is big endian.
    pub fn big() -> Self {
        Self {
            mode: EndiannessMode::Standard,
            byte_order: ByteOrder::FirstToLast,
            reverse_byte_order: UserDefinedReversal::default(),
            field_order: FieldOrder::FirstToLast,
            reverse_field_order: UserDefinedReversal::default(),
        }
    }
    /// Returns a new `Self` that is packed little endian.
    ///
    /// # Packed?
    /// Fields will be assigned bits starting from the left or bit 7 (bit index order = 0b76543210).
    ///
    /// If this doesn't clear things up read [`Endianness`] main documentation.
    pub fn little_packed() -> Self {
        Self {
            mode: EndiannessMode::Alternative,
            byte_order: ByteOrder::LastToFirst,
            reverse_byte_order: UserDefinedReversal::default(),
            field_order: FieldOrder::LastToFirst,
            reverse_field_order: UserDefinedReversal::default(),
        }
    }
    /// Returns a new `Self` that is packed little endian.
    ///
    /// # Aligned?
    /// Fields will be assigned bits starting from the right or bit 0 (bit index order = 0b76543210).
    ///
    /// If this doesn't clear things up read [`Endianness`] main documentation.
    pub fn little_aligned() -> Self {
        Self {
            mode: EndiannessMode::Standard,
            byte_order: ByteOrder::LastToFirst,
            reverse_byte_order: UserDefinedReversal::default(),
            field_order: FieldOrder::LastToFirst,
            reverse_field_order: UserDefinedReversal::default(),
        }
    }
}

/// Defines when a field is relevant, which could be never if it is a reserved set of bit for future use.
#[derive(Clone, Debug)]
pub enum ReserveFieldOption {
    /// Do not suppress in `from_bytes` or `into_bytes`.
    NotReserve,
    /// User defined, meaning that the field shall not be written-to or read-from on `into_bytes` or
    /// `from_bytes` calls.
    ReserveField,
    /// Used with imaginary fields that bondrewd creates, such as fill_bytes or variant_ids.
    /// these typically do not get any standard generated functions.
    FakeField,
    /// User defined, meaning that the field shall not be written to on `into_bytes` calls.
    ReadOnly,
}

impl ReserveFieldOption {
    /// Tells `bondrewd-derive` that this field should have write functions generated.
    pub fn wants_write_fns(&self) -> bool {
        match self {
            Self::ReadOnly | Self::FakeField | Self::ReserveField => false,
            Self::NotReserve => true,
        }
    }
    /// Tells `bondrewd-derive` that this field should have read functions generated.
    pub fn wants_read_fns(&self) -> bool {
        match self {
            Self::FakeField | Self::ReserveField => false,
            Self::NotReserve | Self::ReadOnly => true,
        }
    }
    /// Tells `bondrewd-derive` if these bits effect the bit total for the field_set
    pub fn count_bits(&self) -> bool {
        match self {
            Self::FakeField => false,
            Self::ReserveField | Self::NotReserve | Self::ReadOnly => true,
        }
    }
    /// Tells `bondrewd-derive` that this field should be completely ignored.
    pub fn is_fake_field(&self) -> bool {
        match self {
            Self::FakeField => true,
            Self::ReserveField | Self::NotReserve | Self::ReadOnly => false,
        }
    }
}

/// Defines if the field shall be allowed to overlap with others. This is a check or safty measure, mostly
/// used in `bondrewd-derive`.
#[derive(Clone, Debug)]
pub enum OverlapOptions {
    /// Shall not overplay
    None,
    /// Allowed to overlap o specific number of bits
    Allow(usize),
    /// Allowed to be fully overlapped by other fields.
    Redundant,
}

impl OverlapOptions {
    /// Returns `true` if any overlapping be done.
    pub fn enabled(&self) -> bool {
        !matches!(self, Self::None)
    }
    /// Returns `true` if the entire fields bits can be overlapped.
    pub fn is_redundant(&self) -> bool {
        matches!(self, Self::Redundant)
    }
}

pub(crate) fn get_lit_str<'a>(
    expr: &'a Expr,
    ident: &Ident,
    example: Option<&str>,
) -> syn::Result<&'a LitStr> {
    let example = if let Some(ex) = example {
        format!("example: `{ex}`")
    } else {
        String::new()
    };
    if let Expr::Lit(ref lit) = expr {
        if let Lit::Str(ref val) = lit.lit {
            Ok(val)
        } else {
            Err(syn::Error::new(
                ident.span(),
                format!("{ident} requires a string literal. {example}"),
            ))
        }
    } else {
        Err(syn::Error::new(
            ident.span(),
            format!("{ident} requires a string literal. {example}"),
        ))
    }
}

pub(crate) fn get_lit_int<'a>(
    expr: &'a Expr,
    ident: &Ident,
    example: Option<&str>,
) -> syn::Result<&'a LitInt> {
    let example = if let Some(ex) = example {
        format!("example: `{ex}`")
    } else {
        String::new()
    };
    if let Expr::Lit(ref lit) = expr {
        if let Lit::Int(ref val) = lit.lit {
            Ok(val)
        } else {
            Err(syn::Error::new(
                ident.span(),
                format!("{ident} requires a integer literal. {example}"),
            ))
        }
    } else {
        Err(syn::Error::new(
            ident.span(),
            format!("{ident} requires a integer literal. {example}"),
        ))
    }
}

pub(crate) fn get_lit_range(expr: &Expr, ident: &Ident) -> syn::Result<Option<Range<usize>>> {
    if let Expr::Range(ref lit) = expr {
        let start = if let Some(ref v) = lit.start {
            if let Expr::Lit(ref el) = v.as_ref() {
                if let Lit::Int(ref i) = el.lit {
                    i.base10_parse()?
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        "start of range must be an integer.",
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "start of range must be an integer literal.",
                ));
            }
        } else {
            return Err(syn::Error::new(
                ident.span(),
                "range for bits must define a start",
            ));
        };
        let end = if let Some(ref v) = lit.end {
            if let Expr::Lit(ref el) = v.as_ref() {
                if let Lit::Int(ref i) = el.lit {
                    i.base10_parse()?
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        "end of range must be an integer.",
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "end of range must be an integer literal.",
                ));
            }
        } else {
            return Err(syn::Error::new(
                ident.span(),
                "range for bits must define a end",
            ));
        };
        Ok(Some(match lit.limits {
            syn::RangeLimits::HalfOpen(_) => start..end,
            #[allow(clippy::range_plus_one)]
            syn::RangeLimits::Closed(_) => {
                // ALLOW we use a plus one here so we keep the same typing of [`Range`], while not creating more
                // code for something so trivial.
                start..end + 1
            }
        }))
    } else {
        Ok(None)
    }
}
