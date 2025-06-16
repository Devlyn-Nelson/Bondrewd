use std::{collections::BTreeMap, ops::Range};

use proc_macro2::Span;
use quote::format_ident;
use syn::Ident;

use crate::build::{
    field::{
        DataBuilderRange, DataType, FullDataType, FullDataTypeArraySpecType, NumberType,
        RustByteSize,
    },
    field_set::{
        EnumBuilder, FieldSetBuilder, FillBits, GenericBuilder, StructBuilder, StructEnforcementTy,
        VariantBuilder,
    },
    ArraySizings, Endianness, OverlapOptions, ReserveFieldOption, Visibility,
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

impl TryFrom<GenericBuilder> for Solved {
    type Error = syn::Error;

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

fn check_for_id(variant: &VariantBuilder, used_ids: &mut Vec<usize>) -> Result<(), syn::Error> {
    if let Some(value) = variant.id {
        if used_ids.contains(&value) {
            let vname = &variant.field_set.name;
            return Err(syn::Error::new(
                vname.span(),
                "Variant name used twice. Variants must have unique names.",
            ));
        }
        used_ids.push(value);
    }
    Ok(())
}

fn get_built_variant(
    variant: VariantBuilder,
    used_ids: &mut Vec<usize>,
    next: &mut usize,
    largest_variant_id: &mut usize,
) -> VariantBuilt {
    let id = if let Some(value) = variant.id {
        value
    } else {
        let mut guess = *next;
        while used_ids.contains(&guess) {
            guess += 1;
        }
        used_ids.push(guess);
        guess
    };
    *next = id + 1;
    if *largest_variant_id < id {
        *largest_variant_id = id;
    }
    VariantBuilt {
        id,
        field_set: variant.field_set,
        tuple: variant.tuple,
    }
}

impl TryFrom<EnumBuilder> for Solved {
    type Error = syn::Error;

    fn try_from(value: EnumBuilder) -> Result<Self, Self::Error> {
        let variants = value.variants;
        // give all variants ids.
        // TODO this code was in the parsing code, but has been moved here.
        let mut used_ids: Vec<usize> = Vec::default();
        let mut last = 0;
        let mut built_variants = Vec::<VariantBuilt>::with_capacity(variants.len());
        let mut largest_variant_id = 0;
        let mut largest_bit_size = 0;
        // go through variants to get id's that are specified.
        for variant in &variants {
            check_for_id(variant, &mut used_ids)?;
        }
        check_for_id(&value.invalid, &mut used_ids)?;
        // go through variants again, assigning id's to the ones that don't have one,
        // and convert them to BuiltVariants (between Builder and Solved)
        for variant in variants {
            let built =
                get_built_variant(variant, &mut used_ids, &mut last, &mut largest_variant_id);
            built_variants.push(built);
        }
        let built_invalid = get_built_variant(
            value.invalid,
            &mut used_ids,
            &mut last,
            &mut largest_variant_id,
        );
        // determine id field.
        let (id_field_type, id_bits) = {
            let id_bits = if let Some(id_bits) = value.id_bit_length {
                id_bits
            } else if let (Some(payload_size), StructEnforcementTy::EnforceBitAmount(total_size)) =
                (&value.payload_bit_length, &value.attrs.enforcement.ty)
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
                invalid => return Err(syn::Error::new(Span::call_site(), format!("The variant id must have a type of: u8, u16, u32, u64, or u128, variant bit length is currently {invalid} and bondrewd doesn't know which type use."))),
            };
            (DataType::Number(NumberType::Unsigned, bytes), id_bits)
        };
        // validity checks
        if bits_needed(largest_variant_id) > id_bits {
            let mut span = None;
            for v in built_variants {
                if v.id == largest_variant_id {
                    span = Some(v.field_set.name.span());
                    break;
                }
            }
            let span = if let Some(s) = span {
                s
            } else {
                Span::call_site()
            };
            return Err(syn::Error::new(span, format!("Largest variant id value ({largest_variant_id}) is larger than `id_bit_size` ({id_bits})")));
        }

        // TODO try to enhance id field detection, use any hints given such as `capture_id` fields.
        let id_field = BuiltData {
            id: format_ident!("{}", EnumBuilder::VARIANT_ID_NAME).into(),
            ty: id_field_type,
            bit_range: BuiltRange {
                bit_range: 0..id_bits,
                ty: BuiltRangeType::SingleElement,
            },
            endianness: value.attrs.endianness,
            reserve: ReserveFieldOption::FakeField,
            overlap: OverlapOptions::None,
            is_captured_id: false,
        };

        let mut solved_variants = BTreeMap::default();
        let (invalid_name, invalid, invalid_fill) = Self::solve_variant(
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
        let bit_size = largest_bit_size + id_bits;
        // after solving the attrs for fill might be set. need to do it here
        // because the largest_payload_size can't be determined until `solve_variant`
        // has been called on all variants.
        // Self::maybe_add_fill_field(&invalid_fill, &mut invalid, true, Some(id_bits), bit_size)?;
        // for (info, (set, fill)) in &mut solved_variants {
        //     Self::maybe_add_fill_field(fill, set, true, Some(id_bits), bit_size)?;
        // }
        match &value.attrs.enforcement.ty {
            StructEnforcementTy::NoRules => {}
            StructEnforcementTy::EnforceFullBytes => {
                if bit_size % 8 != 0 {
                    return Err(syn::Error::new(
                        value.attrs.enforcement.span,
                        "Final bit count was not evenly divisible by 0.",
                    ));
                }
            }
            StructEnforcementTy::EnforceBitAmount(expected_total_bits) => {
                if bit_size != *expected_total_bits {
                    let mut span = None;
                    let expected = bit_size - id_bits;
                    for (key, v) in &solved_variants {
                        // TODO test to see that the struct enforcement error are nice and accurate.
                        if v.0.total_bits_no_fill() >= expected {
                            span = Some(key.name.span());
                            break;
                        }
                    }
                    let (span, message) = if let Some(s) = span {
                        (
                            s,
                            format!("Final bit count does not match enforcement size. [user = {expected_total_bits}, actual = {bit_size}]"),
                        )
                    } else {
                        (
                            value.attrs.enforcement.span,
                            format!("Final bit count of a variant does not match enforcement size. [user = {expected_total_bits}, actual = {bit_size}]")
                        )
                    };
                    return Err(syn::Error::new(span, message));
                }
            }
        }
        let id = SolvedData::from_built(id_field, bit_size);
        Ok(Solved {
            name: value.name,
            ty: SolvedType::Enum {
                id,
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

// TODO determine if this is needed. also fix Span of error if it is needed.
fn detect_variant_fill(
    variant_payload_length: usize,
    largest_bit_size: usize,
    id_bits: usize,
    fill_bits: &FillBits,
) -> Result<FillBits, syn::Error> {
    if largest_bit_size < variant_payload_length {
        return Err(syn::Error::new(Span::call_site(), format!("Largest variant payload ({largest_bit_size}) is larger than `payload_bit_size` ({variant_payload_length})")));
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
    type Error = syn::Error;

    fn try_from(value: StructBuilder) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&StructBuilder> for Solved {
    type Error = syn::Error;

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
    ) -> Result<(VariantInfo, SolvedFieldSet, FillBits), syn::Error> {
        let solved_variant =
            Self::try_from_field_set(&variant.field_set, solved_attrs, Some(id_field))?;

        let bit_length = solved_variant.total_bits_no_fill();
        if *largest_bit_size < bit_length {
            *largest_bit_size = bit_length;
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
                // check invalid variant
                let other = invalid.total_bits_no_fill();
                if other > largest {
                    largest = other;
                }
                // check variants
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
    pub fn dump(&self) -> bool {
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
    ) -> Result<SolvedFieldSet, syn::Error> {
        // TODO verify captured id fields match the generated id field using below commented code.
        let mut pre_fields: Vec<BuiltData> = Vec::default();
        let mut last_end_bit_index: Option<usize> = id_field.map(|f| f.bit_range.bit_length());
        let total_fields = value.fields.len();
        let fields_ref = &value.fields;
        let mut total_bit_size = if let Some(id) = id_field {
            id.bit_range.bit_length()
        } else {
            0
        };
        // First stage checks for validity
        for value_field in fields_ref {
            // get resolved range for the field.
            let bit_range = if value_field.is_captured_id {
                if let Some(id) = id_field {
                    id.bit_range.clone()
                } else {
                    return Err(syn::Error::new(
                        value_field.id.span(),
                        "Field was marked as `capture_id`, but is not in an enum variant",
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
                endianness: value.attrs.endianness.clone(),
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
                    return Err(syn::Error::new(field.id.span(), format!("Field overlaps with `{}` (you can mark this as `redundant` if they read from the same bits)", other.id.name())));
                }
            }
            if !value_field.is_captured_id {
                total_bit_size += field.bit_range.bit_length();
            }
            pre_fields.push(field);
        }
        let mut total_bits_without_id = total_bit_size;
        if let Some(id_field) = id_field {
            total_bits_without_id -= id_field.bit_range.bit_length();
        }
        // verify and add fill
        match &value.attrs.enforcement.ty {
            StructEnforcementTy::NoRules => {}
            StructEnforcementTy::EnforceFullBytes => {
                if total_bits_without_id % 8 != 0 {
                    return Err(syn::Error::new(
                        value.attrs.enforcement.span,
                        "Final bit count was not evenly divisible by 0.",
                    ));
                }
            }
            StructEnforcementTy::EnforceBitAmount(expected_total_bits) => {
                if total_bits_without_id != *expected_total_bits {
                    let message = if id_field.is_some() {
                        format!("Variant `{}` final bit count does not match enforcement size.[user = {expected_total_bits}, actual = {total_bits_without_id}]", value.name)
                    } else {
                        format!("Final bit count does not match enforcement size.[user = {expected_total_bits}, actual = {total_bits_without_id}]")
                    };
                    return Err(syn::Error::new(value.attrs.enforcement.span, message));
                }
            }
        }
        // add reserve for fill bytes. this happens after bit enforcement because bit_enforcement is for checking user code.
        let maybe_fill = Self::maybe_add_fill_field(
            &value.fill_bits,
            &mut pre_fields,
            id_field.is_some(),
            None,
            &mut total_bit_size,
        );
        // finalize
        let mut fields: Vec<SolvedData> = Vec::default();
        let flip_bits = total_bit_size;
        for pre_field in pre_fields {
            if let Some(field) = id_field {
                if field.conflict(&pre_field) {
                    return Err(syn::Error::new(pre_field.id.span(), format!("Field overlaps with `{}` (you can mark this as `redundant` if they read from the same bits)", field.id.name())));
                }
            }
            fields.push(SolvedData::from_built(pre_field, flip_bits));
        }
        if let Some(fill) = maybe_fill {
            fields.push(fill);
        }
        let out = SolvedFieldSet {
            fields,
            attrs: attrs.clone(),
        };
        Ok(out)
    }
    fn maybe_add_fill_field(
        fill: &FillBits,
        fields: &mut Vec<BuiltData>,
        has_id_field: bool,
        id_bit_size: Option<usize>,
        total_bits: &mut usize,
    ) -> Option<SolvedData> {
        let auto_fill = match fill {
            FillBits::None => None,
            FillBits::Bits(bits) => Some(*bits),
            FillBits::Auto => {
                let unused_bits = (8 - (*total_bits % 8)) % 8;
                if unused_bits == 0 {
                    None
                } else {
                    Some(unused_bits)
                    // None
                }
            }
        };
        let thing = if let Some(fill_bits) = auto_fill {
            *total_bits += fill_bits;
            let first_bit = if let Some(last_range) = fields.iter().last() {
                last_range.bit_range.end()
            } else {
                0_usize
            };
            let end_bit = first_bit + fill_bits;
            let fill_bytes_size = (end_bit - first_bit).div_ceil(8);
            let ident = quote::format_ident!("bondrewd_fill_bits");
            let fill_field = BuiltData {
                id: ident.into(),
                ty: DataType::Number(NumberType::Unsigned, RustByteSize::One),
                bit_range: BuiltRange {
                    bit_range: first_bit..end_bit,
                    ty: BuiltRangeType::BlockArray(vec![fill_bytes_size]),
                },
                endianness: Endianness::default(),
                // START_HERE this should be a reserve field so it gets accounted for.
                reserve: ReserveFieldOption::FakeField,
                overlap: OverlapOptions::None,
                is_captured_id: false,
            };
            Some(SolvedData::from_built(fill_field, *total_bits))
        } else {
            None
        };
        thing
    }
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
                let bit_range = start..(start + { *bit_length });
                Self { bit_range, ty }
            }
            DataBuilderRange::None => {
                let mut ty_size = data_ty.data_type.default_bit_size();
                if let BuiltRangeType::ElementArray(items) = &ty {
                    for i in items {
                        ty_size *= i;
                    }
                }
                let bit_range = start..(start + ty_size);
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
