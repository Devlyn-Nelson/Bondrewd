use proc_macro2::Span;
use syn::{spanned::Spanned, DataEnum, DeriveInput, Error, Expr, Fields, Ident, Lit};

use crate::{build::EndiannessFn, solved::field_set::SolvedFieldSetAttributes};

use super::{field::DataBuilder, Endianness};

use darling::{FromDeriveInput, FromMeta, FromVariant};

/// Builds a bitfield model. This is not the friendliest user facing entry point for `bondrewd-builder`.
/// please look at either [`FieldSetBuilder`] or [`EnumBuilder`] for a more user friendly builder.
/// This is actually intended to be used by `bondrewd-derive`.
#[derive(Debug)]
pub struct GenericBuilder {
    /// Define if we are building a single `field_set` or variant type containing
    /// multiple `field_sets` switched by an id field.
    pub ty: BuilderType,
}

impl GenericBuilder {
    pub fn name(&self) -> Ident {
        match &self.ty {
            BuilderType::Enum(enum_builder) => enum_builder.name.clone(),
            BuilderType::Struct(struct_builder) => struct_builder.field_set.name.clone(),
        }
    }
    pub fn from_struct_builder(s: Box<StructBuilder>) -> Self {
        Self {
            ty: BuilderType::Struct(s),
        }
    }
    pub fn from_struct_builder_derive(s: Box<StructBuilder>, tuple: bool) -> Self {
        Self {
            ty: BuilderType::Struct(s),
        }
    }
    #[must_use]
    pub fn get(&self) -> &BuilderType {
        &self.ty
    }
    pub fn get_mut(&mut self) -> &mut BuilderType {
        &mut self.ty
    }
    pub fn parse(input: &DeriveInput) -> syn::Result<Self> {
        match &input.data {
            syn::Data::Struct(data_struct) => {
                let attrs: StructDarlingSimplified =
                    StructDarling::from_derive_input(input)?.try_into()?;
                let endianness = attrs.endianness.unwrap_or_default();
                let mut fields = Vec::default();
                let tuple = Self::extract_fields(&mut fields, &data_struct.fields, &endianness)?;
                let s = StructBuilder {
                    field_set: FieldSetBuilder {
                        name: attrs.ident,
                        fields,
                        fill_bits: attrs.fill_bits,
                        attrs: AttrsBuilder {
                            enforcement: attrs.enforcement,
                            endianness,
                        },
                    },
                    attrs: SolvedFieldSetAttributes {
                        dump: attrs.dump,
                        vis: super::Visibility(attrs.vis),
                    },
                    tuple,
                };
                Ok(Self {
                    ty: BuilderType::Struct(Box::new(s)),
                })
            }
            syn::Data::Enum(data_enum) => {
                let (enum_attrs, attrs) = {
                    let thing: ObjectDarlingSimplifiedPackage =
                        EnumDarling::from_derive_input(input)?.try_into()?;
                    (thing.enum_attrs, thing.struct_attrs)
                };
                Self::parse_enum(data_enum, &attrs, &enum_attrs)
            }
            syn::Data::Union(_) => Err(Error::new(Span::call_site(), "input can not be a union")),
        }
    }
    // Parses the Expression, looking for a literal number expression
    fn parse_lit_discriminant_expr(input: &Expr) -> syn::Result<usize> {
        match input {
            Expr::Lit(ref lit) => match lit.lit {
                Lit::Int(ref i) => Ok(i.base10_parse()?),
                _ => Err(syn::Error::new(
                    input.span(),
                    "Non-integer literals for custom discriminant are illegal.",
                )),
            },
            _ => Err(syn::Error::new(
                input.span(),
                "non-literal expressions for custom discriminant are illegal.",
            )),
        }
    }
    fn parse_enum(
        data_enum: &DataEnum,
        struct_attrs: &StructDarlingSimplified,
        enum_attrs: &EnumDarlingSimplified,
    ) -> syn::Result<Self> {
        let mut variants: Vec<VariantBuilder> = Vec::default();
        // let mut id_hint =
        let mut invalid_variant: Option<VariantBuilder> = None;
        for variant in &data_enum.variants {
            let mut attrs = StructDarlingSimplified::default();
            attrs.endianness = struct_attrs.endianness.clone();
            let lit_id = if let Some((_, ref expr)) = variant.discriminant {
                let parsed = Self::parse_lit_discriminant_expr(expr)?;
                Some(parsed)
            } else {
                None
            };
            let mut variant_attrs = {
                let vd = VariantDarling::from_variant(variant)?;

                VariantDarlingSimplified::do_thing(vd, &mut attrs)?
            };
            let endianness = attrs.endianness.clone().unwrap_or_default();
            if variant_attrs.id.is_none() {
                variant_attrs.id = lit_id;
            } else if lit_id.is_some() {
                return Err(syn::Error::new(variant.span(), "variant was given an id value via 'id' attribute and literal expression, please only use 1 method of defining id."));
            }
            let variant_name = variant.ident.clone();
            // let fields = Self::parse_fields(
            //     &variant_name,
            //     &variant.fields,
            //     &attrs,
            //     Some(id_field.clone()),
            //     tuple,
            // )?;
            // TODO currently we always add the id field, but some people might want the id to be a
            // field in the variant. this would no longer need to insert the id as a "fake-field".
            let mut fields = Vec::default();
            let tuple = Self::extract_fields(&mut fields, &variant.fields, &endianness)?;
            let field_set = FieldSetBuilder {
                name: variant_name,
                fields,
                fill_bits: attrs.fill_bits,
                attrs: AttrsBuilder {
                    enforcement: attrs.enforcement,
                    endianness,
                },
            };
            let id = variant_attrs.id;
            if variant_attrs.invalid {
                if invalid_variant.is_some() {
                    return Err(Error::new(
                        field_set.name.span(),
                        "second \"invalid\" variant found. This acts as the Enum's default for invalid cases and bondrewd currently only allows 1.",
                    ));
                }
                invalid_variant = Some(VariantBuilder {
                    id,
                    field_set,
                    tuple,
                });
            } else {
                variants.push(VariantBuilder {
                    id,
                    field_set,
                    tuple,
                });
            }
        }
        // detect and fix variants without ids and verify non conflict.
        let invalid_variant = if let Some(iv) = invalid_variant {
            iv
        } else if let Some(last) = variants.pop() {
            last
        } else {
            return Err(Error::new(
                Span::call_site(),
                "Enums must contain at least one variant... Please...",
            ));
        };

        let out = Self {
            ty: BuilderType::Enum(Box::new(EnumBuilder {
                name: struct_attrs.ident.clone(),
                id: None,
                invalid: invalid_variant,
                variants,
                solved_attrs: SolvedFieldSetAttributes {
                    dump: struct_attrs.dump,
                    vis: super::Visibility(struct_attrs.vis.clone()),
                },
                id_bit_length: enum_attrs.id_bit_length,
                payload_bit_length: enum_attrs.payload_bit_length,
                attrs: AttrsBuilder {
                    enforcement: struct_attrs.enforcement.clone(),
                    endianness: struct_attrs.endianness.clone().unwrap_or_default(),
                },
            })),
        };
        Ok(out)
    }
    /// `bondrewd_fields` should either be empty or have exactly 1 field. If a field is provided it is assumed that
    /// this "struct" is an enum variant and the field is the enum value.
    ///
    /// `id_field` in the case of enum variants the id field is defined by the enums attributes or calculated.
    /// The id field bondrewd creates must be passed in here. otherwise this will be treated as a Struct.
    ///
    /// `syn_fields` is just the raw fields from the `syn` crate.
    ///
    /// `endianness` is the endianness any fields that do not have specified endianness will be given.
    ///
    /// Returns `true` if the fields are unnamed, meaning this is a tuple Struct or Variant.
    /// `false` indicating that the fields are named.
    fn extract_fields(
        bondrewd_fields: &mut Vec<DataBuilder>,
        syn_fields: &Fields,
        endianness: &Endianness,
    ) -> syn::Result<bool> {
        let (stripped_fields, tuple) = match syn_fields {
            syn::Fields::Named(ref named_fields) => (
                Some(
                    named_fields
                        .named
                        .iter()
                        .cloned()
                        .collect::<Vec<syn::Field>>(),
                ),
                false,
            ),
            syn::Fields::Unnamed(ref fields) => (
                Some(fields.unnamed.iter().cloned().collect::<Vec<syn::Field>>()),
                true,
            ),
            syn::Fields::Unit => (None, false),
        };
        // figure out what the field are and what/where they should be in byte form.
        if let Some(fields) = stripped_fields {
            for (i, field) in fields.iter().enumerate() {
                let parsed_field = DataBuilder::parse(field, bondrewd_fields, endianness)?;
                bondrewd_fields.push(parsed_field);
            }
        }
        Ok(tuple)
    }
}

