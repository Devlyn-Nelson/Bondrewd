use std::{collections::BTreeMap, env::current_dir, ops::Range, str::FromStr};

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;
use thiserror::Error;

use crate::{
    build::{
        field::{
            DataBuilderRange, DataType, FullDataType, FullDataTypeArraySpecType, NumberType,
            RustByteSize,
        },
        field_set::{
            EnumBuilder, FieldSetBuilder, FillBits, GenericBuilder, StructBuilder,
            StructEnforcement, VariantBuilder,
        },
        ArraySizings, Endianness, OverlapOptions, ReserveFieldOption, Visibility,
    },
    derive::{
        quotes::{GeneratedDynFunctions, GeneratedFunctions},
        GenStructFieldsEnumInfo, SolvedFieldSetAdditive,
    },
};

use super::field::{DynamicIdent, SolvedData};

#[derive(Debug)]
pub struct Solved {
    /// `DataSet`'s name.
    ///
    /// for derive this would be the Enum or Struct ident.
    pub(crate) name: Ident,
    pub(crate) ty: SolvedType,
}
#[derive(Debug)]
pub enum SolvedType {
    Enum {
        /// The id field. or the field that determines the variant.
        id: SolvedData,
        /// The default variant. in the case not other variant matches, this will be used.
        invalid: SolvedFieldSet,
        /// The default variant's name/ident
        invalid_name: VariantInfo,
        /// Sets of fields, each representing a variant of an enum. the String
        /// being the name of the variant
        variants: BTreeMap<VariantInfo, SolvedFieldSet>,
        dump: bool,
    },
    Struct(SolvedFieldSet),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantInfo {
    pub(crate) id: usize,
    pub(crate) name: Ident,
    pub(crate) tuple: bool,
}

impl PartialOrd for VariantInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for VariantInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug)]
pub struct SolvedFieldSet {
    pub(crate) attrs: SolvedFieldSetAttributes,
    pub(crate) fields: Vec<SolvedData>,
}

#[derive(Debug, Clone)]
pub struct SolvedFieldSetAttributes {
    pub dump: bool,
    pub vis: Visibility,
}

impl Default for SolvedFieldSetAttributes {
    fn default() -> Self {
        Self {
            vis: Visibility(syn::Visibility::Public(syn::token::Pub {
                span: proc_macro2::Span::call_site(),
            })),
            dump: false,
        }
    }
}

#[derive(Debug, Error)]
pub enum SolvingError {
    /// Fields overlaps
    #[error("Fields overlap")]
    Overlap,
    /// Tried to solve a field without a type.
    ///
    /// # Field
    /// the `String` provided should be the id or name of the field.
    #[error("No data type was provided for field with id {0}")]
    NoTypeProvided(String),
    /// Tried to solve a number field without endianness.
    ///
    /// # Field
    /// the `String` provided should be the id or name of the field.
    #[error("No endianness was provided for field with id {0}")]
    NoEndianness(String),
    /// [`Resolver::new`] had a left shift underflow.
    ///
    /// # Field
    /// the `String` provided should be the id or name of the field.
    #[error("Failed solving the `left_shift` due to underflow.")]
    ResolverUnderflow(String),
    #[error("Final bit count was not evenly divisible by 0.")]
    EnforceFullBytes,
    #[error("Final bit count does not match enforcement size.[user = {user}, actual = {actual}]")]
    EnforceBitCount { actual: usize, user: usize },
    #[error("Variant name \"{0}\" used twice. Variants must have unique names.")]
    VariantConflict(Ident),
    #[error("Largest variant id value ({variant_id_bit_length}) is larger than `id_bit_size` ({bit_length})")]
    VariantIdBitLength {
        variant_id_bit_length: usize,
        bit_length: usize,
    },
    #[error("Largest variant payload ({largest_payload_length}) is larger than `payload_bit_size` ({bit_length})")]
    VariantPayloadBitLength {
        largest_payload_length: usize,
        bit_length: usize,
    },
    #[error("The variant id must have a type of: u8, u16, u32, u64, or u128, variant bit length is currently {0} and bondrewd doesn't know which type use.")]
    VariantIdType(usize),
    #[error("A feature you are trying to use in not yet supported. complain to the Bondrewd maintainer about \"{0}\"")]
    Unfinished(String),
}

impl From<SolvingError> for syn::Error {
    fn from(value: SolvingError) -> Self {
        syn::Error::new(Span::call_site(), format!("{value}"))
    }
}

impl TryFrom<GenericBuilder> for Solved {
    type Error = SolvingError;

    fn try_from(value: GenericBuilder) -> Result<Self, Self::Error> {
        match value.ty {
            crate::build::field_set::BuilderType::Enum(e) => (*e).try_into(),
            crate::build::field_set::BuilderType::Struct(s) => (*s).try_into(),
        }
    }
}

/// Contains builder information for constructing variant style bitfield models.
#[derive(Debug)]
pub struct VariantBuilt {
    /// The id value that this variant shall be used for.
    pub(crate) id: usize,
    /// the `field_set`
    pub(crate) field_set: FieldSetBuilder,
    pub(crate) tuple: bool,
}

fn bits_needed(x: usize) -> usize {
    let mut x = x;
    // find minimal id size from largest id value
    let mut n = 0;
    while x != 0 {
        x >>= 1;
        n += 1;
    }
    n
}

fn get_built_variant(
    variant: VariantBuilder,
    used_ids: &mut Vec<usize>,
    next: &mut usize,
    largest_variant_id: &mut usize,
) -> Result<VariantBuilt, SolvingError> {
    let id = if let Some(value) = variant.id {
        if used_ids.contains(&value) {
            return Err(SolvingError::VariantConflict(
                variant.field_set.name.clone(),
            ));
        }
        value
    } else {
        let mut guess = *next;
        while used_ids.contains(&guess) {
            guess += 1;
        }
        guess
    };
    *next = id + 1;
    if *largest_variant_id < id {
        *largest_variant_id = id;
    }
    used_ids.push(id);
    Ok(VariantBuilt {
        id,
        field_set: variant.field_set,
        tuple: variant.tuple,
    })
}

