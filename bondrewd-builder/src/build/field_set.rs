use super::field::DataBuilder;

pub struct Builder {}
enum BuilderType {
    Enum {
        name: String,
        id: DataBuilder,
        variants: Vec<BuilderFieldSet>,
    },
    Struct(BuilderFieldSet),
}
struct BuilderFieldSet {
    name: String,
    fields: Vec<DataBuilder>,
}
