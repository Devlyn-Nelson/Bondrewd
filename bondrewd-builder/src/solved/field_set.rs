use std::{collections::BTreeMap, ops::Range};

use proc_macro2::TokenStream;
use syn::Ident;
use thiserror::Error;

use crate::build::{
    field::DataType,
    field_set::{EnumBuilder, FieldSetBuilder, GenericBuilder, StructEnforcement},
    ArraySizings, BuilderRange, Endianness, OverlapOptions, ReserveFieldOption,
};

use super::field::{DynamicIdent, SolvedData};

pub struct Solved {
    /// `DataSet`'s name.
    ///
    /// for derive this would be the Enum or Struct ident.
    name: Ident,
    ty: SolvedType,
}
enum SolvedType {
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
    },
    Struct(SolvedFieldSet),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VariantInfo {
    id: i64,
    name: Ident,
}

struct SolvedFieldSet {
    fields: Vec<SolvedData>,
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

impl TryFrom<EnumBuilder> for Solved {
    type Error = SolvingError;

    fn try_from(value: EnumBuilder) -> Result<Self, Self::Error> {
        let id = value.id;
        let variants = value.variants;
        let invalid = value.invalid;
        let name = value.name;
        todo!("write conversion from EnumBuilder to Solved")
    }
}

impl TryFrom<FieldSetBuilder> for Solved {
    type Error = SolvingError;

    fn try_from(value: FieldSetBuilder) -> Result<Self, Self::Error> {
        Self::try_from_field_set(&value, None)
    }
}

impl TryFrom<&FieldSetBuilder> for Solved {
    type Error = SolvingError;

    fn try_from(value: &FieldSetBuilder) -> Result<Self, Self::Error> {
        Self::try_from_field_set(value, None)
    }
}

impl Solved {
    // TODO START_HERE
    pub fn gen(&self) -> TokenStream {
        match &self.ty {
            SolvedType::Enum { id, invalid, invalid_name, variants } => Self::gen_enum(id, invalid, invalid_name, variants),
            SolvedType::Struct(solved_field_set) => Self::gen_struct(solved_field_set),
        }
    }
    fn gen_enum(
        id: &SolvedData,
        invalid: &SolvedFieldSet,
        invalid_name: &VariantInfo,
        variants: &BTreeMap<VariantInfo, SolvedFieldSet>,
    ) -> TokenStream {
        for (info, variant) in variants {
            
        }
        todo!("generate enum code.");
    }
    fn gen_struct(field_set: &SolvedFieldSet) -> TokenStream {
        todo!("generate struct code.")
    }
    fn try_from_field_set(
        value: &FieldSetBuilder,
        id_field: Option<&SolvedData>,
    ) -> Result<Self, SolvingError> {
        let bit_size = if let Some(id_field) = id_field {
            id_field.bit_length()
        } else {
            0
        };
        let mut pre_fields: Vec<BuiltData> = Vec::default();
        let mut last_end_bit_index: Option<usize> = None;
        let total_fields = value.fields.len();
        let fields_ref = &value.fields;
        // First stage checks for validity
        for value_field in fields_ref {
            let rust_size = value_field.ty.rust_size();
            // get resolved range for the field.
            let bit_range =
                BuiltRange::from_builder(&value_field.bit_range, rust_size, last_end_bit_index); // get_range(&value_field.bit_range, &rust_size, last_end_bit_index);
                                                                                                 // update internal last_end_bit_index to allow automatic bit-range feature to work.
            if !value_field.overlap.is_redundant() {
                last_end_bit_index = Some(bit_range.end());
            }
            let ty = value_field.ty.clone();
            let field = BuiltData {
                ty,
                bit_range,
                endianness: if let Some(e) = &value_field.endianness {
                    e.clone()
                } else {
                    // TODO no endianess is actually valid in the case of nested structs/enums.
                    // We need to check if the value is a primitive number, then if it is a number and does
                    // not have endianess we can throw this error.
                    return Err(SolvingError::NoEndianness(format!(
                        "{}",
                        value_field.id.ident()
                    )));
                },
                id: value_field.id.clone(),
                reserve: value_field.reserve.clone(),
                overlap: value_field.overlap.clone(),
            };
            let field_range = field.bit_range.range();
            for other in &pre_fields {
                if !field.overlap.enabled() && !other.overlap.enabled() {
                    let other_range = other.bit_range.range();
                    // check that self's start is not within other's range
                    if field_range.start >= other_range.start
                        && (field_range.start == other_range.start
                            || field_range.start < other_range.end)
                    {
                        return Err(SolvingError::Overlap);
                    }
                    // check that other's start is not within self's range
                    if other_range.start >= field_range.start
                        && (other_range.start == field_range.start
                            || other_range.start < field_range.end)
                    {
                        return Err(SolvingError::Overlap);
                    }
                    if other_range.end > field_range.start && other_range.end <= field_range.end {
                        return Err(SolvingError::Overlap);
                    }
                    if field_range.end > other_range.start && field_range.end <= other_range.end {
                        return Err(SolvingError::Overlap);
                    }
                }
            }
            // let name = format!("{}", field.id);
            pre_fields.push(field);
        }
        let mut fields: Vec<SolvedData> = Vec::default();
        for pre_field in pre_fields {
            fields.push(SolvedData::from(pre_field));
        }
        // let keys: Vec<DynamicIdent> = fields.keys().cloned().collect();
        // for key in keys {
        //     let field = fields.get(&key);
        // }
        match value.enforcement {
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
        //TODO check and uphold enforcements.
        Ok(Self {
            name: value.name.clone(),
            ty: SolvedType::Struct(SolvedFieldSet { fields }),
        })
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
    pub fn bit_size(&self) -> usize {
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
        builder: &BuilderRange,
        rust_size: usize,
        last_field_end: Option<usize>,
    ) -> Self {
        let start = if let Some(prev) = &last_field_end {
            *prev
        } else {
            0
        };
        match builder {
            BuilderRange::Range(bit_range) => Self {
                bit_range: bit_range.clone(),
                ty: BuiltRangeType::SingleElement,
            },
            BuilderRange::Size(bit_length) => {
                let bit_range = start..(start + *bit_length as usize);
                Self {
                    bit_range,
                    ty: BuiltRangeType::SingleElement,
                }
            }
            BuilderRange::None => {
                let bit_range = start..(start + (rust_size * 8));
                Self {
                    bit_range,
                    ty: BuiltRangeType::SingleElement,
                }
            }
            BuilderRange::ElementArray {
                sizings,
                element_bit_length,
            } => {
                let mut total_bits = *element_bit_length as usize;
                for size in sizings {
                    total_bits *= size;
                }
                let bit_range = start..(start + total_bits);
                Self {
                    bit_range,
                    ty: BuiltRangeType::ElementArray(sizings.clone()),
                }
            }
            BuilderRange::BlockArray {
                sizings,
                total_bits,
            } => {
                let bit_range = start..(start + *total_bits as usize);
                Self {
                    bit_range,
                    ty: BuiltRangeType::BlockArray(sizings.clone()),
                }
            }
        }
    }
}
