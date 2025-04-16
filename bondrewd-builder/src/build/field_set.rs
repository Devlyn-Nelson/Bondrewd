use proc_macro2::{Span, TokenStream};
use syn::{spanned::Spanned, DeriveInput, Error, Fields, Ident, LitStr};

use crate::solved::{field::get_number_type_ident, field_set::SolvedFieldSetAttributes};

use super::{
    field::{DataBuilder, DataType},
    Endianness,
};

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
        let darling = StructDarling::from_derive_input(input)?;
        // determine byte enforcement if any.
        let enforcement = if darling.enforce_full_bytes {
            if darling.enforce_bytes.is_none() && darling.enforce_bits.is_none() {
                return Err(Error::new(input.span(), "Please only use 1 byte enforcement attribute (enforce_full_bytes, enforce_bytes, enforce_bits)"));
            } else {
                StructEnforcement::EnforceFullBytes
            }
        } else if let Some(bytes) = darling.enforce_bytes {
            if darling.enforce_bits.is_none() {
                StructEnforcement::EnforceBitAmount(bytes * 8)
            } else {
                return Err(Error::new(input.span(), "Please only use 1 byte enforcement attribute (enforce_full_bytes, enforce_bytes, enforce_bits)"));
            }
        } else if let Some(bits) = darling.enforce_bits {
            StructEnforcement::EnforceBitAmount(bits)
        } else {
            StructEnforcement::NoRules
        };
        // determine byte filling if any.
        let fill_bits = if darling.fill {
            if darling.fill_bytes.is_none() && darling.fill_bits.is_none() {
                return Err(Error::new(
                    input.span(),
                    "Please only use 1 byte filling attribute (fill, fill_bits, fill_bytes)",
                ));
            } else {
                FillBits::Auto
            }
        } else if let Some(bytes) = darling.fill_bytes {
            if darling.fill_bits.is_none() {
                FillBits::Bits(bytes * 8)
            } else {
                return Err(Error::new(
                    input.span(),
                    "Please only use 1 byte filling attribute (fill, fill_bits, fill_bytes)",
                ));
            }
        } else if let Some(bits) = darling.fill_bits {
            FillBits::Bits(bits)
        } else {
            FillBits::None
        };
        let default_endianness = if let Some(val) = darling.default_endianness {
            Endianness::from_expr(&val)?
        } else {
            // TODO decide final default for endianness.
            Endianness::little_packed()
        }; // HERE
        match &input.data {
            syn::Data::Struct(data_struct) => {
                let tuple = matches!(data_struct.fields, syn::Fields::Unnamed(_));
                let mut fields = Vec::default();
                let bit_size = Self::extract_fields(
                    &mut fields,
                    &data_struct.fields,
                    &default_endianness,
                    tuple,
                )?;
                // START_HERE below commented code should be moved to the solving process.
                // match enforcement {
                //     StructEnforcement::NoRules => {}
                //     StructEnforcement::EnforceFullBytes => {
                //         if bit_size % 8 != 0 {
                //             return Err(syn::Error::new(
                //                 data_struct.struct_token.span(),
                //                 "BIT_SIZE modulus 8 is not zero",
                //             ));
                //         }
                //     }
                //     StructEnforcement::EnforceBitAmount(expected_total_bits) => {
                //         if bit_size != expected_total_bits {
                //             return Err(syn::Error::new(
                //                 data_struct.struct_token.span(),
                //                 format!(
                //                     "Bit Enforcement failed because bondrewd detected {bit_size} total bits used by defined fields, but the bit enforcement attribute is defined as {expected_total_bits} bits.",
                //                 ),
                //             ));
                //         }
                //     }
                // }
                // let first_bit = if let Some(last_range) = fields.iter().last() {
                //     last_range.bit_range.end
                // } else {
                //     0_usize
                // };
                // let auto_fill = match fill_bits {
                //     FillBits::None => None,
                //     FillBits::Bits(bits) => Some(bits),
                //     FillBits::Auto => {
                //         let unused_bits = bit_size % 8;
                //         if unused_bits == 0 {
                //             None
                //         } else {
                //             Some(8 - unused_bits)
                //             // None
                //         }
                //     }
                // };
                // add reserve for fill bytes. this happens after bit enforcement because bit_enforcement is for checking user code.
                // if let Some(fill_bits) = auto_fill {
                //     let end_bit = first_bit + fill_bits;
                //     bit_size += fill_bits;
                //     let fill_bytes_size = (end_bit - first_bit).div_ceil(8);
                //     let ident = quote::format_ident!("bondrewd_fill_bits");
                //     let mut endian = attrs.default_endianess.clone();
                //     if !endian.has_endianness() {
                //         endian.set_mode(crate::common::EndiannessMode::Standard);
                //     }
                //     bondrewd_fields.push(FieldInfo {
                //         ident: Box::new(ident.into()),
                //         attrs: Attributes {
                //             bit_range: first_bit..end_bit,
                //             endianness: Box::new(endian),
                //             reserve: ReserveFieldOption::FakeField,
                //             overlap: OverlapOptions::None,
                //             capture_id: false,
                //         },
                //         ty: DataType::BlockArray {
                //             sub_type: Box::new(SubFieldInfo {
                //                 ty: DataType::Number {
                //                     size: 1,
                //                     sign: NumberSignage::Unsigned,
                //                     type_quote: quote! {u8},
                //                 },
                //             }),
                //             length: fill_bytes_size,
                //             type_quote: quote! {[u8;#fill_bytes_size]},
                //         },
                //     });
                // }
                //
                let s = StructBuilder {
                    field_set: FieldSetBuilder {
                        name: darling.ident,
                        fields,
                        enforcement,
                        fill_bits,
                    },
                    attrs: SolvedFieldSetAttributes {
                        dump: darling.dump,
                        vis: super::Visibility(darling.vis),
                    },
                    tuple,
                };
                Ok(Self {
                    ty: BuilderType::Struct(Box::new(s)),
                })
            }
            syn::Data::Enum(data_enum) => Err(Error::new(
                Span::call_site(),
                "Enum support is in development",
            )),
            syn::Data::Union(_) => Err(Error::new(Span::call_site(), "input can not be a union")),
        }
    }
    /// `bondrewd_fields` should either be empty or have exactly 1 field. If a field is provided it is assumed that
    /// this "struct" is an enum variant and the field is the enum value.
    ///
    /// `syn_fields` is just the raw fields from the `syn` crate.
    ///
    /// `default_endianness` is the endianness any fields that do not have specified endianness will be given.
    ///
    /// `tuple` do the fields have names.
    ///
    /// Returns the total bits used by the fields.
    fn extract_fields(
        bondrewd_fields: &mut Vec<DataBuilder>,
        syn_fields: &Fields,
        default_endianness: &Endianness,
        tuple: bool,
    ) -> syn::Result<usize> {
        let is_enum = !bondrewd_fields.is_empty();
        let mut bit_size = if let Some(id_field) = bondrewd_fields.first() {
            id_field.bit_length()
        } else {
            0
        };
        let stripped_fields = match syn_fields {
            syn::Fields::Named(ref named_fields) => Some(
                named_fields
                    .named
                    .iter()
                    .cloned()
                    .collect::<Vec<syn::Field>>(),
            ),
            syn::Fields::Unnamed(ref fields) => {
                Some(fields.unnamed.iter().cloned().collect::<Vec<syn::Field>>())
            }
            syn::Fields::Unit => {
                if bit_size == 0 {
                    return Err(Error::new(Span::call_site(), "Packing a Unit Struct (Struct with no data) seems pointless to me, so i didn't write code for it."));
                }
                None
            }
        };
        // figure out what the field are and what/where they should be in byte form.
        if let Some(fields) = stripped_fields {
            for (i, ref field) in fields.iter().enumerate() {
                let mut parsed_field =
                    DataBuilder::parse(field, &bondrewd_fields, default_endianness)?;
                // let mut parsed_field = FieldInfo::from_syn_field(field, &bondrewd_fields, attrs)?;
                if parsed_field.is_captured_id {
                    if is_enum {
                        if i == 0 {
                            match (&bondrewd_fields[0].ty, &mut parsed_field.ty) {
                                (
                                    DataType::Number(number_ty_bon, rust_size_bon),
                                    DataType::Number(number_ty, rust_size)
                                ) => {
                                    let ty_ident = get_number_type_ident(number_ty, rust_size.bits());
                                    let ty_ident_bon = get_number_type_ident(number_ty_bon, rust_size_bon.bits());
                                    // TODO this if statements actions could cause confusing behavior
                                    if bondrewd_fields[0].bit_range != parsed_field.bit_range {
                                        parsed_field.bit_range = bondrewd_fields[0].bit_range.clone();
                                    }
                                    if number_ty_bon != number_ty {
                                        return Err(Error::new(field.span(), format!("`capture_id` field must be unsigned. bondrewd will enforce the type as {ty_ident_bon}")));
                                    }else if ty_ident_bon != ty_ident {
                                        return Err(Error::new(field.span(), format!("`capture_id` field currently must be {ty_ident_bon} in this instance, because bondrewd makes an assumption about the id type. changing this would be difficult")));
                                    }
                                    let old_id = bondrewd_fields.remove(0);
                                    if tuple {
                                        parsed_field.id = old_id.id;
                                    }
                                }
                                (DataType::Number(number_ty_bon, rust_size_bon), _) => {
                                    let ty_ident_bon = get_number_type_ident(number_ty_bon, rust_size_bon.bits());
                                    return Err(Error::new(field.span(), format!("capture_id field must be an unsigned number. detected type is {ty_ident_bon}.")))
                                }
                                _ => return Err(Error::new(field.span(), "an error with bondrewd has occurred, the id field should be a number but bondrewd did not use a number for the id.")),
                            }
                        } else {
                            return Err(Error::new(
                                field.span(),
                                "`capture_id` attribute must be the first field.",
                            ));
                        }
                    } else {
                        return Err(Error::new(
                            field.span(),
                            "`capture_id` attribute is intended for enum variants only.",
                        ));
                    }
                } else {
                    bit_size += parsed_field.bit_length();
                }
                bondrewd_fields.push(parsed_field);
            }
        }
        Ok(bit_size)
    }
    pub fn generate(&self) -> syn::Result<TokenStream> {
        Err(Error::new(Span::call_site(), "generate is not done"))
    }
}
#[derive(Debug, FromDeriveInput, FromVariant)]
#[darling(attributes(bondrewd))]
pub struct StructDarling {
    pub default_endianness: Option<LitStr>,
    pub reverse: bool,
    // Below are being used.
    pub ident: Ident,
    pub vis: syn::Visibility,
    pub dump: bool,
    pub enforce_full_bytes: bool,
    pub enforce_bytes: Option<usize>,
    pub enforce_bits: Option<usize>,
    pub fill_bits: Option<usize>,
    pub fill_bytes: Option<usize>,
    pub fill: bool,
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
    pub(crate) invalid: Option<VariantBuilder>,
    /// The collection of variant `field_sets`.
    pub(crate) variants: Vec<VariantBuilder>,
    pub(crate) attrs: SolvedFieldSetAttributes,
}

impl EnumBuilder {
    #[must_use]
    pub fn new(name: Ident) -> Self {
        Self {
            name,
            id: None,
            invalid: None,
            variants: Vec::default(),
            attrs: SolvedFieldSetAttributes::default(),
        }
    }
}
/// Contains builder information for constructing variant style bitfield models.
#[derive(Debug)]
pub struct VariantBuilder {
    /// The id value that this variant shall be used for.
    id: Option<i64>,
    /// If the variant has a field that whats to capture the
    /// value read for the variant resolution the fields shall be placed here
    /// NOT in the field set, useful for invalid variant.
    capture_field: Option<DataBuilder>,
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
}

impl FieldSetBuilder {
    #[must_use]
    pub fn new(key: Ident) -> Self {
        Self {
            name: key,
            fields: Vec::default(),
            enforcement: StructEnforcement::default(),
            fill_bits: FillBits::default(),
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
