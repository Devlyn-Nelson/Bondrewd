use bondrewd::Bitfields;

// for situation where all bits are accounted for, like if this enum was used as a 2bit field than
// we can just let the last option be a valid catch all (in proc_macro code it is still marked as
// an invalid catch all but that doesn't really matter)
#[derive(Bitfields, PartialEq, Debug)]
#[bondrewd(id_byte_length = 1, default_endianness = "be")]
enum NoInvalidEnum {
    Zero,
    One,
    Two,
    /// because a field using only 2 bits has no more than 4 possible values this last field will be
    /// automatically marked as the Invalid catch all.
    Three,
}

#[derive(Bitfields, PartialEq, Debug)]
#[bondrewd(id_byte_length = 1, default_endianness = "be")]
enum InferPrimitiveTypeWithInvalidEnum {
    Zero,
    One,
    Two,
    Three,
}

#[test]
fn enum_infer_primitive_type_with_auto_catch_all() {
    assert!(InferPrimitiveTypeWithInvalidEnum::from_bytes([0u8]).into_bytes()[0] == 0);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_bytes([1u8]).into_bytes()[0] == 1);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_bytes([2u8]).into_bytes()[0] == 2);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_bytes([3u8]).into_bytes()[0] == 3);

    // test the catch all functionality
    assert!(InferPrimitiveTypeWithInvalidEnum::from_bytes([4u8]).into_bytes()[0] == 3);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_bytes([5u8]).into_bytes()[0] == 3);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_bytes([154u8]).into_bytes()[0] == 3);
    assert!(InferPrimitiveTypeWithInvalidEnum::from_bytes([255u8]).into_bytes()[0] == 3);
}

#[derive(Bitfields, PartialEq, Debug)]
#[bondrewd(id_byte_length = 1, default_endianness = "be")]
enum CenteredInvalid {
    BLue,
    One,
    #[bondrewd(invalid)]
    Invalid,
    Three,
    Four,
}

#[test]
fn enum_centered_catch_all() {
    assert_eq!(CenteredInvalid::from_bytes([0u8]).into_bytes()[0], 0);
    assert_eq!(CenteredInvalid::from_bytes([1u8]).into_bytes()[0], 1);
    assert_eq!(CenteredInvalid::from_bytes([2u8]).into_bytes()[0], 2);
    let test = CenteredInvalid::from_bytes([3u8]);
    assert_eq!(CenteredInvalid::Three, test);
    assert_eq!(test.into_bytes()[0], 3);
    assert_eq!(CenteredInvalid::from_bytes([4u8]).into_bytes()[0], 4);

    // test the catch all functionality
    assert_eq!(CenteredInvalid::from_bytes([5u8]).into_bytes()[0], 2);
    assert!(CenteredInvalid::from_bytes([6u8]).into_bytes()[0] == 2);
    assert!(CenteredInvalid::from_bytes([154u8]).into_bytes()[0] == 2);
    assert!(CenteredInvalid::from_bytes([255u8]).into_bytes()[0] == 2);
}

#[derive(Bitfields)]
#[bondrewd(id_byte_length = 1, default_endianness = "be")]
enum CenteredInvalidPrimitive {
    Zero,
    One,
    #[bondrewd(invalid)]
    Invalid {
        #[bondrewd(capture_id)]
        id: u8,
    },
    Three,
    Four,
}

#[test]
fn enum_centered_catch_primitive() {
    assert_eq!(
        CenteredInvalidPrimitive::from_bytes([0u8]).into_bytes()[0],
        0
    );
    assert_eq!(
        CenteredInvalidPrimitive::from_bytes([1u8]).into_bytes()[0],
        1
    );
    assert_eq!(
        CenteredInvalidPrimitive::from_bytes([2u8]).into_bytes()[0],
        2
    );
    assert_eq!(
        CenteredInvalidPrimitive::from_bytes([3u8]).into_bytes()[0],
        3
    );
    assert_eq!(
        CenteredInvalidPrimitive::from_bytes([4u8]).into_bytes()[0],
        4
    );

    let invalid_test = CenteredInvalidPrimitive::from_bytes([5u8]);
    if let CenteredInvalidPrimitive::Invalid { id } = invalid_test {
        assert_eq!(id, 5)
    }
    // test the catch all functionality
    assert_eq!(invalid_test.into_bytes()[0], 5);
    assert!(CenteredInvalidPrimitive::from_bytes([6u8]).into_bytes()[0] == 6);
    assert!(CenteredInvalidPrimitive::from_bytes([154u8]).into_bytes()[0] == 154);
    assert!(CenteredInvalidPrimitive::from_bytes([255u8]).into_bytes()[0] == 255);
}