impl TryFrom<EnumBuilder> for Solved {
    type Error = SolvingError;

    fn try_from(value: EnumBuilder) -> Result<Self, Self::Error> {
        let variants = value.variants;
        // give all variants ids.
        // TODO this code was in the parsing code, but has been moved here.
        let mut used_ids: Vec<usize> = Vec::default();
        let mut last = 0;
        let mut built_variants = Vec::<VariantBuilt>::with_capacity(variants.len());
        let mut largest_variant_id = 0;
        let mut largest_bit_size = 0;
        for variant in variants {
            let built =
                get_built_variant(variant, &mut used_ids, &mut last, &mut largest_variant_id)?;
            built_variants.push(built);
        }
        let built_invalid = get_built_variant(
            value.invalid,
            &mut used_ids,
            &mut last,
            &mut largest_variant_id,
        )?;
        // determine id field.
        let (id_field_type, id_bits) = {
            let id_bits = if let Some(id_bits) = value.id_bit_length {
                id_bits
            } else if let (Some(payload_size), StructEnforcement::EnforceBitAmount(total_size)) =
                (&value.payload_bit_length, &value.attrs.enforcement)
            {
                total_size - payload_size
            } else {
                bits_needed(largest_variant_id)
            };
            let bytes = match id_bits.div_ceil(8) {
                1 => RustByteSize::One,
                2 => RustByteSize::Two,
                3..=4 => RustByteSize::Four,
                5..=8 => RustByteSize::Eight,
                9..=16 => RustByteSize::Sixteen,
                invalid => return Err(SolvingError::VariantIdType(invalid)),
            };
            (DataType::Number(NumberType::Unsigned, bytes), id_bits)
        };
        // validity checks
        if bits_needed(largest_variant_id) > id_bits {
            return Err(SolvingError::VariantIdBitLength {
                variant_id_bit_length: largest_variant_id,
                bit_length: id_bits,
            });
        }

        // TODO try to enhance id field detection, use any hints given such as `capture_id` fields.
        let id_field = BuiltData {
            id: format_ident!("{}", EnumBuilder::VARIANT_ID_NAME).into(),
            ty: id_field_type,
            bit_range: BuiltRange {
                bit_range: 0..id_bits,
                ty: BuiltRangeType::SingleElement,
            },
            endianness: value.attrs.default_endianness,
            reserve: ReserveFieldOption::FakeField,
            overlap: OverlapOptions::None,
            is_captured_id: false,
        };

        let mut solved_variants = BTreeMap::default();
        let (invalid_name, mut invalid, invalid_fill) = Self::solve_variant(
            built_invalid,
            &id_field,
            &value.solved_attrs,
            &mut largest_bit_size,
            id_bits,
        )?;
        for variant in built_variants {
            let (variant_info, solved_variant, fill) = Self::solve_variant(
                variant,
                &id_field,
                &value.solved_attrs,
                &mut largest_bit_size,
                id_bits,
            )?;
            solved_variants.insert(variant_info, (solved_variant, fill));
        }
        // after solving the attrs for fill might be set. need to do it here
        // because the largest_payload_size can't be determined until `solve_variant`
        // has been called on all variants.
        Self::maybe_add_fill_field(&invalid_fill, &mut invalid, true)?;
        for (info, (set, fill)) in &mut solved_variants {
            Self::maybe_add_fill_field(fill, set, true)?;
        }
        let bit_size = largest_bit_size + id_bits;
        match value.attrs.enforcement {
            StructEnforcement::NoRules => {}
            StructEnforcement::EnforceFullBytes => {
                if bit_size % 8 != 0 {
                    return Err(SolvingError::EnforceFullBytes);
                }
            }
            StructEnforcement::EnforceBitAmount(expected_total_bits) => {
                if bit_size != expected_total_bits {
                    return Err(SolvingError::EnforceBitCount {
                        actual: bit_size,
                        user: expected_total_bits,
                    });
                }
            }
        }
        Ok(Solved {
            name: value.name,
            ty: SolvedType::Enum {
                id: id_field.into(),
                invalid,
                invalid_name,
                variants: solved_variants
                    .into_iter()
                    .map(|(info, (set, fill))| (info, set))
                    .collect(),
                dump: value.solved_attrs.dump,
            },
        })
        // // verify the size doesn't go over set size.
        // for variant in &variants {
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
        // TODO add validity check that ensures all capture-id fields are valid.
        // // add fill_bits if needed.
        // // TODO fix fill byte getting inserted of wrong side sometimes.
        // // the problem is, things get calculated before fill is added. also fill might be getting added when it shouldn't.
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
        // // TODO make the id_field of truth. or the one that bondrewd actually reads NOT captured id_fields.
        // // the truth Id_field should also match the capture_id fields that do exist. use below code for this.
        //
        // todo!("write conversion from EnumBuilder to Solved")
    }
}

fn detect_variant_fill(
    variant_payload_length: usize,
    largest_bit_size: usize,
    id_bits: usize,
    fill_bits: &FillBits,
) -> Result<FillBits, SolvingError> {
    if largest_bit_size < variant_payload_length {
        return Err(SolvingError::VariantPayloadBitLength {
            largest_payload_length: largest_bit_size,
            bit_length: variant_payload_length,
        });
    }
    Ok(
        if matches!(fill_bits, FillBits::Auto) || variant_payload_length < largest_bit_size {
            let mut target = largest_bit_size + id_bits;
            target = target.div_ceil(8) * 8;
            let fill = target - (id_bits + variant_payload_length);
            FillBits::Bits(fill)
        } else {
            FillBits::None
        },
    )
}

impl TryFrom<StructBuilder> for Solved {
    type Error = SolvingError;

