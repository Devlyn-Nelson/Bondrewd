use super::field::DataBuilder;

pub struct Builder {}
enum BuilderType {
    Enum {
        name: String,
        id: DataBuilder,
        invalid: Option<VariantBuilder>,
        variants: Vec<VariantBuilder>,
    },
    Struct(BuilderFieldSet),
}
struct VariantBuilder {
    id: Option<i64>,
    capture_field: Option<DataBuilder>,
    field_set: BuilderFieldSet,
}
struct BuilderFieldSet {
    name: String,
    fields: Vec<DataBuilder>,
}
