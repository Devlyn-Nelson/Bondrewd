#[cfg(feature = "derive")]
use syn::token::Pub;

use super::field::DataBuilder;
#[cfg(feature = "derive")]
use super::field::Visibility;

/// Builds a bitfield model. This is not the friendliest user facing entry point for `bondrewd-builder`.
/// please look at either [`FieldSetBuilder`] or [`EnumBuilder`] for a more user friendly builder.
/// This is actually intended to be used by `bondrewd-derive`.
pub struct GenericBuilder<FieldSetId, DataId> {
    /// Define if we are building a single field_set or variant type containing
    /// multiple field_sets switched by an id field.
    pub ty: BuilderType<FieldSetId, DataId>,
    /// The viability of the struct/enum
    #[cfg(feature = "derive")]
    pub vis: Visibility,
    /// Is it a tuple struct/variant
    #[cfg(feature = "derive")]
    pub tuple: bool,
}

impl<FieldSetId, DataId> GenericBuilder<FieldSetId, DataId> {
    pub fn single_set<S: Into<FieldSetId>>(name: S) -> Self {
        Self {
            ty: BuilderType::Struct(FieldSetBuilder::new(name.into())),
            #[cfg(feature = "derive")]
            tuple: false,
            #[cfg(feature = "derive")]
            vis: Visibility(syn::Visibility::Public(Pub::default())),
        }
    }
    pub fn variant_set() -> Self {
        Self {
            ty: BuilderType::Enum(EnumBuilder::new()),
            #[cfg(feature = "derive")]
            tuple: false,
            #[cfg(feature = "derive")]
            vis: Visibility(syn::Visibility::Public(Pub::default())),
        }
    }
    pub fn get(&self) -> &BuilderType<FieldSetId, DataId> {
        &self.ty
    }
    pub fn get_mut(&mut self) -> &mut BuilderType<FieldSetId, DataId> {
        &mut self.ty
    }
}
/// Distinguishes between enums and structs or a single field_set vs multiple
/// field_sets that switch based on an id field.
pub enum BuilderType<FieldSetId, DataId> {
    /// Multiple field_sets that switch based on an id field.
    Enum(EnumBuilder<FieldSetId, DataId>),
    /// A single field_set.
    Struct(FieldSetBuilder<FieldSetId, DataId>),
}

impl<FieldSetId, DataId> BuilderType<FieldSetId, DataId> {
    pub fn get_struct(&self) -> Option<&FieldSetBuilder<FieldSetId, DataId>> {
        if let Self::Struct(ref thing) = self {
            Some(thing)
        } else {
            None
        }
    }
    pub fn get_enum(&self) -> Option<&EnumBuilder<FieldSetId, DataId>> {
        if let Self::Enum(ref thing) = self {
            Some(thing)
        } else {
            None
        }
    }
    pub fn get_mut_struct(&mut self) -> Option<&mut FieldSetBuilder<FieldSetId, DataId>> {
        if let Self::Struct(ref mut thing) = self {
            Some(thing)
        } else {
            None
        }
    }
    pub fn get_mut_enum(&mut self) -> Option<&mut EnumBuilder<FieldSetId, DataId>> {
        if let Self::Enum(ref mut thing) = self {
            Some(thing)
        } else {
            None
        }
    }
}

/// Builds an enum bitfield model.
pub struct EnumBuilder<FieldSetId, DataId> {
    /// Name or ident of the enum, really only matters for `bondrewd-derive`
    #[cfg(feature = "derive")]
    name: Option<Ident>,
    /// The id field with determines the field_set to use.
    id: Option<DataBuilder<DataId>>,
    /// The default variant for situations where no other variant matches.
    invalid: Option<VariantBuilder<FieldSetId, DataId>>,
    /// The collection of variant field_sets.
    variants: Vec<VariantBuilder<FieldSetId, DataId>>,
}

impl<FieldSetId, DataId> EnumBuilder<FieldSetId, DataId> {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "derive")]
            name: None,
            id: None,
            invalid: None,
            variants: Vec::default(),
        }
    }
}
/// Contains builder information for constructing variant style bitfield models.
pub struct VariantBuilder<FieldSetId, DataId> {
    /// The id value that this variant shall be used for.
    id: Option<i64>,
    /// If the variant has a field that whats to capture the
    /// value read for the variant resolution the fields shall be placed here
    /// NOT in the field set, useful for invalid variant.
    capture_field: Option<DataBuilder<DataId>>,
    /// the field_set
    field_set: FieldSetBuilder<FieldSetId, DataId>,
}
/// A builder for a single named set of fields used to construct a bitfield model.
pub struct FieldSetBuilder<FieldSetId, DataId> {
    pub(crate) name: FieldSetId,
    /// the set of fields.
    pub(crate) fields: Vec<DataBuilder<DataId>>,
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

impl<FieldSetId, DataId> FieldSetBuilder<FieldSetId, DataId> {
    pub fn new(key: FieldSetId) -> Self {
        Self {
            name: key,
            fields: Vec::default(),
            enforcement: StructEnforcement::default(),
            fill_bits: FillBits::default(),
        }
    }
    pub fn add_field(&mut self, new_data: DataBuilder<DataId>) {
        self.fields.push(new_data);
    }
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