    fn try_from(value: StructBuilder) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&StructBuilder> for Solved {
    type Error = SolvingError;

    fn try_from(value: &StructBuilder) -> Result<Self, Self::Error> {
        let fs = Self::try_from_field_set(&value.field_set, &value.attrs, None)?;
        Ok(Self {
            name: value.field_set.name.clone(),
            ty: SolvedType::Struct(fs),
        })
    }
}

impl Solved {
    fn solve_variant(
        variant: VariantBuilt,
        id_field: &BuiltData,
        solved_attrs: &SolvedFieldSetAttributes,
        largest_bit_size: &mut usize,
        id_bits: usize,
    ) -> Result<(VariantInfo, SolvedFieldSet, FillBits), SolvingError> {
        let solved_variant =
            Self::try_from_field_set(&variant.field_set, solved_attrs, Some(id_field))?;

        let bit_length = solved_variant.total_bits_no_fill();
        if *largest_bit_size < bit_length {
            *largest_bit_size = bit_length
        }
        let variant_info = VariantInfo {
            id: variant.id,
            name: variant.field_set.name,
            tuple: variant.tuple,
        };
        Ok((variant_info, solved_variant, variant.field_set.fill_bits))
    }
    pub fn total_bits_no_fill(&self) -> usize {
        match &self.ty {
            SolvedType::Enum {
                id,
                invalid,
                invalid_name,
                variants,
                dump,
            } => {
                let mut largest = 0;
                for var in variants {
                    let other = var.1.total_bits_no_fill();
                    if other > largest {
                        largest = other;
                    }
                }
                let id_length = id.bit_length();
                largest + id_length
            }
            SolvedType::Struct(solved_field_set) => solved_field_set.total_bits_no_fill(),
        }
    }
    pub fn total_bytes_no_fill(&self) -> usize {
        self.total_bits_no_fill().div_ceil(8)
    }
    pub fn gen(&self, dyn_fns: bool, hex_fns: bool, setters: bool) -> syn::Result<TokenStream> {
        let struct_name = &self.name;
        let struct_size = self.total_bytes_no_fill();
        let gen = match &self.ty {
            SolvedType::Enum {
                id,
                invalid,
                invalid_name,
                variants,
                dump,
            } => Self::gen_enum(
                struct_name,
                id,
                invalid,
                invalid_name,
                variants,
                struct_size,
                dyn_fns,
            )?,
            SolvedType::Struct(solved_field_set) => solved_field_set
                .generate_quotes(struct_name, None, struct_size, dyn_fns)?
                .finish(),
        };
        // get the struct size and name so we can use them in a quote.
        let impl_fns = gen.non_trait;
        // get the bit size of the entire set of fields to fill in trait requirement.
        let bit_size = self.total_bits_no_fill();
        let (mut output, bit_size) = match self.ty {
            SolvedType::Struct(..) => {
                if setters {
                    return Err(syn::Error::new(
                        self.name.span(),
                        "Setters are currently unsupported",
                    ));
                    // TODO get setter for arrays working.
                    // get the setters, functions that set a field disallowing numbers
                    // outside of the range the Bitfield.
                    // let setters_quote = match struct_fns::create_setters_quotes(struct_info) {
                    //     Ok(parsed_struct) => parsed_struct,
                    //     Err(err) => {
                    //         return Err(err);
                    //     }
                    // };
                    // quote! {
                    //     impl #struct_name {
                    //         #impl_fns
                    //         #setters_quote
                    //     }
                    // }
                } else {
                    (
                        quote! {
                            impl #struct_name {
                                #impl_fns
                            }
                        },
                        bit_size,
                    )
                }
            }
            SolvedType::Enum { ref id, .. } => {
                // TODO implement getters and setters for enums.
                (
                    quote! {
                        impl #struct_name {
                            #impl_fns
                        }
                    },
                    bit_size,
                )
            }
        };
        let trait_impl_fn = gen.bitfield_trait;
        output = quote! {
            #output
            impl bondrewd::Bitfields<#struct_size> for #struct_name {
                const BIT_SIZE: usize = #bit_size;
                #trait_impl_fn
            }
        };
        if hex_fns {
            let hex_size = struct_size * 2;
            output = quote! {
                #output
                impl bondrewd::BitfieldHex<#hex_size, #struct_size> for #struct_name {}
            };
            if dyn_fns {
                output = quote! {
                    #output
                    impl bondrewd::BitfieldHexDyn<#hex_size, #struct_size> for #struct_name {}
                };
            }
        }
        if let Some(dyn_fns) = gen.dyn_fns {
            let checked_structs = dyn_fns.checked_struct;
            let from_vec_quote = dyn_fns.bitfield_dyn_trait;
            output = quote! {
                #output
                #checked_structs
                impl bondrewd::BitfieldsDyn<#struct_size> for #struct_name {
                    #from_vec_quote
                }
            }
        }
        if self.dump() {
            let name = self.name.to_string().to_case(Case::Snake);
            match current_dir() {
                Ok(mut file_name) => {
                    file_name.push("target/bondrewd_debug");
                    let _ = std::fs::create_dir_all(&file_name);
                    file_name.push(format!("{name}_code_gen.rs"));
                    let _ = std::fs::write(file_name, output.to_string());
                }
                Err(err) => {
                    return Err(syn::Error::new(self.name.span(), format!("Failed to dump code gen because target folder could not be located. remove `dump` from struct or enum bondrewd attributes. [{err}]")));
                }
            }
        }
        Ok(output)
    }
    fn dump(&self) -> bool {
        match &self.ty {
            SolvedType::Enum {
                id,
                invalid,
                invalid_name,
                variants,
                dump,
            } => *dump,
            SolvedType::Struct(solved_field_set) => solved_field_set.attrs.dump,
        }
    }
    fn try_from_field_set(
        value: &FieldSetBuilder,
        attrs: &SolvedFieldSetAttributes,
        id_field: Option<&BuiltData>,
    ) -> Result<SolvedFieldSet, SolvingError> {
        // TODO solve arrays.
        //
        // let bit_size = if let Some(id_field) = id_field {
        //     id_field.bit_length()
        // } else {
        //     0
        // };
        //
        // TODO verify captured id fields match the generated id field using below commented code.
        // if let Some(bondrewd_field) = id_field {
        //     if i == 0 {
        //         match (&bondrewd_field.ty, &mut parsed_field.ty) {
        //             (
        //                 DataType::Number(number_ty_bon, rust_size_bon),
        //                 DataType::Number(number_ty, rust_size)
        //             ) => {
        //                 let ty_ident = get_number_type_ident(number_ty, rust_size.bits());
        //                 let ty_ident_bon = get_number_type_ident(number_ty_bon, rust_size_bon.bits());
        //                 // TODO this if statements actions could cause confusing behavior
        //                 if bondrewd_field.bit_range != parsed_field.bit_range {
        //                     parsed_field.bit_range = bondrewd_field.bit_range.clone();
        //                 }
        //                 if number_ty_bon != number_ty {
        //                     return Err(Error::new(field.span(), format!("`capture_id` field must be unsigned. bondrewd will enforce the type as {ty_ident_bon}")));
        //                 }else if ty_ident_bon != ty_ident {
        //                     return Err(Error::new(field.span(), format!("`capture_id` field currently must be {ty_ident_bon} in this instance, because bondrewd makes an assumption about the id type. changing this would be difficult")));
        //                 }
        //             }
        //             (DataType::Number(number_ty_bon, rust_size_bon), _) => {
        //                 let ty_ident_bon = get_number_type_ident(number_ty_bon, rust_size_bon.bits());
        //                 return Err(Error::new(field.span(), format!("capture_id field must be an unsigned number. detected type is {ty_ident_bon}.")))
        //             }
        //             _ => return Err(Error::new(field.span(), "an error with bondrewd has occurred, the id field should be a number but bondrewd did not use a number for the id.")),
        //         }
        //     } else {
        //         return Err(Error::new(
        //             field.span(),
        //             "`capture_id` attribute must be the first field.",
        //         ));
        //     }
        // } else {
        //     return Err(Error::new(
        //         field.span(),
        //         "`capture_id` attribute is intended for enum variants only.",
        //     ));
        // }
        let mut pre_fields: Vec<BuiltData> = Vec::default();
        let mut last_end_bit_index: Option<usize> = id_field.map(|f| f.bit_range.bit_length());
        let total_fields = value.fields.len();
        let fields_ref = &value.fields;
        // First stage checks for validity
        for value_field in fields_ref {
            // get resolved range for the field.
            let bit_range = if value_field.is_captured_id {
                if let Some(id) = id_field {
                    id.bit_range.clone()
                } else {
                    return Err(SolvingError::Unfinished(
                        "Field was marked as capture_id, but is not an enum variant".to_string(),
                    ));
                }
            } else {
                BuiltRange::from_builder(
                    &value_field.bit_range,
                    &value_field.ty,
                    last_end_bit_index,
                )
            };
            // get_range(&value_field.bit_range, &rust_size, last_end_bit_index);
            // update internal last_end_bit_index to allow automatic bit-range feature to work.
            if !value_field.overlap.is_redundant() {
                last_end_bit_index = Some(bit_range.end());
            }
            let ty = value_field.ty.data_type.clone();
            let nested = ty.needs_endianness();
            let field = BuiltData {
                endianness: if let Some(e) = &value_field.endianness {
                    e.clone()
                } else if ty.needs_endianness() {
                    return Err(SolvingError::NoEndianness(format!(
                        "{}",
                        value_field.id.ident()
                    )));
                } else {
                    // TODO determine if using big endian for nested objects is the correct answer.
                    Endianness::big()
                },
                ty,
                bit_range,
                id: value_field.id.clone(),
                reserve: value_field.reserve.clone(),
                overlap: value_field.overlap.clone(),
                is_captured_id: value_field.is_captured_id,
            };
            let field_range = field.bit_range.range();
            for other in &pre_fields {
                if field.conflict(other) {
                    return Err(SolvingError::Overlap);
                }
            }
            // let name = format!("{}", field.id);
            pre_fields.push(field);
        }
        let mut fields: Vec<SolvedData> = Vec::default();
        for pre_field in pre_fields {
            if let Some(field) = id_field {
                if field.conflict(&pre_field) {
                    return Err(SolvingError::Overlap);
                }
            }
            fields.push(SolvedData::from(pre_field));
        }
        let mut out = SolvedFieldSet {
            fields,
            attrs: attrs.clone(),
        };
        // let keys: Vec<DynamicIdent> = fields.keys().cloned().collect();
        // for key in keys {
        //     let field = fields.get(&key);
        // }
        let bit_size = out.total_bits();
        match value.attrs.enforcement {
            StructEnforcement::NoRules => {}
            StructEnforcement::EnforceFullBytes => {
                if bit_size % 8 != 0 {
                    return Err(SolvingError::EnforceFullBytes);
                }
            }
            StructEnforcement::EnforceBitAmount(expected_total_bits) => {
                if bit_size != expected_total_bits {
                    return Err(SolvingError::EnforceBitCount {
                        actual: bit_size,
                        user: expected_total_bits,
                    });
                }
            }
        }

        // add reserve for fill bytes. this happens after bit enforcement because bit_enforcement is for checking user code.
        Self::maybe_add_fill_field(&value.fill_bits, &mut out, id_field.is_some())?;
        Ok(out)
    }
    fn maybe_add_fill_field(
        fill: &FillBits,
        out: &mut SolvedFieldSet,
        has_id_field: bool,
    ) -> Result<(), SolvingError> {
        let bit_size = out.total_bits();
        let auto_fill = match fill {
            FillBits::None => None,
            FillBits::Bits(bits) => Some(*bits),
            FillBits::Auto => {
                if has_id_field {
                    return Err(SolvingError::Unfinished(
                        "Auto fill_bits is currently not finished for enums.".to_string(),
                    ));
                }
                let unused_bits = bit_size % 8;
                if unused_bits == 0 {
                    None
                } else {
                    Some(8 - unused_bits)
                    // None
                }
            }
        };
        if let Some(fill_bits) = auto_fill {
            let first_bit = if let Some(last_range) = out.fields.iter().last() {
                last_range.bit_range().end
            } else {
                0_usize
            };
            let end_bit = first_bit + fill_bits;
            // bit_size += fill_bits;
            let fill_bytes_size = (end_bit - first_bit).div_ceil(8);
            let ident = quote::format_ident!("bondrewd_fill_bits");
            // fields.push(FieldInfo {
            //     ident: Box::new(ident.into()),
            //     attrs: Attributes {
            //         bit_range: first_bit..end_bit,
            //         endianness: Box::new(endian),
            //         reserve: ReserveFieldOption::FakeField,
            //         overlap: OverlapOptions::None,
            //         capture_id: false,
            //     },
            //     ty: DataType::BlockArray {
            //         sub_type: Box::new(SubFieldInfo {
            //             ty: DataType::Number {
            //                 size: 1,
            //                 sign: NumberSignage::Unsigned,
            //                 type_quote: quote! {u8},
            //             },
            //         }),
            //         length: fill_bytes_size,
            //         type_quote: quote! {[u8;#fill_bytes_size]},
            //     },
            // });
            let fill_field = BuiltData {
                id: ident.into(),
                ty: DataType::Number(NumberType::Unsigned, RustByteSize::One),
                bit_range: BuiltRange {
                    bit_range: first_bit..end_bit,
                    ty: BuiltRangeType::BlockArray(vec![fill_bytes_size]),
                },
                endianness: Endianness::default(),
                reserve: ReserveFieldOption::FakeField,
                overlap: OverlapOptions::None,
                is_captured_id: false,
            };
            out.fields.push(fill_field.into());
        }
        Ok(())
    }
    fn gen_enum(
        enum_name: &Ident,
        id: &SolvedData,
        invalid: &SolvedFieldSet,
        invalid_name: &VariantInfo,
        variants: &BTreeMap<VariantInfo, SolvedFieldSet>,
        struct_size: usize,
        dyn_fns: bool,
    ) -> syn::Result<GeneratedFunctions> {
        let mut gen_read = GeneratedFunctions::new(dyn_fns);
        let mut gen_write = GeneratedFunctions::new(dyn_fns);
        let set_add = SolvedFieldSetAdditive::new_struct(enum_name);
        let field_access = id.get_quotes()?;
        // TODO pass field list into make read function.
        invalid.make_read_fns(
            id,
            &set_add,
            &mut quote! {},
            &mut gen_read,
            &field_access,
            struct_size,
        )?;
        invalid.make_write_fns(id, &set_add, &mut gen_write, &field_access, struct_size)?;
        gen_read.merge(&gen_write);
        let mut gen = gen_read;
        if let Some(ref mut thing) = gen.dyn_fns {
            thing.checked_struct = quote! {};
        }
        // TODO generate slice functions for id field.
        // let id_slice_read = generate_read_slice_field_fn(
        //     access.read(),
        //     &field,
        //     &temp_struct_info,
        //     field_name,
        // );
        // let id_slice_write = generate_write_slice_field_fn(
        //     access.write(),
        //     access.zero(),
        //     &field,
        //     &temp_struct_info,
        //     field_name,
        // );
        // quote! {
        //     #output
        //     #id_slice_read
        //     #id_slice_write
        // }

        // let last_variant = self.variants.len() - 1;
        // stores all of the into/from bytes functions across variants.
        let mut into_bytes_fn: TokenStream = quote! {};
        let mut from_bytes_fn: TokenStream = quote! {};
        // stores the build up for the id function.
        let mut id_fn: TokenStream = quote! {};
        // stores the build up for the `check_slice` fn for an enum.
        let (mut check_slice_fn, checked_ident): (TokenStream, Ident) =
            (quote! {}, format_ident!("{enum_name}Checked"));
        // stores the build up for the `check_slice_mut` fn for an enum.
        let (mut check_slice_mut_fn, checked_ident_mut): (TokenStream, Ident) =
            (quote! {}, format_ident!("{enum_name}CheckedMut"));
        // Stores a build up for creating a match enum type that contains CheckStruct for each variant.
        let (mut checked_slice_enum, mut checked_slice_enum_mut, mut lifetime): (
            TokenStream,
            TokenStream,
            bool,
        ) = (quote! {}, quote! {}, false);
        // the string `variant_id` as an Ident
        let v_id = format_ident!("{}", EnumBuilder::VARIANT_ID_NAME);
        // setup function names for getting variant id.
        let v_id_read_call = format_ident!("read_{v_id}");
        let v_id_write_call = format_ident!("write_{v_id}");
        let v_id_read_slice_call = format_ident!("read_slice_{v_id}");
        for (variant_info, variant) in variants.iter() {
            Self::gen_variant(
                GenVariant {
                    id: &id,
                    gen: &mut gen,
                    variant_info: &variant_info,
                    variant: &variant,
                    checked_ident: &checked_ident,
                    checked_ident_mut: &checked_ident_mut,
                    enum_name: &enum_name,
                    v_id: &v_id,
                    v_id_write_call: &v_id_write_call,
                    check_slice_fn: &mut check_slice_fn,
                    check_slice_mut_fn: &mut check_slice_mut_fn,
                    checked_slice_enum: &mut checked_slice_enum,
                    checked_slice_enum_mut: &mut checked_slice_enum_mut,
                    into_bytes_fn: &mut into_bytes_fn,
                    from_bytes_fn: &mut from_bytes_fn,
                    id_fn: &mut id_fn,
                    lifetime: &mut lifetime,
                    dyn_fns,
                },
                struct_size,
                false,
            )?;
        }
        Self::gen_variant(
            GenVariant {
                id: &id,
                gen: &mut gen,
                variant_info: &invalid_name,
                variant: &invalid,
                checked_ident: &checked_ident,
                checked_ident_mut: &checked_ident_mut,
                enum_name: &enum_name,
                v_id: &v_id,
                v_id_write_call: &v_id_write_call,
                check_slice_fn: &mut check_slice_fn,
                check_slice_mut_fn: &mut check_slice_mut_fn,
                checked_slice_enum: &mut checked_slice_enum,
                checked_slice_enum_mut: &mut checked_slice_enum_mut,
                into_bytes_fn: &mut into_bytes_fn,
                from_bytes_fn: &mut from_bytes_fn,
                id_fn: &mut id_fn,
                lifetime: &mut lifetime,
                dyn_fns,
            },
            struct_size,
            true,
        )?;
        // Finish `from_bytes` function.
        from_bytes_fn = quote! {
            fn from_bytes(input_byte_buffer: [u8;#struct_size]) -> Self {
                let #v_id = Self::#v_id_read_call(&input_byte_buffer);
                match #v_id {
                    #from_bytes_fn
                }
            }
        };
        if let Some(dyn_fns_gen) = &mut gen.dyn_fns {
            let from_vec_fn_inner = dyn_fns_gen.bitfield_dyn_trait.clone();
            let comment_take = "Creates a new instance of `Self` by copying field from the bitfields, removing bytes that where used. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
            let comment = "Creates a new instance of `Self` by copying field from the bitfields. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
            dyn_fns_gen.bitfield_dyn_trait = quote! {
                #[doc = #comment]
                fn from_slice(input_byte_buffer: &[u8]) -> Result<Self, bondrewd::BitfieldLengthError> {
                    if input_byte_buffer.len() < Self::BYTE_SIZE {
                        return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                    }
                    let #v_id = Self::#v_id_read_slice_call(&input_byte_buffer)?;
                    let out = match #v_id {
                        #from_vec_fn_inner
                    };
                    Ok(out)
                }
            };
            #[cfg(feature = "std")]
            {
                let from_vec_fn = &dyn_fns_gen.bitfield_dyn_trait;
                dyn_fns_gen.bitfield_dyn_trait = quote! {
                    #from_vec_fn
                    #[doc = #comment_take]
                    fn from_vec(input_byte_buffer: &mut Vec<u8>) -> Result<Self, bondrewd::BitfieldLengthError> {
                        if input_byte_buffer.len() < Self::BYTE_SIZE {
                            return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                        }
                        let #v_id = Self::#v_id_read_slice_call(&input_byte_buffer)?;
                        let out = match #v_id {
                            #from_vec_fn_inner
                        };
                        let _ = input_byte_buffer.drain(..Self::BYTE_SIZE);
                        Ok(out)
                    }
                };
            }
            let comment = format!(
                "Returns a checked structure which allows you to read any field for a `{enum_name}` from provided slice.",
            );
            gen.append_impl_fns(&quote! {
                #[doc = #comment]
                pub fn check_slice(buffer: &[u8]) -> Result<#checked_ident, bondrewd::BitfieldLengthError> {
                    let #v_id = Self::#v_id_read_slice_call(&buffer)?;
                    match #v_id {
                        #check_slice_fn
                    }
                }
            });
            let comment = format!(
                "Returns a checked mutable structure which allows you to read/write any field for a `{enum_name}` from provided mut slice.",
            );
            gen.append_impl_fns(&quote! {
                #[doc = #comment]
                pub fn check_slice_mut(buffer: &mut [u8]) -> Result<#checked_ident_mut, bondrewd::BitfieldLengthError> {
                    let #v_id = Self::#v_id_read_slice_call(&buffer)?;
                    match #v_id {
                        #check_slice_mut_fn
                    }
                }
            });
            let lifetime = if lifetime {
                quote! {<'a>}
            } else {
                quote! {}
            };
            gen.append_checked_struct_impl_fns(&quote! {
                pub enum #checked_ident #lifetime {
                    #checked_slice_enum
                }
                pub enum #checked_ident_mut #lifetime {
                    #checked_slice_enum_mut
                }
            });
        }
        // Finish `into_bytes` function.
        into_bytes_fn = quote! {
            fn into_bytes(self) -> [u8;#struct_size] {
                let mut output_byte_buffer = [0u8;#struct_size];
                match self {
                    #into_bytes_fn
                }
                output_byte_buffer
            }
        };
        // Finish Variant Id function.
        let id_ident = id.resolver.ty.get_type_quote()?;
        gen.append_impl_fns(&quote! {
            pub fn id(&self) -> #id_ident {
                match self {
                    #id_fn
                }
            }
        });

        gen.bitfield_trait = quote! {
            #from_bytes_fn
            #into_bytes_fn
        };

        Ok(gen)
        // todo!("finish enum generation.");
    }
    fn gen_variant(
        package: GenVariant,
        struct_size: usize,
        invalid_variant: bool,
    ) -> syn::Result<()> {
        let into_bytes_fn = package.into_bytes_fn;
        let from_bytes_fn = package.from_bytes_fn;
        let gen = package.gen;
        let variant_info = package.variant_info;
        let variant = package.variant;
        let enum_name = package.enum_name;
        let v_id = package.v_id;
        let dyn_fns = package.dyn_fns;
        let id = package.id;
        let id_fn = package.id_fn;
        let check_slice_fn = package.check_slice_fn;
        let checked_ident = package.checked_ident;
        let check_slice_mut_fn = package.check_slice_mut_fn;
        let checked_ident_mut = package.checked_ident_mut;
        let checked_slice_enum = package.checked_slice_enum;
        let checked_slice_enum_mut = package.checked_slice_enum_mut;
        let lifetime = package.lifetime;
        let v_id_write_call = package.v_id_write_call;

        if gen.dyn_fns.is_none() {
            gen.dyn_fns = Some(GeneratedDynFunctions::default());
        }

        // this is the slice indexing that will fool the set function code into thinking
        // it is looking at a smaller array.
        //
        // v_name is the name of the variant.
        let v_name = &variant_info.name;
        // upper_v_name is an Screaming Snake Case of v_name.
        let upper_v_name = v_name.to_string().to_case(Case::UpperSnake);
        // constant names for variant bit and byte sizings.
        let v_byte_const_name = format_ident!("{upper_v_name}_BYTE_SIZE");
        let v_bit_const_name = format_ident!("{upper_v_name}_BIT_SIZE");
        // constant values for variant bit and byte sizings.
        let v_bit_size = variant.total_bits_no_fill() + id.bit_length();
        let v_byte_size = (variant.total_bits() + id.bit_length()).div_ceil(8);
        // TokenStream of v_name.
        let variant_name = quote! {#v_name};

        let thing = variant.gen_struct_fields(
            &v_name,
            Some(GenStructFieldsEnumInfo {
                ident: enum_name,
                full_size: v_byte_size,
            }),
            struct_size,
            dyn_fns,
        )?;
        if let Some(gen_read) = &thing.read_fns.dyn_fns {
            gen.append_checked_struct_impl_fns(&gen_read.checked_struct);
        }
        if let Some(gen_write) = &thing.write_fns.dyn_fns {
            gen.append_checked_struct_impl_fns(&gen_write.checked_struct);
        }
        gen.append_impl_fns(&thing.read_fns.non_trait);
        gen.append_impl_fns(&thing.write_fns.non_trait);
        gen.append_impl_fns(&quote! {
            pub const #v_byte_const_name: usize = #v_byte_size;
            pub const #v_bit_const_name: usize = #v_bit_size;
        });
        // make setter for each field.
        // construct from bytes function. use input_byte_buffer as input name because,
        // that is what the field quotes expect to extract from.
        // wrap our list of field names with commas with Self{} so we it instantiate our struct,
        // because all of the from_bytes field quote store there data in a temporary variable with the same
        // name as its destination field the list of field names will be just fine.

        let variant_id = if invalid_variant {
            quote! {_}
        } else {
            // COPIED_1 Below code is duplicate, look further below to see other copy.
            let id = &variant_info.id;
            if let Ok(yes) = TokenStream::from_str(&format!("{id}")) {
                yes
            } else {
                return Err(syn::Error::new(
                    variant_info.name.span(),
                    "failed to construct id, this is a bug in bondrewd.",
                ));
            }
        };
        let mut variant_value = if let Some(captured_id_field_name) = variant.get_captured_id_name()
        {
            quote! {#captured_id_field_name}
        } else {
            // COPIED_1 Below code is duplicate, look above to see other copy.
            let id = &variant_info.id;
            if let Ok(yes) = TokenStream::from_str(&format!("{id}")) {
                yes
            } else {
                return Err(syn::Error::new(
                    variant_info.name.span(),
                    "failed to construct id, this is a bug in bondrewd.",
                ));
            }
        };
        let variant_constructor = if thing.field_list.is_empty() {
            quote! {Self::#variant_name}
        } else if variant_info.tuple {
            let field_name_list = thing.field_list;
            quote! {Self::#variant_name ( #field_name_list )}
        } else {
            let field_name_list = thing.field_list;
            quote! {Self::#variant_name { #field_name_list }}
        };
        // From Bytes
        let from_bytes_quote = &thing.read_fns.bitfield_trait;
        *from_bytes_fn = quote! {
            #from_bytes_fn
            #variant_id => {
                #from_bytes_quote
                #variant_constructor
            }
        };
        if let (Some(dyn_fns_thing), Some(dyn_fns_gen)) =
            (&thing.read_fns.dyn_fns, &mut gen.dyn_fns)
        {
            let bitfield_dyn_trait_impl_fns = &dyn_fns_gen.bitfield_dyn_trait;
            let from_vec_quote = &dyn_fns_thing.bitfield_dyn_trait;
            dyn_fns_gen.bitfield_dyn_trait = quote! {
                #bitfield_dyn_trait_impl_fns
                #variant_id => {
                    #from_vec_quote
                    #variant_constructor
                }
            };
            // Check Slice
            if let Some(slice_info) = thing.slice_info {
                // do the match statement stuff
                let check_slice_name = &slice_info.func;
                let check_slice_struct = &slice_info.structure;
                *check_slice_fn = quote! {
                    #check_slice_fn
                    #variant_id => {
                        Ok(#checked_ident :: #variant_name (Self::#check_slice_name(buffer)?))
                    }
                };
                let check_slice_name_mut = &slice_info.mut_func;
                let check_slice_struct_mut = &slice_info.mut_structure;
                *check_slice_mut_fn = quote! {
                    #check_slice_mut_fn
                    #variant_id => {
                        Ok(#checked_ident_mut :: #variant_name (Self::#check_slice_name_mut(buffer)?))
                    }
                };

                // do enum stuff
                if !(*lifetime) {
                    *lifetime = true;
                }
                *checked_slice_enum = quote! {
                    #checked_slice_enum
                    #v_name (#check_slice_struct<'a>),
                };
                *checked_slice_enum_mut = quote! {
                    #checked_slice_enum_mut
                    #v_name (#check_slice_struct_mut<'a>),
                };
            } else {
                // do the match statement stuff
                *check_slice_fn = quote! {
                    #check_slice_fn
                    #variant_id => {
                        Ok(#checked_ident :: #variant_name)
                    }
                };
                *check_slice_mut_fn = quote! {
                    #check_slice_mut_fn
                    #variant_id => {
                        Ok(#checked_ident_mut :: #variant_name)
                    }
                };
                // do enum stuff
                *checked_slice_enum = quote! {
                    #checked_slice_enum
                    #v_name,
                };
                *checked_slice_enum_mut = quote! {
                    #checked_slice_enum_mut
                    #v_name,
                };
            }
        }
        // Into Bytes
        let into_bytes_quote = &thing.write_fns.bitfield_trait;
        *into_bytes_fn = quote! {
            #into_bytes_fn
            #variant_constructor => {
                Self::#v_id_write_call(&mut output_byte_buffer, #variant_value);
                #into_bytes_quote
            }
        };

        let mut ignore_fields = if let Some(id_field_name) = variant.get_captured_id_name() {
            variant_value = quote! {*#variant_value};
            quote! { #id_field_name, }
        } else {
            quote! {}
        };
        if variant.fields.is_empty() {
            ignore_fields = quote! { #ignore_fields };
        } else {
            ignore_fields = quote! { #ignore_fields .. };
        };
        if variant_info.tuple {
            ignore_fields = quote! {(#ignore_fields)};
        } else {
            ignore_fields = quote! {{#ignore_fields}};
        }
        *id_fn = quote! {
            #id_fn
            Self::#variant_name #ignore_fields => #variant_value,
        };
        Ok(())
    }
}

struct GenVariant<'a> {
    id: &'a SolvedData,
    gen: &'a mut GeneratedFunctions,
    variant_info: &'a VariantInfo,
    variant: &'a SolvedFieldSet,
    checked_ident: &'a Ident,
    checked_ident_mut: &'a Ident,
    enum_name: &'a Ident,
    v_id: &'a Ident,
    v_id_write_call: &'a Ident,
    check_slice_fn: &'a mut TokenStream,
    check_slice_mut_fn: &'a mut TokenStream,
    checked_slice_enum: &'a mut TokenStream,
    checked_slice_enum_mut: &'a mut TokenStream,
    into_bytes_fn: &'a mut TokenStream,
    from_bytes_fn: &'a mut TokenStream,
    id_fn: &'a mut TokenStream,
    lifetime: &'a mut bool,
    dyn_fns: bool,
}
/// This is going to house all of the information for a Field. This acts as the stage between Builder and
/// Solved, the point being that this can not be created unless a valid `BuilderData` that can be solved is
/// provided. Then we can do all of the calculation because everything has been determined as solvable.
#[derive(Clone, Debug)]
pub struct BuiltData {
    /// The name or ident of the field.
    pub(crate) id: DynamicIdent,
    pub(crate) ty: DataType,
    pub(crate) bit_range: BuiltRange,
    pub(crate) endianness: Endianness,
    /// Describes when the field should be considered.
    pub(crate) reserve: ReserveFieldOption,
    /// How much you care about the field overlapping other fields.
    pub(crate) overlap: OverlapOptions,
    pub(crate) is_captured_id: bool,
}

impl BuiltData {
    pub fn conflict(&self, other: &Self) -> bool {
        if self.reserve.is_fake_field() {
            return false;
        }
        if !self.overlap.enabled() && !other.overlap.enabled() {
            let field_range = self.bit_range.range();
            let other_range = other.bit_range.range();
            // check that self's start is not within other's range
            if field_range.start >= other_range.start
                && (field_range.start == other_range.start || field_range.start < other_range.end)
            {
                return true;
            }
            // check that other's start is not within self's range
            if other_range.start >= field_range.start
                && (other_range.start == field_range.start || other_range.start < field_range.end)
            {
                return true;
            }
            if other_range.end > field_range.start && other_range.end <= field_range.end {
                return true;
            }
            if field_range.end > other_range.start && field_range.end <= other_range.end {
                return true;
            }
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct BuiltRange {
    /// This is the full bit range of the field. The `ty` does not effect this values meaning, `bit_range`
    /// shall contain the entire bit range for a field, array elements should each calculate their own
    /// range within this range.
    pub(crate) bit_range: Range<usize>,
    /// The range type determines if the `bit_range` contains a single value or contains a array of values.
    pub(crate) ty: BuiltRangeType,
}

impl BuiltRange {
    #[must_use]
    pub fn range(&self) -> &Range<usize> {
        &self.bit_range
    }
    #[must_use]
    pub fn bit_length(&self) -> usize {
        self.bit_range.end - self.bit_range.start
    }
}

#[derive(Clone, Debug)]
pub enum BuiltRangeType {
    SingleElement,
    BlockArray(ArraySizings),
    ElementArray(ArraySizings),
}

impl BuiltRange {
    fn end(&self) -> usize {
        self.bit_range.end
    }
    fn from_builder(
        builder: &DataBuilderRange,
        data_ty: &FullDataType,
        last_field_end: Option<usize>,
    ) -> Self {
        let start = if let Some(prev) = &last_field_end {
            *prev
        } else {
            0
        };
        let ty = if let Some(ref a_spec) = data_ty.array_spec {
            match a_spec.ty {
                FullDataTypeArraySpecType::NotSpecified | FullDataTypeArraySpecType::Element => {
                    BuiltRangeType::ElementArray(a_spec.sizings.clone())
                }
                FullDataTypeArraySpecType::Block => {
                    BuiltRangeType::BlockArray(a_spec.sizings.clone())
                }
            }
        } else {
            BuiltRangeType::SingleElement
        };
        match builder {
            DataBuilderRange::Range(bit_range) => Self {
                bit_range: bit_range.clone(),
                ty,
            },
            DataBuilderRange::Size(bit_length) => {
                let bit_range = start..(start + *bit_length as usize);
                Self { bit_range, ty }
            }
            DataBuilderRange::None => {
                let bit_range = start..(start + data_ty.data_type.default_bit_size());
                Self { bit_range, ty }
            } // BuilderRange::ElementArray { sizings, size } => {
              //     let bit_range = match &size {
              //         BuilderRangeArraySize::Size(element_bit_length) => {
              //             let mut total_bits = *element_bit_length as usize;
              //             for size in sizings {
              //                 total_bits *= size;
              //             }
              //             start..(start + total_bits)
              //         }
              //         BuilderRangeArraySize::Range(range) => range.clone(),
              //     };

              //     Self {
              //         bit_range,
              //         ty: BuiltRangeType::ElementArray(sizings.clone()),
              //     }
              // }
              // BuilderRange::BlockArray { sizings, size } => {
              //     let bit_range = match &size {
              //         BuilderRangeArraySize::Size(total_bits) => {
              //             start..(start + *total_bits as usize)
              //         }
              //         BuilderRangeArraySize::Range(range) => range.clone(),
              //     };
              //     Self {
              //         bit_range,
              //         ty: BuiltRangeType::BlockArray(sizings.clone()),
              //     }
              // }
        }
    }
}
