use std::collections::{BTreeMap, HashMap};

use crate::build::field_set::{EnumBuilder, FieldSetBuilder, GenericBuilder};

use super::field::SolvedData;

pub struct Solved<FieldSetId, DataId> {
    /// DataSet's name.
    ///
    /// for derive this would be the Enum or Struct ident.
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

impl<FieldSetId, DataId> From<GenericBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId> {
    fn from(value: GenericBuilder<FieldSetId, DataId>) -> Self {
        match value.ty {
            crate::build::field_set::BuilderType::Enum(e) => e.into(),
            crate::build::field_set::BuilderType::Struct(s) => s.into(),
        }
    }
}

impl<FieldSetId, DataId> From<EnumBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId> {
    fn from(value: EnumBuilder<FieldSetId, DataId>) -> Self {
        //
        todo!()
    }
}

impl<FieldSetId, DataId> From<FieldSetBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId> {
    fn from(value: FieldSetBuilder<FieldSetId, DataId>) -> Self {
        //
        todo!()
    }
}