#[derive(Debug)]
pub enum BitTraversal {
    Back,
    Front,
}

impl FromMeta for BitTraversal {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value.to_lowercase().as_str() {
            "back" => Ok(Self::Back),
            "front" => Ok(Self::Front),
            _ => Err(darling::Error::unknown_value(
                "unknown bit_traversal value use \"front\", or \"back\"",
            )),
        }
    }
}

#[derive(Debug, FromDeriveInput, FromVariant)]
#[darling(attributes(bondrewd))]
pub struct StructDarling {
    pub endianness: Option<Endianness>,
    pub endianness_fn: Option<EndiannessFn>,
    pub bit_traversal: Option<BitTraversal>,
    pub reverse: darling::util::Flag,
    pub ident: Ident,
    pub vis: syn::Visibility,
    pub dump: darling::util::Flag,
    pub enforce_full_bytes: darling::util::Flag,
    pub enforce_bytes: Option<darling::util::SpannedValue<usize>>,
    pub enforce_bits: Option<darling::util::SpannedValue<usize>>,
    pub fill_bits: Option<FillDarling>,
    pub fill_bytes: Option<FillDarling>,
    pub fill: darling::util::Flag,
}
#[derive(Debug, Clone)]
pub struct StructDarlingSimplified {
    pub endianness: Option<Endianness>,
    pub ident: Ident,
    pub vis: syn::Visibility,
    pub dump: bool,
    pub enforcement: StructEnforcement,
    pub fill_bits: FillBits,
}

