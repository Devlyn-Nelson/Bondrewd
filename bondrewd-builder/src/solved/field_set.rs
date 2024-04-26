use std::collections::BTreeMap;

use super::field::SolvedData;

pub struct Solved {
    /// DataSet's name.
    ///
    /// for derive this would be the Enum or Struct ident.
    name: String,
    ty: SolvedType,
}
enum SolvedType {
    Enum {
        /// The id field. or the field that determines the variant.
        id: SolvedData,
        /// Sets of fields, each representing a variant of an enum. the String
        /// being the name of the variant
        variants: BTreeMap<String, Vec<SolvedData>>,
    },
    Struct(Vec<SolvedData>),
}
