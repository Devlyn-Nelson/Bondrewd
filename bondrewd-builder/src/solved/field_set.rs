use std::collections::BTreeMap;

use super::field::SolvedData;

pub struct Solved<Id> {
    /// DataSet's name.
    ///
    /// for derive this would be the Enum or Struct ident.
    name: String,
    ty: SolvedType<Id>,
}
enum SolvedType<Id> {
    Enum {
        /// The id field. or the field that determines the variant.
        id: SolvedData,
        /// The default variant. in the case not other variant matches, this will be used.
        invalid: SolvedFieldSet,
        /// The default variant's name/ident
        invalid_name: VariantInfo<Id>,
        /// Sets of fields, each representing a variant of an enum. the String
        /// being the name of the variant
        variants: BTreeMap<VariantInfo<Id>, SolvedFieldSet>,
    },
    Struct(SolvedFieldSet),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VariantInfo<Id> {
    id: i64,
    name: Id,
}

struct SolvedFieldSet {
    fields: Vec<SolvedData>,
}
