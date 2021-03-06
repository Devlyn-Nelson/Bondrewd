use bondrewd::BitfieldEnum;

// for situation where all bits are accounted for, like if this enum was used as a 2bit field than
// we can just let the last option be a valid catch all (in proc_macro code it is still marked as
// an invalid catch all but that doesn't really matter)
#[derive(BitfieldEnum, PartialEq, Debug)]
#[bondrewd_enum(u8)]
enum NoInvalidEnum {
    Zero,
    One,
    Two,
    /// because a field using only 2 bits has no more than 4 possible values this last field will be
    /// automatically marked as the Invalid catch all.
    Three,
}

#[derive(BitfieldEnum, PartialEq, Debug)]
enum InferPrimitiveTypeWithInvalidEnum {
    Zero,
    One,
    Two,
    Three,
}

#[test]
fn enum_infer_primitive_type_with_auto_catch_all() {
    assert!(InferPrimitiveTypeWithInvalidEnum::from_primitive(0u8).into_primitive() == 0);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_primitive(1u8).into_primitive() == 1);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_primitive(2u8).into_primitive() == 2);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_primitive(3u8).into_primitive() == 3);

    // test the catch all functionality
    assert!(InferPrimitiveTypeWithInvalidEnum::from_primitive(4u8).into_primitive() == 3);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_primitive(5u8).into_primitive() == 3);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_primitive(154u8).into_primitive() == 3);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_primitive(255u8).into_primitive() == 3);
}

#[derive(BitfieldEnum, PartialEq, Debug)]
#[bondrewd_enum(u8)]
enum CenteredInvalid {
    BLue,
    One,
    #[bondrewd_enum(invalid)]
    Invalid,
    Three,
    Four,
}

#[test]
fn enum_centered_catch_all() {
    assert_eq!(CenteredInvalid::from_primitive(0u8).into_primitive(), 0);
    assert_eq!(CenteredInvalid::from_primitive(1u8).into_primitive(), 1);
    assert_eq!(CenteredInvalid::from_primitive(2u8).into_primitive(), 2);
    let test = CenteredInvalid::from_primitive(3u8);
    assert_eq!(CenteredInvalid::Three, test);
    assert_eq!(test.into_primitive(), 3);
    assert_eq!(CenteredInvalid::from_primitive(4u8).into_primitive(), 4);

    // test the catch all functionality
    assert_eq!(CenteredInvalid::from_primitive(5u8).into_primitive(), 2);
    assert!(CenteredInvalid::from_primitive(6u8).into_primitive() == 2);
    assert!(CenteredInvalid::from_primitive(154u8).into_primitive() == 2);
    assert!(CenteredInvalid::from_primitive(255u8).into_primitive() == 2);
}

#[derive(BitfieldEnum)]
enum CenteredInvalidPrimitive {
    Zero,
    One,
    Invalid(u8),
    Three,
    Four,
}

#[test]
fn enum_centered_catch_primitive() {
    assert_eq!(
        CenteredInvalidPrimitive::from_primitive(0u8).into_primitive(),
        0
    );
    assert_eq!(
        CenteredInvalidPrimitive::from_primitive(1u8).into_primitive(),
        1
    );
    assert_eq!(
        CenteredInvalidPrimitive::from_primitive(2u8).into_primitive(),
        2
    );
    assert_eq!(
        CenteredInvalidPrimitive::from_primitive(3u8).into_primitive(),
        3
    );
    assert_eq!(
        CenteredInvalidPrimitive::from_primitive(4u8).into_primitive(),
        4
    );

    // test the catch all functionality
    assert_eq!(
        CenteredInvalidPrimitive::from_primitive(5u8).into_primitive(),
        5
    );
    assert!(CenteredInvalidPrimitive::from_primitive(6u8).into_primitive() == 6);
    assert!(CenteredInvalidPrimitive::from_primitive(154u8).into_primitive() == 154);
    assert!(CenteredInvalidPrimitive::from_primitive(255u8).into_primitive() == 255);
}
