use proc_macro2::Span;
use syn::{spanned::Spanned, DataEnum, DeriveInput, Error, Expr, Fields, Ident, Lit, LitStr};

use crate::solved::field_set::SolvedFieldSetAttributes;

use super::{field::DataBuilder, Endianness};

use darling::{FromDeriveInput, FromVariant};

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
        let attrs: StructDarlingSimplified = StructDarling::from_derive_input(input)?.try_into()?;
        match &input.data {
            syn::Data::Struct(data_struct) => {
                let mut fields = Vec::default();
                let tuple = Self::extract_fields(
                    &mut fields,
                    &data_struct.fields,
                    &attrs.default_endianness,
                )?;
                let s = StructBuilder {
                    field_set: FieldSetBuilder {
                        name: attrs.ident,
                        fields,
                        enforcement: attrs.enforcement,
                        fill_bits: attrs.fill_bits,
                        default_endianness: attrs.default_endianness,
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
                let enum_attrs = EnumDarling::from_derive_input(input)?.try_into()?;
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
            let mut attrs = struct_attrs.clone();
            let lit_id = if let Some((_, ref expr)) = variant.discriminant {
                let parsed = Self::parse_lit_discriminant_expr(expr)?;
                Some(parsed)
            } else {
                None
            };
            let () = attrs.merge(StructDarling::from_variant(variant)?)?;
            let mut variant_attrs = VariantDarling::from_variant(variant)?;
            if variant_attrs.id.is_none() {
                variant_attrs.id = lit_id;
            } else if lit_id.is_some() {
                return Err(syn::Error::new(variant.span(), "variant was given an id value via 'id' attribute and literal expression, please only use 1 method of defining id."));
            };
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
            let tuple =
                Self::extract_fields(&mut fields, &variant.fields, &attrs.default_endianness)?;
            let field_set = FieldSetBuilder {
                name: variant_name,
                fields,
                enforcement: attrs.enforcement,
                fill_bits: attrs.fill_bits,
                default_endianness: attrs.default_endianness,
            };
            let id = variant_attrs.id.into();
            if enum_attrs.invalid {
                if invalid_variant.is_some() {
                    return Err(Error::new(
                        field_set.name.span(),
                        "second \"invalid\" variant found. This acts as the Enum's default for invalid cases and bondrewd currently only allows 1.",
                    ));
                }
                invalid_variant = Some(VariantBuilder { id, field_set });
            } else {
                variants.push(VariantBuilder { id, field_set });
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
                attrs: SolvedFieldSetAttributes {
                    dump: struct_attrs.dump,
                    vis: super::Visibility(struct_attrs.vis.clone()),
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
    /// `default_endianness` is the endianness any fields that do not have specified endianness will be given.
    ///
    /// Returns `true` if the fields are unnamed, meaning this is a tuple Struct or Variant.
    /// `false` indicating that the fields are named.
    fn extract_fields(
        bondrewd_fields: &mut Vec<DataBuilder>,
        syn_fields: &Fields,
        default_endianness: &Endianness,
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
            for (i, ref field) in fields.iter().enumerate() {
                let parsed_field = DataBuilder::parse(field, &bondrewd_fields, default_endianness)?;
                bondrewd_fields.push(parsed_field);
            }
        }
        Ok(tuple)
    }
}
#[derive(Debug, FromDeriveInput, FromVariant)]
#[darling(attributes(bondrewd))]
pub struct StructDarling {
    pub default_endianness: Option<LitStr>,
    pub reverse: darling::util::Flag,
    pub ident: Ident,
    pub vis: syn::Visibility,
    pub dump: darling::util::Flag,
    pub enforce_full_bytes: darling::util::Flag,
    pub enforce_bytes: Option<usize>,
    pub enforce_bits: Option<usize>,
    pub fill_bits: Option<usize>,
    pub fill_bytes: Option<usize>,
    pub fill: darling::util::Flag,
}
#[derive(Debug, Clone)]
pub struct StructDarlingSimplified {
    pub default_endianness: Endianness,
    pub reverse: bool,
    pub ident: Ident,
    pub vis: syn::Visibility,
    pub dump: bool,
    pub enforcement: StructEnforcement,
    pub fill_bits: FillBits,
}

impl StructDarlingSimplified {
    pub fn try_solve_endianness(lit_str: &LitStr) -> syn::Result<Endianness> {
        Endianness::from_expr(lit_str)
    }
    pub fn try_solve_enforcement(
        enforce_full_bytes: darling::util::Flag,
        enforce_bytes: Option<usize>,
        enforce_bits: Option<usize>,
    ) -> syn::Result<StructEnforcement> {
        if enforce_full_bytes.is_present() {
            if enforce_bytes.is_none() && enforce_bits.is_none() {
                Ok(StructEnforcement::EnforceFullBytes)
            } else {
                Err(Error::new(Span::call_site(), "Please only use 1 byte enforcement attribute (enforce_full_bytes, enforce_bytes, enforce_bits)"))
            }
        } else if let Some(bytes) = enforce_bytes {
            if enforce_bits.is_none() {
                Ok(StructEnforcement::EnforceBitAmount(bytes * 8))
            } else {
                Err(Error::new(Span::call_site(), "Please only use 1 byte enforcement attribute (enforce_full_bytes, enforce_bytes, enforce_bits)"))
            }
        } else if let Some(bits) = enforce_bits {
            Ok(StructEnforcement::EnforceBitAmount(bits))
        } else {
            Ok(StructEnforcement::NoRules)
        }
    }
    pub fn try_solve_fill_bits(
        fill_bits: Option<usize>,
        fill_bytes: Option<usize>,
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
                Ok(FillBits::Bits(bytes * 8))
            } else {
                Err(Error::new(
                    Span::call_site(),
                    "Please only use 1 byte filling attribute (fill, fill_bits, fill_bytes)",
                ))
            }
        } else if let Some(bits) = fill_bits {
            Ok(FillBits::Bits(bits))
        } else {
            Ok(FillBits::None)
        }
    }
    pub fn merge(&mut self, other: StructDarling) -> syn::Result<()> {
        if let Some(ref val) = other.default_endianness {
            self.default_endianness = Self::try_solve_endianness(val)?
        };
        let enforcement = Self::try_solve_enforcement(
            other.enforce_full_bytes,
            other.enforce_bytes,
            other.enforce_bits,
        )?;
        let fill_bits = Self::try_solve_fill_bits(other.fill_bits, other.fill_bytes, other.fill)?;
        if !matches!(enforcement, StructEnforcement::NoRules) {
            self.enforcement = enforcement;
        }
        if !matches!(fill_bits, FillBits::None) {
            self.fill_bits = fill_bits;
        }
        if other.dump.is_present() {
            self.dump = true;
        }
        if other.reverse.is_present() {
            self.reverse = true;
        }
        self.ident = other.ident;
        Ok(())
    }
}

impl TryFrom<StructDarling> for StructDarlingSimplified {
    type Error = syn::Error;

    fn try_from(darling: StructDarling) -> Result<Self, Self::Error> {
        let default_endianness = if let Some(ref val) = darling.default_endianness {
            Self::try_solve_endianness(val)?
        } else {
            Endianness::default()
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
            default_endianness,
            reverse: darling.reverse.is_present(),
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
    pub invalid: darling::util::Flag,
}
#[derive(Debug)]
pub struct EnumDarlingSimplified {
    pub ident: Ident,
    pub vis: syn::Visibility,
    pub payload_bit_length: Option<usize>,
    pub id_bit_length: Option<usize>,
    pub invalid: bool,
}

impl TryFrom<EnumDarling> for EnumDarlingSimplified {
    type Error = syn::Error;

    fn try_from(value: EnumDarling) -> Result<Self, Self::Error> {
        Ok(EnumDarlingSimplified {
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
            ident: value.ident,
            vis: value.vis,
            invalid: value.invalid.is_present(),
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
    pub(crate) attrs: SolvedFieldSetAttributes,
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
            attrs: SolvedFieldSetAttributes::default(),
        }
    }
}
/// Contains builder information for constructing variant style bitfield models.
#[derive(Debug)]
pub struct VariantBuilder {
    /// The id value that this variant shall be used for.
    id: Option<usize>,
    /// the `field_set`
    field_set: FieldSetBuilder,
}
/// A builder for a single named set of fields used to construct a bitfield model.
#[derive(Debug)]
pub struct FieldSetBuilder {
    pub(crate) name: Ident,
    /// the set of fields.
    pub(crate) fields: Vec<DataBuilder>,
    /// Imposes checks on the sizing of the `field_set`
    pub enforcement: StructEnforcement,
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
    pub default_endianness: Endianness,
}

impl FieldSetBuilder {
    #[must_use]
    pub fn new(key: Ident) -> Self {
        Self {
            name: key,
            fields: Vec::default(),
            enforcement: StructEnforcement::default(),
            fill_bits: FillBits::default(),
            default_endianness: Endianness::default(),
        }
    }
    pub fn add_field(&mut self, new_data: DataBuilder) {
        self.fields.push(new_data);
    }
    pub fn with_field(mut self, new_data: DataBuilder) -> Self {
        self.fields.push(new_data);
        self
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
#[derive(Debug, Clone, Default)]
pub enum StructEnforcement {
    /// No enforcement on the amount of bits used by the entire `field_set`
    #[default]
    NoRules,
    /// Enforce the `BIT_SIZE` equals `BYTE_SIZE` * 8
    EnforceFullBytes,
    /// Enforce the amount of bits that need to be used tot a specific value.
    EnforceBitAmount(usize),
}
