use proc_macro2::Span;
use syn::{spanned::Spanned, DataEnum, DeriveInput, Error, Fields, Ident, LitStr};

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
        let enforcement = if darling.enforce_full_bytes.is_present() {
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
        let fill_bits = if darling.fill.is_present() {
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
        let default_endianness = if let Some(ref val) = darling.default_endianness {
            Endianness::from_expr(val)?
        } else {
            Endianness::default()
        };
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
                let s = StructBuilder {
                    field_set: FieldSetBuilder {
                        name: darling.ident,
                        fields,
                        enforcement,
                        fill_bits,
                        default_endianness,
                    },
                    attrs: SolvedFieldSetAttributes {
                        dump: darling.dump.is_present(),
                        vis: super::Visibility(darling.vis),
                    },
                    tuple,
                };
                Ok(Self {
                    ty: BuilderType::Struct(Box::new(s)),
                })
            }
            syn::Data::Enum(data_enum) => {
                let enum_attrs = EnumDarling::from_derive_input(input)?.try_into()?;
                Self::parse_enum(data_enum, &darling, &enum_attrs)
            }
            syn::Data::Union(_) => Err(Error::new(Span::call_site(), "input can not be a union")),
        }
    }

    fn parse_enum(
        data_enum: &DataEnum,
        struct_attrs: &StructDarling,
        enum_attrs: &EnumDarlingSimplified,
    ) -> syn::Result<Self> {
        // START_HERE implement enum parsing.

        // let mut variants: Vec<StructInfo> = Vec::default();
        // let (id_field_type, id_bits) = {
        //     let id_bits = if let Some(id_bits) = enum_attrs.id_bits {
        //         id_bits
        //     } else if let (Some(payload_size), Some(total_size)) =
        //         (enum_attrs.payload_bit_size, enum_attrs.total_bit_size)
        //     {
        //         total_size - payload_size
        //     } else {
        //         return Err(syn::Error::new(
        //                     data.enum_token.span(),
        //                     "Must define the length of the id use #[bondrewd(id_bit_length = AMOUNT_OF_BITS)]",
        //                 ));
        //     };
        //     (
        //         DataType::Number {
        //             size: id_bits.div_ceil(8),
        //             sign: NumberSignage::Unsigned,
        //             type_quote: get_id_type(id_bits, name.span())?,
        //         },
        //         id_bits,
        //     )
        // };
        // let id_field = FieldInfo {
        //     ident: Box::new(format_ident!("{}", EnumInfo::VARIANT_ID_NAME).into()),
        //     ty: id_field_type,
        //     attrs: Attributes {
        //         endianness: Box::new(attrs.default_endianness.clone()),
        //         // this need to accommodate tailing ids, currently this locks the
        //         // id field to the first field read from the starting point of reading.
        //         // TODO make sure this gets corrected if the id size is unknown.
        //         bit_range: 0..id_bits,
        //         reserve: ReserveFieldOption::EnumId,
        //         overlap: OverlapOptions::None,
        //         capture_id: false,
        //     },
        // };
        // for variant in &data.variants {
        //     let tuple = matches!(variant.fields, syn::Fields::Unnamed(_));
        //     let mut attrs = attrs.clone();
        //     if let Some((_, ref expr)) = variant.discriminant {
        //         let parsed = Self::parse_lit_discriminant_expr(expr)?;
        //         attrs.id = Some(parsed);
        //     }
        //     Self::parse_struct_attrs(&variant.attrs, &mut attrs, true)?;
        //     let variant_name = variant.ident.clone();
        //     // TODO currently we always add the id field, but some people might want the id to be a
        //     // field in the variant. this would no longer need to insert the id as a "fake-field".
        //     let fields = Self::parse_fields(
        //         &variant_name,
        //         &variant.fields,
        //         &attrs,
        //         Some(id_field.clone()),
        //         tuple,
        //     )?;
        //     variants.push(StructInfo {
        //         name: variant_name,
        //         attrs,
        //         fields,
        //         vis: crate::common::Visibility(input.vis.clone()),
        //         tuple,
        //     });
        // }
        // // detect and fix variants without ids and verify non conflict.
        // let mut used_ids: Vec<u128> = Vec::default();
        // let mut unassigned_indices: Vec<usize> = Vec::default();
        // let mut invalid_index: Option<usize> = None;
        // let mut largest = 0;
        // for (i, variant) in variants.iter().enumerate() {
        //     if let Some(ref value) = variant.attrs.id {
        //         if used_ids.contains(value) {
        //             return Err(Error::new(
        //                 variant.name.span(),
        //                 "variant identifier used twice.",
        //             ));
        //         }
        //         used_ids.push(*value);
        //     } else {
        //         unassigned_indices.push(i);
        //     }
        //     if variant.attrs.invalid {
        //         if invalid_index.is_none() {
        //             invalid_index = Some(i);
        //         } else {
        //             return Err(Error::new(
        //                 variant.name.span(),
        //                 "second catch invalid variant found. only 1 is currently allowed.",
        //             ));
        //         }
        //     }
        // }
        // if !unassigned_indices.is_empty() {
        //     let mut current_guess: u128 = 0;
        //     for i in unassigned_indices {
        //         while used_ids.contains(&current_guess) {
        //             current_guess += 1;
        //         }
        //         variants[i].attrs.id = Some(current_guess);
        //         used_ids.push(current_guess);
        //         current_guess += 1;
        //     }
        // }
        // for variant in &variants {
        //     // verify the size doesn't go over set size.
        //     let size = variant.total_bits();
        //     if largest < size {
        //         largest = size;
        //     }
        //     let variant_id_field = {
        //         if let Some(id) = variant.get_id_field()? {
        //             id
        //         } else {
        //             return Err(syn::Error::new(variant.name.span(), "failed to get variant field for variant. (this is a bondrewd issue, please report issue)"));
        //         }
        //     };

        //     if let Some(bit_size) = enum_attrs.payload_bit_size {
        //         if bit_size < size - variant_id_field.bit_size() {
        //             return Err(Error::new(
        //                         variant.name.span(),
        //                         format!("variant is larger than defined payload_size of enum. defined size: {bit_size}. variant size: {}", size- variant_id_field.bit_size()),
        //                     ));
        //         }
        //     } else if let (Some(bit_size), Some(id_size)) =
        //         (enum_attrs.total_bit_size, enum_attrs.id_bits)
        //     {
        //         if bit_size - id_size < size - variant_id_field.bit_size() {
        //             return Err(Error::new(
        //                         variant.name.span(),
        //                         format!("variant with id is larger than defined total_size of enum. defined size: {}. calculated size: {}", bit_size - id_size, size - variant_id_field.bit_size()),
        //                     ));
        //         }
        //     }
        // }
        // if let Some(ii) = invalid_index {
        //     let var = variants.remove(ii);
        //     variants.push(var);
        // }
        // // find minimal id size from largest id value
        // used_ids.sort_unstable();
        // let min_id_size = if let Some(last_id) = used_ids.last() {
        //     let mut x = *last_id;
        //     // find minimal id size from largest id value
        //     let mut n = 0;
        //     while x != 0 {
        //         x >>= 1;
        //         n += 1;
        //     }
        //     n
        // } else {
        //     return Err(Error::new(
        //         data.enum_token.span(),
        //         "found no variants and could not determine size of id".to_string(),
        //     ));
        // };
        // let enum_attrs = match (enum_attrs.payload_bit_size, enum_attrs.total_bit_size) {
        //     (Some(payload), None) => {
        //         if let Some(id) = enum_attrs.id_bits {
        //             EnumAttrInfo {
        //                 payload_bit_size: payload,
        //                 id_bits: id,
        //                 id_position: enum_attrs.id_position,
        //                 attrs: attrs.clone(),
        //             }
        //         } else {
        //             EnumAttrInfo {
        //                 payload_bit_size: payload,
        //                 id_bits: min_id_size,
        //                 id_position: enum_attrs.id_position,
        //                 attrs: attrs.clone(),
        //             }
        //         }
        //     }
        //     (None, Some(total)) => {
        //         if let Some(id) = enum_attrs.id_bits {
        //             EnumAttrInfo {
        //                 payload_bit_size: total - id,
        //                 id_bits: id,
        //                 id_position: enum_attrs.id_position,
        //                 attrs: attrs.clone(),
        //             }
        //         } else if largest < total {
        //             let id = total - largest;
        //             EnumAttrInfo {
        //                 payload_bit_size: largest,
        //                 id_bits: id,
        //                 id_position: enum_attrs.id_position,
        //                 attrs: attrs.clone(),
        //             }
        //         } else {
        //             return Err(Error::new(
        //                         data.enum_token.span(),
        //                         "specified total is not smaller than the largest payload size, meaning there is not room the the variant id.".to_string(),
        //                     ));
        //         }
        //     }
        //     (Some(payload), Some(total)) => {
        //         if let Some(id) = enum_attrs.id_bits {
        //             if payload + id != total {
        //                 return Err(Error::new(
        //                             data.enum_token.span(),
        //                             format!("total_size, payload_size, and id_size where all specified but id_size ({id}) + payload_size ({payload}) is not equal to total_size ({total})"),
        //                         ));
        //             }
        //             if payload < largest {
        //                 return Err(Error::new(
        //                     data.enum_token.span(),
        //                     "detected a variant over the maximum defined size.".to_string(),
        //                 ));
        //             }
        //             EnumAttrInfo {
        //                 id_bits: id,
        //                 id_position: enum_attrs.id_position,
        //                 payload_bit_size: payload,
        //                 attrs: attrs.clone(),
        //             }
        //         } else {
        //             EnumAttrInfo {
        //                 payload_bit_size: largest,
        //                 id_bits: min_id_size,
        //                 id_position: enum_attrs.id_position,
        //                 attrs: attrs.clone(),
        //             }
        //         }
        //     }
        //     _ => {
        //         if let Some(id) = enum_attrs.id_bits {
        //             EnumAttrInfo {
        //                 id_bits: id,
        //                 id_position: enum_attrs.id_position,
        //                 payload_bit_size: largest,
        //                 attrs: attrs.clone(),
        //             }
        //         } else {
        //             EnumAttrInfo {
        //                 payload_bit_size: largest,
        //                 id_bits: min_id_size,
        //                 id_position: enum_attrs.id_position,
        //                 attrs: attrs.clone(),
        //             }
        //         }
        //     }
        // };
        // if enum_attrs.id_bits < min_id_size {
        //     return Err(Error::new(
        //         data.enum_token.span(),
        //         "the bit size being used is less than required to describe each variant"
        //             .to_string(),
        //     ));
        // }
        // if enum_attrs.payload_bit_size + enum_attrs.id_bits < largest {
        //     return Err(Error::new(
        //         data.enum_token.span(),
        //         "the payload size being used is less than largest variant".to_string(),
        //     ));
        // }
        // let id_field_ty = FieldDataType::Number(
        //     enum_attrs.id_bits,
        //     NumberSignage::Unsigned,
        //     get_id_type(enum_attrs.id_bits, name.span())?,
        // );
        // add fill_bits if needed.
        // TODO fix fill byte getting inserted of wrong side sometimes.
        // the problem is, things get calculated before fill is added. also fill might be getting added when it shouldn't.
        // for v in &mut variants {
        //     let first_bit = v.total_bits();
        //     if first_bit < largest {
        //         let fill_bytes_size = (largest - first_bit).div_ceil(8);
        //         let ident = quote::format_ident!("enum_fill_bits");
        //         let fill = FieldInfo {
        //             ident: Box::new(ident.into()),
        //             attrs: Attributes {
        //                 bit_range: first_bit..largest,
        //                 endianness: Box::new(Endianness::big()),
        //                 reserve: ReserveFieldOption::FakeField,
        //                 overlap: OverlapOptions::None,
        //                 capture_id: false,
        //             },
        //             ty: DataType::BlockArray {
        //                 sub_type: Box::new(SubFieldInfo {
        //                     ty: DataType::Number {
        //                         size: 1,
        //                         sign: NumberSignage::Unsigned,
        //                         type_quote: quote! {u8},
        //                     },
        //                 }),
        //                 length: fill_bytes_size,
        //                 type_quote: quote! {[u8;#fill_bytes_size]},
        //             },
        //         };
        //         if v.attrs.default_endianess.is_byte_order_reversed() {
        //             v.fields.insert(0, fill);
        //         } else {
        //             v.fields.push(fill);
        //         }
        //     }
        // }
        // let out = Self::Enum(EnumInfo {
        //     name,
        //     variants,
        //     attrs: enum_attrs,
        //     vis: crate::common::Visibility(input.vis.clone()),
        // });
        // println!("enum - {out:?}");
        // Ok(out)
        //
        Err(Error::new(
            Span::call_site(),
            "Enum support is in development",
        ))
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
}
#[derive(Debug)]
pub struct EnumDarlingSimplified {
    pub ident: Ident,
    pub vis: syn::Visibility,
    pub payload_bit_length: Option<usize>,
    pub id_bit_length: Option<usize>,
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