impl Default for StructDarlingSimplified {
    fn default() -> Self {
        Self {
            endianness: Default::default(),
            dump: Default::default(),
            enforcement: Default::default(),
            fill_bits: Default::default(),
            ident: Ident::new("error", Span::call_site()),
            vis: syn::Visibility::Public(syn::token::Pub(Span::call_site())),
        }
    }
}

impl StructDarlingSimplified {
    pub fn try_solve_enforcement(
        enforce_full_bytes: darling::util::Flag,
        enforce_bytes: Option<darling::util::SpannedValue<usize>>,
        enforce_bits: Option<darling::util::SpannedValue<usize>>,
    ) -> syn::Result<StructEnforcement> {
        if enforce_full_bytes.is_present() {
            if enforce_bytes.is_none() && enforce_bits.is_none() {
                Ok(StructEnforcement {
                    ty: StructEnforcementTy::EnforceFullBytes,
                    span: enforce_full_bytes.span(),
                })
            } else {
                Err(Error::new(Span::call_site(), "Please only use 1 byte enforcement attribute (enforce_full_bytes, enforce_bytes, enforce_bits)"))
            }
        } else if let Some(bytes) = enforce_bytes {
            if enforce_bits.is_none() {
                Ok(StructEnforcement {
                    ty: StructEnforcementTy::EnforceBitAmount(*bytes * 8),
                    span: bytes.span(),
                })
            } else {
                Err(Error::new(Span::call_site(), "Please only use 1 byte enforcement attribute (enforce_full_bytes, enforce_bytes, enforce_bits)"))
            }
        } else if let Some(bits) = enforce_bits {
            Ok(StructEnforcement {
                ty: StructEnforcementTy::EnforceBitAmount(*bits),
                span: bits.span(),
            })
        } else {
            Ok(StructEnforcement {
                ty: StructEnforcementTy::NoRules,
                span: Span::call_site(),
            })
        }
    }
    pub fn try_solve_fill_bits(
        fill_bits: Option<FillDarling>,
        fill_bytes: Option<FillDarling>,
        fill: darling::util::Flag,
    ) -> syn::Result<FillBits> {
        if fill.is_present() {
            if fill_bytes.is_none() && fill_bits.is_none() {
                Ok(FillBits::Auto)
            } else {
                Err(Error::new(
                    Span::call_site(),
                    "Please only use 1 byte filling attribute (fill, fill_bits, fill_bytes)",
                ))
            }
        } else if let Some(bytes) = fill_bytes {
            if fill_bits.is_none() {
                let out = match bytes {
                    FillDarling::Auto => FillBits::Auto,
                    FillDarling::Size(bytes) => FillBits::Bits(bytes * 8),
                };
                Ok(out)
            } else {
                Err(Error::new(
                    Span::call_site(),
                    "Please only use 1 byte filling attribute (fill, fill_bits, fill_bytes)",
                ))
            }
        } else if let Some(bits) = fill_bits {
            let out = match bits {
                FillDarling::Auto => FillBits::Auto,
                FillDarling::Size(bits) => FillBits::Bits(bits),
            };
            Ok(out)
        } else {
            Ok(FillBits::None)
        }
    }
    pub fn merge(&mut self, other: StructDarlingSimplified) -> syn::Result<()> {
        if let Some(val) = other.endianness {
            self.endianness = Some(val);
        }
        if !matches!(other.enforcement.ty, StructEnforcementTy::NoRules) {
            self.enforcement = other.enforcement;
        }
        if !matches!(other.fill_bits, FillBits::None) {
            self.fill_bits = other.fill_bits;
        }
        self.dump = other.dump;
        self.ident = other.ident;
        self.vis = other.vis;
        Ok(())
    }
}

