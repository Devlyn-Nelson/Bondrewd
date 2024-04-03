use bondrewd::Bitfields;

#[derive(Bitfields, Clone, Debug, PartialEq, Eq)]
#[bondrewd(id_bit_length = 4, dump)]
pub enum SecretFormat {
    Zero = 0x0,
    One = 0x1,
    Two = 0x2,
    Three = 0x3,
    Invalid,
}

#[test]
/// this must work for secret reasons.
fn must_work() {
    let zero = SecretFormat::Zero;
    let one = SecretFormat::One;
    let two = SecretFormat::Two;
    let three = SecretFormat::Three;
    let invalid = SecretFormat::Invalid;

    assert_eq!(zero.clone().into_bytes(), [0x0]);
    assert_eq!(one.clone().into_bytes(), [0b0001_0000]);
    assert_eq!(two.clone().into_bytes(), [0b0010_0000]);
    assert_eq!(three.clone().into_bytes(), [0b0011_0000]);
    assert_eq!(invalid.clone().into_bytes(), [0b0100_0000]);
}

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
        assert_eq!(id, 5);
    }
    // test the catch all functionality
    assert_eq!(invalid_test.into_bytes()[0], 5);
    assert!(CenteredInvalidPrimitive::from_bytes([6u8]).into_bytes()[0] == 6);
    assert!(CenteredInvalidPrimitive::from_bytes([154u8]).into_bytes()[0] == 154);
    assert!(CenteredInvalidPrimitive::from_bytes([255u8]).into_bytes()[0] == 255);
}

#[derive(Bitfields, Debug, Clone)]
#[bondrewd(id_bit_length = 8, default_endianness = "be")]
enum TupleEnum {
    One(u8),
    Two(u8),
    Invalid(#[bondrewd(capture_id)] u8, u8),
}

#[test]
fn tuple_enum() {
    let one: TupleEnum = TupleEnum::One(1);
    let two = TupleEnum::Two(2);
    let err = TupleEnum::Invalid(4, 3);

    let mut one_bytes = one.clone().into_bytes();
    let mut two_bytes = two.clone().into_bytes();
    let mut err_bytes = err.clone().into_bytes();

    assert_eq!(one_bytes, [0, 1]);
    assert_eq!(two_bytes, [1, 2]);
    assert_eq!(err_bytes, [4, 3]);

    // i am rotating the values so that `one` gets `two's` value, `two` gets `err's`, and `err` gets `one's`.
    TupleEnum::write_one_field_1(&mut one_bytes, 2);
    TupleEnum::write_two_field_1(&mut two_bytes, 3);
    TupleEnum::write_invalid_field_1(&mut err_bytes, 1);

    // rotating the id's the same way the values were rotated.
    TupleEnum::write_variant_id(&mut one_bytes, 1);
    TupleEnum::write_variant_id(&mut two_bytes, 4);
    TupleEnum::write_variant_id(&mut err_bytes, 0);

    // because we rotated the bytes above using the write function we should name to reconstructed
    // structures as they should be based oin the actual values.
    //
    // ex.
    // `two_bytes` was set to the same values as `one` so `two_bytes` will become `new_one` and be checked
    // against `one`.
    let mut _new_one: TupleEnum = TupleEnum::from_bytes(two_bytes);
    let mut _new_two = TupleEnum::from_bytes(err_bytes);
    let mut _new_err = TupleEnum::from_bytes(one_bytes);

    assert!(matches!(one, _new_one));
    assert!(matches!(two, _new_two));
    assert!(matches!(err, _new_err));
}

#[derive(Bitfields, Debug, Clone)]
#[bondrewd(id_bit_length = 8, default_endianness = "be")]
enum CrazyEnum {
    Wack {
        #[bondrewd(bit_length = 4)]
        funky: u8,
        #[bondrewd(bit_length = 4)]
        groovy: u8,
    },
    Loco(u8),
    InsaneInTheBrain,
    CrazyBin(#[bondrewd(capture_id)] u8, i8),
}

#[cfg(dyn_fns)]
#[test]
fn crazy_enum() {
    let mut thing = CrazyEnum::Wack {
        funky: 1,
        groovy: 2,
    }
    .into_bytes();
    match CrazyEnum::check_slice(&thing) {
        Ok(checked) => match checked {
            CrazyEnumChecked::Wack(w) => {
                assert_eq!(w.read_funky(), 1);
                assert_eq!(w.read_groovy(), 2);
            }
            CrazyEnumChecked::Loco(_) => panic!("check slice returned incorrect variant (Loco)"),
            CrazyEnumChecked::InsaneInTheBrain(_) => {
                panic!("check slice returned incorrect variant (InsaneInTheBrain)")
            }
            CrazyEnumChecked::CrazyBin(_) => {
                panic!("check slice returned incorrect variant (CrazyBin)")
            }
        },
        Err(err) => panic!("{err}"),
    }
    match CrazyEnum::check_slice_mut(&mut thing) {
        Ok(checked) => match checked {
            CrazyEnumCheckedMut::Wack(mut w) => {
                w.write_funky(3);
                w.write_groovy(4);
                assert_eq!(w.read_funky(), 3);
                assert_eq!(w.read_groovy(), 4);
            }
            CrazyEnumCheckedMut::Loco(_) => panic!("check slice returned incorrect variant (Loco)"),
            CrazyEnumCheckedMut::InsaneInTheBrain(_) => {
                panic!("check slice returned incorrect variant (InsaneInTheBrain)")
            }
            CrazyEnumCheckedMut::CrazyBin(_) => {
                panic!("check slice returned incorrect variant (CrazyBin)")
            }
        },
        Err(err) => panic!("{err}"),
    }
    CrazyEnum::write_variant_id(&mut thing, 3);
    match CrazyEnum::check_slice(&thing) {
        Ok(checked) => match checked {
            CrazyEnumChecked::Wack(_) => {
                panic!("check slice returned incorrect variant (CrazyBin)")
            }
            CrazyEnumChecked::Loco(_) => panic!("check slice returned incorrect variant (Loco)"),
            CrazyEnumChecked::InsaneInTheBrain(_) => {
                panic!("check slice returned incorrect variant (InsaneInTheBrain)")
            }
            CrazyEnumChecked::CrazyBin(cb) => {
                assert_eq!(cb.read_field_1(), 0b00110100);
                assert_eq!(cb.read_variant_id(), 3);
            }
        },
        Err(err) => panic!("{err}"),
    }
}
