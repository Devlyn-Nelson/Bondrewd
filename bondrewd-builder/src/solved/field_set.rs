use std::{
    collections::{BTreeMap, HashMap},
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
}

impl<FieldSetId, DataId> TryFrom<GenericBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId>
where
DataId: Hash + PartialEq + Eq, {
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
DataId: Hash + PartialEq + Eq, {
    type Error = SolvingError;

    fn try_from(value: EnumBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        todo!("write conversion from EnumBuilder to Solved")
    }
}

impl<FieldSetId, DataId> TryFrom<FieldSetBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId>
where
DataId: Hash + PartialEq + Eq, {
    type Error = SolvingError;

    fn try_from(value: FieldSetBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        Self::try_from_field_set(value, None)
    }
}

impl<FieldSetId, DataId> Solved<FieldSetId, DataId>
where
    DataId: Hash + PartialEq + Eq,
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
        let mut last_field: Option<DataId> = None;
        for field in value.fields {
            let id = field.id;

            // TODO overlap protection
            let new_field = SolvedData {
                resolver: todo!("write resolver solving logic"),
            };
            fields.insert(id, new_field);
            last_field = Some(id);
        }
        todo!("write conversion from Builder to Solved");
        Ok(Self {
            ty: SolvedType::Struct(SolvedFieldSet { fields }),
        })
    }
}