impl TryFrom<StructDarling> for StructDarlingSimplified {
    type Error = syn::Error;

    fn try_from(darling: StructDarling) -> Result<Self, Self::Error> {
        let endianness = if let Some(mut val) = darling.endianness {
            if let Some(bt) = darling.bit_traversal {
                val.set_reverse_field_order(matches!(bt, BitTraversal::Back));
            }
            if darling.reverse.is_present() {
                val.set_reverse_byte_order(true);
            }
            if let Some(ef) = darling.endianness_fn {
                val.set_endianness_fn(ef);
            }
            Some(val)
        } else if darling.bit_traversal.is_some() || darling.reverse.is_present() || darling.endianness_fn.is_some() {
            let mut val = Endianness::default();
            if let Some(bt) = darling.bit_traversal {
                val.set_reverse_field_order(matches!(bt, BitTraversal::Back));
            }
            if darling.reverse.is_present() {
                val.set_reverse_byte_order(true);
            }
            if let Some(ef) = darling.endianness_fn {
                val.set_endianness_fn(ef);
            }
            Some(val)
        } else {
            None
        };
        // determine byte enforcement if any.
        let enforcement = Self::try_solve_enforcement(
            darling.enforce_full_bytes,
            darling.enforce_bytes,
            darling.enforce_bits,
        )?;
        // determine byte filling if any.
        let fill_bits =
            Self::try_solve_fill_bits(darling.fill_bits, darling.fill_bytes, darling.fill)?;
        Ok(Self {
            endianness,
            ident: darling.ident,
            vis: darling.vis,
            dump: darling.dump.is_present(),
            enforcement,
            fill_bits,
        })
    }
}
#[derive(Debug, FromVariant)]
#[darling(attributes(bondrewd))]
pub struct VariantDarling {
    pub id: Option<usize>,
    pub invalid: darling::util::Flag,
    // struct
    pub endianness: Option<Endianness>,
    pub endianness_fn: Option<EndiannessFn>,
    pub bit_traversal: Option<BitTraversal>,
    pub reverse: darling::util::Flag,
    pub ident: Ident,
    pub dump: darling::util::Flag,
    pub enforce_full_bytes: darling::util::Flag,
    pub enforce_bytes: Option<darling::util::SpannedValue<usize>>,
    pub enforce_bits: Option<darling::util::SpannedValue<usize>>,
    pub fill_bits: Option<FillDarling>,
    pub fill_bytes: Option<FillDarling>,
    pub fill: darling::util::Flag,
}

