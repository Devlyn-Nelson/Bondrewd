use bondrewd::Bitfields;

#[derive(Eq, PartialEq, Clone, Debug, Bitfields)]
#[bondrewd(id_byte_length = 1, default_endianness = "be")]
enum TestPartialEqEnum {
    Zero,
    One,
    Two,
    Three,
    Invalid,
}

#[derive(Eq, PartialEq, Clone, Debug, Bitfields)]
#[bondrewd(id_byte_length = 1, default_endianness = "be")]
enum TestPartialEqCustomEnum {
    CustomZero = 0x10,
    CustomOne = 0x20,
    CustomTwo = 0x30,
    CustomThree = 0x40,
    Invalid = 0xFF,
}

#[derive(Eq, PartialEq, Clone, Debug, Bitfields)]
#[bondrewd(id_byte_length = 1, default_endianness = "be")]
enum TestNoPartialEqCustomEnum {
    CustomZero = 0x10,
    CustomOne = 0x20,
    CustomTwo = 0x30,
    CustomThree = 0x40,
    Invalid = 0xFF,
}

#[cfg(feature = "part_eq_enums")]
#[test]
fn enum_partial_eq_tests() -> anyhow::Result<()> {
    // Create some enums and compare directly to numbers
    let simple_one = TestPartialEqEnum::One;
    let simple_three = TestPartialEqEnum::Three;
    let simple_invalid = TestPartialEqEnum::Invalid;

    assert_eq!(simple_one, 1_u8);
    assert_eq!(simple_three, 3_u8);
    assert_eq!(simple_invalid, 4_u8);

    // Create some custom enums
    let custom_one = TestPartialEqCustomEnum::CustomOne;
    let custom_three = TestPartialEqCustomEnum::CustomThree;
    let custom_invalid = TestPartialEqCustomEnum::Invalid;

    assert_eq!(custom_one, 0x20_u8);
    assert_eq!(custom_three, 0x40_u8);
    assert_eq!(custom_invalid, 0xFF);

    // Test against a non partial_eq enum too
    let no_partial_one = TestNoPartialEqCustomEnum::CustomOne;
    let no_partial_three = TestNoPartialEqCustomEnum::CustomThree;
    assert_eq!(no_partial_one.into_bytes()[0], 0x20);
    assert_eq!(no_partial_three.into_bytes()[0], 0x40);
    Ok(())
}
