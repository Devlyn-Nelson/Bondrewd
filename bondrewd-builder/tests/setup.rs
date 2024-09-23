use bondrewd_builder::build::{
    field::{DataBuilder, DataType, NumberType},
    field_set::GenericBuilder,
};

#[test]
fn derive_struct_setup() {
    let field_one = DataBuilder::new("one", DataType::Number(NumberType::Float));
    // This is a round about way of doing structs and is not recommended.
    let mut builder: GenericBuilder<&str, &str> = GenericBuilder::single_set("test");
    let inner_builder = builder.get_mut().get_mut_struct().unwrap();
    inner_builder.add_field(field_one);

    panic!("{inner_builder:?}");
}
