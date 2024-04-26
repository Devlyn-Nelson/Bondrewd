use super::{field::DataBuilder, Visibility};

/// Builds a bitfield model.
pub struct Builder {
    /// Define if we are building a single field_set or variant type containing
    /// multiple field_sets switched by an id field.
    pub ty: BuilderType,
    /// The viability of the struct/enum
    pub vis: Visibility,
    /// Is it a tuple struct/variant
    pub tuple: bool,
}
/// Distinguishes between enums and structs or a single field_set vs multiple
/// field_sets that switch based on an id field.
pub enum BuilderType {
    /// Multiple field_sets that switch based on an id field.
    Enum {
        /// Name or ident of the enum, really only matters for `bondrewd-derive`
        name: String,
        /// The id field with determines the field_set to use.
        id: DataBuilder,
        /// The default variant for situations where no other variant matches.
        invalid: Option<VariantBuilder>,
        /// The collection of variant field_sets.
        variants: Vec<VariantBuilder>,
    },
    /// A single field_set.
    Struct(FieldSetBuilder),
}
/// Contains builder information for constructing variant style bitfield models.
pub struct VariantBuilder {
    /// The id value that this variant shall be used for.
    id: Option<i64>,
    /// If the variant has a field that whats to capture the
    /// value read for the variant resolution the fields shall be placed here
    /// NOT in the field set, useful for invalid variant.
    capture_field: Option<DataBuilder>,
    /// the field_set
    field_set: FieldSetBuilder,
}
/// A builder for a single named set of fields used to construct a bitfield model.
pub struct FieldSetBuilder {
    /// Name or ident of the field_set
    name: String,
    /// the set of fields.
    fields: Vec<DataBuilder>,
    /// Imposes checks on the sizing of the field_set
    pub enforcement: StructEnforcement,
    /// PLEASE READ IF YOU ARE NOT USING [`StructEnforcement::EnforceFullBytes`]
    ///
    /// If you define a field_sets with a total bit count that does not divide evenly by 8, funny behavior can
    /// occur; Should be consistent but i don't what to try and predict how it would behave in all 6 of the
    /// resolvers. Anyway if you think you may run into this, i recommend using fill bits to define that you
    /// want the remaining bits to be reserved as a invisible field using [`FillBits::Auto`]. Otherwise i
    /// will not even try to predict how your bit-location-determination will be solved.
    ///
    /// Tells system to add bits to the end as a reserve field.
    /// Using Auto is useful because if the field_set doesn't
    /// take a multiple of 8 bits, it will fill bits until it does.
    pub fill_bits: FillBits,
}

/// A [`Builder`] option that can dynamically add reserve bits to the end of a [`FieldSetBuilder`].
#[derive(Clone, Debug, Default)]
pub enum FillBits {
    // TODO I might want this to default to Auto in the future.
    /// Does not fill bits.
    #[default]
    None,
    /// Fills a specific amount of bits.
    Bits(usize),
    /// Fills bits up until the total amount of bits used is a multiple of 8.
    Auto,
}

impl FillBits {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// Tells bondrewd to enforce specific rules about the amount of bits used by the entire field_set
#[derive(Debug, Clone, Default)]
pub enum StructEnforcement {
    /// No enforcement on the amount of bits used by the entire field_set
    #[default]
    NoRules,
    /// Enforce the BIT_SIZE equals BYTE_SIZE * 8
    EnforceFullBytes,
    /// Enforce the amount of bits that need to be used tot a specific value.
    EnforceBitAmount(u128),
}
