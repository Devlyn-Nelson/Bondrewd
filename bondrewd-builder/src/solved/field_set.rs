use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    hash::Hash,
};

use thiserror::Error;

use crate::{
    build::field_set::{EnumBuilder, FieldSetBuilder, GenericBuilder},
    solved::field::Resolver,
};

use super::field::SolvedData;

pub struct Solved<FieldSetId, DataId> {
    /// DataSet's name.
    ///
    /// for derive this would be the Enum or Struct ident.
    #[cfg(feature = "derive")]
    name: String,
    ty: SolvedType<FieldSetId, DataId>,
}
enum SolvedType<FieldSetId, DataId> {
    Enum {
        /// The id field. or the field that determines the variant.
        id: SolvedData,
        /// The default variant. in the case not other variant matches, this will be used.
        invalid: SolvedFieldSet<DataId>,
        /// The default variant's name/ident
        invalid_name: VariantInfo<FieldSetId>,
        /// Sets of fields, each representing a variant of an enum. the String
        /// being the name of the variant
        variants: BTreeMap<VariantInfo<FieldSetId>, SolvedFieldSet<DataId>>,
    },
    Struct(SolvedFieldSet<DataId>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VariantInfo<FieldSetId> {
    id: i64,
    name: FieldSetId,
}

struct SolvedFieldSet<DataId> {
    fields: HashMap<DataId, SolvedData>,
}

#[derive(Debug, Error)]
pub enum SolvingError {
    #[error("Fields overlap")]
    Overlap,
    #[error("No data type was provided for field with id {0}")]
    NoTypeProvided(String),
    #[error("No endianness was provided for field with id {0}")]
    NoEndianness(String),
    #[error("Caught panic: [{0}]")]
    WouldPanic(String),
}

impl<FieldSetId, DataId> TryFrom<GenericBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId>
where
    DataId: Hash + PartialEq + Eq + Display,
{
    type Error = SolvingError;

    fn try_from(value: GenericBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        match value.ty {
            crate::build::field_set::BuilderType::Enum(e) => e.try_into(),
            crate::build::field_set::BuilderType::Struct(s) => s.try_into(),
        }
    }
}

impl<FieldSetId, DataId> TryFrom<EnumBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId>
where
    DataId: Hash + PartialEq + Eq + Display,
{
    type Error = SolvingError;

    fn try_from(value: EnumBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        todo!("write conversion from EnumBuilder to Solved")
    }
}

impl<FieldSetId, DataId> TryFrom<FieldSetBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId>
where
    DataId: Hash + PartialEq + Eq + Display,
{
    type Error = SolvingError;

    fn try_from(value: FieldSetBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        Self::try_from_field_set(value, None)
    }
}

impl<FieldSetId, DataId> Solved<FieldSetId, DataId>
where
    DataId: Hash + PartialEq + Eq + Display,
{
    fn try_from_field_set(
        value: FieldSetBuilder<FieldSetId, DataId>,
        id_field: Option<&SolvedData>,
    ) -> Result<Self, SolvingError> {
        let mut bit_size = if let Some(id_field) = id_field {
            id_field.bit_length()
        } else {
            0
        };
        let mut fields: HashMap<DataId, SolvedData> = HashMap::default();
        let mut last_end_bit_index: Option<usize> = None;
        for field in value.fields {
            let id = field.id;
            let bit_range = match field.bit_range {
                crate::build::BuilderRange::Range(range) => range,
                crate::build::BuilderRange::Size(bit_length) => {
                    let start = if let Some(prev) = &last_end_bit_index {
                        *prev
                    } else {
                        0
                    };
                    start..(start + bit_length as usize)
                }
                crate::build::BuilderRange::None => {
                    let start = if let Some(prev) = &last_end_bit_index {
                        *prev
                    } else {
                        0
                    };
                    start..(start + (field.rust_size as usize * 8))
                }
            };
            if !field.overlap.is_redundant() {
                last_end_bit_index = Some(bit_range.end);
            }
            let bit_length = bit_range.end - bit_range.start;
            let spans_multiple_bytes = (bit_range.start / 8) != (bit_range.end / 8);
            let name = format!("{id}");
            let resolver = match field.ty {
                crate::build::field::DataType::None => {
                    return Err(SolvingError::NoTypeProvided(format!("{id}")))
                }
                crate::build::field::DataType::Number(ty, endianess) => match endianess {
                    Some(e) => match e.mode() {
                        crate::build::EndiannessMode::Alternative => {
                            if spans_multiple_bytes {
                                Resolver::multi_alt(&bit_range, name.as_str(), ty)
                            } else {
                                Resolver::single_alt(&bit_range, name.as_str(), ty)
                            }
                        }
                        crate::build::EndiannessMode::Standard => {
                            if spans_multiple_bytes {
                                Resolver::multi_standard(&bit_range, name.as_str(), ty)
                            } else {
                                Resolver::single_standard(&bit_range, name.as_str(), ty)
                            }
                        }
                    },
                    None => return Err(SolvingError::NoEndianness(format!("{id}"))),
                },
                #[cfg(feature = "derive")]
                crate::build::field::DataType::Nested(struct_name) => {
                    if spans_multiple_bytes {
                        Resolver::multi_nested(&bit_range, name.as_str(), struct_name)
                    } else {
                        Resolver::single_nested(&bit_range, name.as_str(), struct_name)
                    }
                }
            };
            let new_field = SolvedData {
                resolver: todo!("write resolver solving logic"),
            };
            fields.insert(id, new_field);
        }
        // TODO solve for flip, might do it in loop after `last_end_bit_index` is set.
        // TODO overlap protection for fields
        // TODO handle array solving
        // TODO enforcements.
        Ok(Self {
            #[cfg(feature = "derive")]
            name: todo!("fill Solved name for derive"),
            ty: SolvedType::Struct(SolvedFieldSet { fields }),
        })
    }
}
