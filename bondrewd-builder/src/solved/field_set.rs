use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    hash::Hash,
};

use thiserror::Error;

use crate::build::field_set::{EnumBuilder, FieldSetBuilder, GenericBuilder};

use super::field::SolvedData;

pub struct Solved<FieldSetId, DataId>
where
    FieldSetId: Display + Clone + Copy,
{
    /// DataSet's name.
    ///
    /// for derive this would be the Enum or Struct ident.
    #[cfg(feature = "derive")]
    name: FieldSetId,
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
}

impl<FieldSetId, DataId> TryFrom<GenericBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId>
where
    FieldSetId: Display + Clone + Copy,
    DataId: Hash + PartialEq + Eq + Display + Clone + Copy,
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
    FieldSetId: Display + Clone + Copy,
    DataId: Hash + PartialEq + Eq + Display + Clone + Copy,
{
    type Error = SolvingError;

    fn try_from(value: EnumBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        todo!("write conversion from EnumBuilder to Solved")
    }
}

impl<FieldSetId, DataId> TryFrom<FieldSetBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId>
where
    FieldSetId: Display + Clone + Copy,
    DataId: Hash + PartialEq + Eq + Display + Clone + Copy,
{
    type Error = SolvingError;

    fn try_from(value: FieldSetBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        Self::try_from_field_set(&value, None)
    }
}

impl<FieldSetId, DataId> TryFrom<&FieldSetBuilder<FieldSetId, DataId>>
    for Solved<FieldSetId, DataId>
where
    FieldSetId: Display + Clone + Copy,
    DataId: Hash + PartialEq + Eq + Display + Clone + Copy,
{
    type Error = SolvingError;

    fn try_from(value: &FieldSetBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        Self::try_from_field_set(value, None)
    }
}

impl<FieldSetId, DataId> Solved<FieldSetId, DataId>
where
    FieldSetId: Display + Clone + Copy,
    DataId: Hash + PartialEq + Eq + Display + Clone + Copy,
{
    fn try_from_field_set(
        value: &FieldSetBuilder<FieldSetId, DataId>,
        id_field: Option<&SolvedData>,
    ) -> Result<Self, SolvingError> {
        let bit_size = if let Some(id_field) = id_field {
            id_field.bit_length()
        } else {
            0
        };
        let fields: HashMap<DataId, SolvedData> = HashMap::default();
        let last_end_bit_index: Option<usize> = None;
        for field in &value.fields {
            let name = format!("{}", field.id);
            let resolver = todo!("write resolvers");
            let new_field = SolvedData { resolver };
            fields.insert(field.id, new_field);
        }
        todo!("solve for flip (reverse byte order)");
        todo!("solve for field order reversal, might do it in loop after `last_end_bit_index` is set.");
        todo!("overlap protection for fields");
        todo!("handle array solving");
        todo!("enforcements.");
        Ok(Self {
            #[cfg(feature = "derive")]
            name: value.name,
            ty: SolvedType::Struct(SolvedFieldSet { fields }),
        })
    }
}
