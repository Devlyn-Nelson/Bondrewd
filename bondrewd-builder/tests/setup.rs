use bondrewd_builder::{
    build::{
        field::{DataBuilder, DataType, NumberType},
        field_set::GenericBuilder, Endianness,
    },
    solved::field_set::Solved,
};
use quote::format_ident;

#[test]
fn derive_struct_setup() {
    let field_one = DataBuilder::new(
        format_ident!("one").into(),
        DataType::Number(
            NumberType::Float,
            bondrewd_builder::build::field::RustByteSize::Four,
        ),
    ).with_endianess(Endianness::big());
    // This is a round about way of doing structs and is not recommended.
    let mut builder: GenericBuilder = GenericBuilder::single_set(format_ident!("test").into());
    {
        let inner_builder = builder.get_mut().get_mut_struct().unwrap();
        inner_builder.add_field(field_one);
    }

    let thing: Solved = match builder.try_into() {
        Ok(yay) => yay,
        Err(err) => panic!("Failed Solving [{err}]"),
    };

    let gen = match thing.gen() {
        Ok(f) => f,
        Err(err) => panic!("{err}"),
    };

    println!("[{gen}]");

    panic!("This test is incomplete");
}