#[derive(Debug)]
pub enum FillDarling {
    Auto,
    Size(usize),
}

impl FromMeta for FillDarling {
    fn from_word() -> darling::Result<Self> {
        Ok(Self::Auto)
    }
    fn from_value(value: &Lit) -> darling::Result<Self> {
        match value {
            Lit::Int(lit_int) => Ok(Self::Size(lit_int.base10_parse()?)),
            _ => Err(darling::Error::unsupported_format(
                "fill values must be integers",
            )),
        }
    }
}

pub struct VariantDarlingSimplified {
    pub id: Option<usize>,
    pub invalid: bool,
}
impl VariantDarlingSimplified {
    fn do_thing(
        value: VariantDarling,
        attrs: &mut StructDarlingSimplified,
    ) -> Result<Self, syn::Error> {
        if let Some(val) = value.endianness {
            attrs.endianness = Some(val);
        }
        if let Some(val) = &mut attrs.endianness {
            if let Some(bt) = value.bit_traversal {
                val.set_reverse_field_order(matches!(bt, BitTraversal::Back));
            }
            if value.reverse.is_present() {
                val.set_reverse_field_order(true);
            }
            if let Some(ef) = value.endianness_fn {
                val.set_endianness_fn(ef);
            }
        }
        // determine byte enforcement if any.
        let enforcement = StructDarlingSimplified::try_solve_enforcement(
            value.enforce_full_bytes,
            value.enforce_bytes,
            value.enforce_bits,
        )?;
        // determine byte filling if any.
        let fill_bits = StructDarlingSimplified::try_solve_fill_bits(
            value.fill_bits,
            value.fill_bytes,
            value.fill,
        )?;
        if !matches!(enforcement.ty, StructEnforcementTy::NoRules) {
            attrs.enforcement = enforcement;
        }
        if !matches!(fill_bits, FillBits::None) {
            attrs.fill_bits = fill_bits;
        }
        attrs.dump = value.dump.is_present();
        attrs.ident = value.ident;
        Ok(VariantDarlingSimplified {
            id: value.id,
            invalid: value.invalid.is_present(),
        })
    }
}
#[derive(Debug, FromDeriveInput, FromVariant)]
#[darling(attributes(bondrewd))]
pub struct EnumDarling {
    pub ident: Ident,
    pub vis: syn::Visibility,
    // TODO implement id_tail and id_head.
    // pub id_tail: darling::util::Flag,
    // pub id_head: darling::util::Flag,
    pub payload_bit_length: Option<usize>,
    pub payload_byte_length: Option<usize>,
    pub id_bit_length: Option<usize>,
    pub id_byte_length: Option<usize>,
    // Struct
    pub endianness: Option<Endianness>,
    pub endianness_fn: Option<EndiannessFn>,
    pub bit_traversal: Option<BitTraversal>,
    pub reverse: darling::util::Flag,
    pub dump: darling::util::Flag,
    pub enforce_full_bytes: darling::util::Flag,
    pub enforce_bytes: Option<darling::util::SpannedValue<usize>>,
    pub enforce_bits: Option<darling::util::SpannedValue<usize>>,
    pub fill_bits: Option<FillDarling>,
    pub fill_bytes: Option<FillDarling>,
    pub fill: darling::util::Flag,
}

pub struct ObjectDarlingSimplifiedPackage {
    pub enum_attrs: EnumDarlingSimplified,
    pub struct_attrs: StructDarlingSimplified,
}
#[derive(Debug)]
pub struct EnumDarlingSimplified {
    pub payload_bit_length: Option<usize>,
    pub id_bit_length: Option<usize>,
}

impl TryFrom<EnumDarling> for ObjectDarlingSimplifiedPackage {
    type Error = syn::Error;

    fn try_from(value: EnumDarling) -> Result<Self, Self::Error> {
        let struct_attrs = StructDarling {
            endianness: value.endianness,
            endianness_fn: value.endianness_fn,
            bit_traversal: value.bit_traversal,
            reverse: value.reverse,
            ident: value.ident.clone(),
            vis: value.vis.clone(),
            dump: value.dump,
            enforce_full_bytes: value.enforce_full_bytes,
            enforce_bytes: value.enforce_bytes,
            enforce_bits: value.enforce_bits,
            fill_bits: value.fill_bits,
            fill_bytes: value.fill_bytes,
            fill: value.fill,
        }
        .try_into()?;
        let enum_attrs = EnumDarlingSimplified {
            payload_bit_length: if value.payload_bit_length.is_some()
                && value.payload_byte_length.is_some()
            {
                return Err(syn::Error::new(
                    value.ident.span(),
                    "you must use either `payload_bit_length` OR `payload_byte_length`, not both.",
                ));
            } else {
                value
                    .payload_bit_length
                    .or(value.payload_byte_length.map(|x| x * 8))
            },
            id_bit_length: if value.id_bit_length.is_some() && value.id_byte_length.is_some() {
                return Err(syn::Error::new(
                    value.ident.span(),
                    "you must use either `id_bit_length` OR `id_byte_length`, not both.",
                ));
            } else {
                value.id_bit_length.or(value.id_byte_length.map(|x| x * 8))
            },
        };
        Ok(ObjectDarlingSimplifiedPackage {
            enum_attrs,
            struct_attrs,
        })
    }
}

/// Distinguishes between enums and structs or a single `field_set` vs multiple
/// `field_sets` that switch based on an id field.
#[derive(Debug)]
pub enum BuilderType {
    /// Multiple `field_sets` that switch based on an id field.
    Enum(Box<EnumBuilder>),
    /// A single `field_set`.
    Struct(Box<StructBuilder>),
}
/// Builds an enum bitfield model.
#[derive(Debug)]
pub struct StructBuilder {
    pub(crate) field_set: FieldSetBuilder,
    pub(crate) attrs: SolvedFieldSetAttributes,
    /// Is it a tuple struct/variant
    pub tuple: bool,
}
impl From<FieldSetBuilder> for StructBuilder {
    fn from(value: FieldSetBuilder) -> Self {
        Self {
            field_set: value,
            attrs: SolvedFieldSetAttributes::default(),
            tuple: false,
        }
    }
}
impl From<(FieldSetBuilder, SolvedFieldSetAttributes)> for StructBuilder {
    fn from((field_set, attrs): (FieldSetBuilder, SolvedFieldSetAttributes)) -> Self {
        Self {
            field_set,
            attrs,
            tuple: false,
        }
    }
}
#[derive(Debug, Default)]
pub struct AttrsBuilder {
    /// Imposes checks on the sizing of the `field_set`
    pub enforcement: StructEnforcement,
    pub endianness: Endianness,
}
/// Builds an enum bitfield model.
#[derive(Debug)]
pub struct EnumBuilder {
    /// Name or ident of the enum, really only matters for `bondrewd-derive`
    pub(crate) name: Ident,
    /// The id field with determines the `field_set` to use.
    pub(crate) id: Option<DataBuilder>,
    /// The default variant for situations where no other variant matches.
    pub(crate) invalid: VariantBuilder,
    /// The collection of variant `field_sets`.
    pub(crate) variants: Vec<VariantBuilder>,
    pub(crate) solved_attrs: SolvedFieldSetAttributes,
    pub(crate) payload_bit_length: Option<usize>,
    pub(crate) id_bit_length: Option<usize>,
    pub(crate) attrs: AttrsBuilder,
}

impl EnumBuilder {
    pub const VARIANT_ID_NAME: &'static str = "variant_id";
    pub const VARIANT_ID_NAME_KEBAB: &'static str = "variant-id";
    #[must_use]
    pub fn new(name: Ident, invalid: VariantBuilder) -> Self {
        Self {
            name,
            id: None,
            invalid,
            variants: Vec::default(),
            solved_attrs: SolvedFieldSetAttributes::default(),
            id_bit_length: None,
            payload_bit_length: None,
            attrs: AttrsBuilder::default(),
        }
    }
}
/// Contains builder information for constructing variant style bitfield models.
#[derive(Debug)]
pub struct VariantBuilder {
    /// The id value that this variant shall be used for.
    pub(crate) id: Option<usize>,
    /// the `field_set`
    pub(crate) field_set: FieldSetBuilder,
    pub(crate) tuple: bool,
}
/// A builder for a single named set of fields used to construct a bitfield model.
#[derive(Debug)]
pub struct FieldSetBuilder {
    pub(crate) name: Ident,
    /// the set of fields.
    pub(crate) fields: Vec<DataBuilder>,
    /// PLEASE READ IF YOU ARE NOT USING [`StructEnforcement::EnforceFullBytes`]
    ///
    /// If you define a `field_sets` with a total bit count that does not divide evenly by 8, funny behavior can
    /// occur; Should be consistent but i don't what to try and predict how it would behave in all 6 of the
    /// resolvers. Anyway if you think you may run into this, i recommend using fill bits to define that you
    /// want the remaining bits to be reserved as a invisible field using [`FillBits::Auto`]. Otherwise i
    /// will not even try to predict how your bit-location-determination will be solved.
    ///
    /// Tells system to add bits to the end as a reserve field.
    /// Using Auto is useful because if the `field_set` doesn't
    /// take a multiple of 8 bits, it will fill bits until it does.
    pub fill_bits: FillBits,
    pub attrs: AttrsBuilder,
}

impl FieldSetBuilder {
    #[must_use]
    pub fn new(key: Ident) -> Self {
        Self {
            name: key,
            fields: Vec::default(),
            fill_bits: FillBits::default(),
            attrs: AttrsBuilder::default(),
        }
    }
    pub fn add_field(&mut self, new_data: DataBuilder) {
        self.fields.push(new_data);
    }
    pub fn with_field(mut self, new_data: DataBuilder) -> Self {
        self.fields.push(new_data);
        self
    }
    pub fn bit_length(&self) -> usize {
        let mut bl = 0;
        for f in &self.fields {
            if f.reserve.count_bits() {
                bl += match f.overlap {
                    super::OverlapOptions::None => f.bit_length(),
                    super::OverlapOptions::Allow(bits) => f.bit_length() - bits,
                    super::OverlapOptions::Redundant => 0,
                };
            }
        }
        bl
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
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// Tells bondrewd to enforce specific rules about the amount of bits used by the entire `field_set`
#[derive(Debug, Clone)]
pub struct StructEnforcement {
    pub ty: StructEnforcementTy,
    pub span: Span,
}
impl Default for StructEnforcement {
    fn default() -> Self {
        Self {
            ty: Default::default(),
            span: Span::call_site(),
        }
    }
}
/// Tells bondrewd to enforce specific rules about the amount of bits used by the entire `field_set`
#[derive(Debug, Clone, Default)]
pub enum StructEnforcementTy {
    /// No enforcement on the amount of bits used by the entire `field_set`
    #[default]
    NoRules,
    /// Enforce the `BIT_SIZE` equals `BYTE_SIZE` * 8
    EnforceFullBytes,
    /// Enforce the amount of bits that need to be used tot a specific value.
    EnforceBitAmount(usize),
}
