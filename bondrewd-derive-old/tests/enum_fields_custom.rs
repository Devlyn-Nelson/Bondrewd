use bondrewd::Bitfields;

#[derive(Eq, PartialEq, Clone, Debug, bondrewd_derive_old::Bitfields)]
#[bondrewd(default_endianness = "be", id_bit_length = 8)]
enum TestCustomEnum {
    CustomZero = 0x30,
    CustomOne = 0x10,
    CustomTwo = 0x20,
    CustomThree = 0x40,
    Invalid,
}

#[derive(bondrewd_derive_old::Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
struct SimpleCustomEnumUsage {
    one: u8,
    #[bondrewd(bit_length = 8)]
    two: TestCustomEnum,
    three: u8,
}
#[test]
fn to_bytes_simple_with_custom_enum_spanning() -> anyhow::Result<()> {
    let simple = SimpleCustomEnumUsage {
        one: 0x08,
        two: TestCustomEnum::CustomThree,
        three: 0,
    };
    assert_eq!(SimpleCustomEnumUsage::BYTE_SIZE, 3);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 3);
    assert_eq!(bytes[0], 0b0000_1000);
    assert_eq!(bytes[1], 0b0100_0000);
    #[cfg(feature = "dyn_fns")]
    {
        //peeks
        assert_eq!(simple.one, SimpleCustomEnumUsage::read_slice_one(&bytes)?);
        assert_eq!(simple.two, SimpleCustomEnumUsage::read_slice_two(&bytes)?);
        assert_eq!(
            simple.three,
            SimpleCustomEnumUsage::read_slice_three(&bytes)?
        );
    }

    // from_bytes
    let new_simple = SimpleCustomEnumUsage::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

#[derive(Eq, PartialEq, Clone, Debug, bondrewd_derive_old::Bitfields)]
#[bondrewd(default_endianness = "be", id_bit_length = 8)]
enum TestCustomContinuationEnum {
    CustomZero = 0x7F,
    CustomZeroContinued,
    CustomOne = 0x3F,
    CustomOneContinued,
    Invalid,
}

#[derive(bondrewd_derive_old::Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
struct SimpleCustomContinuationEnumUsage {
    one: u8,
    #[bondrewd(bit_length = 8)]
    two: TestCustomContinuationEnum,
    three: u8,
}

#[test]
#[allow(unused_mut)]
fn enum_contiunation_tests() -> anyhow::Result<()> {
    let simple = SimpleCustomContinuationEnumUsage {
        one: 0x80,
        two: TestCustomContinuationEnum::CustomOneContinued,
        three: 0x08,
    };
    assert_eq!(SimpleCustomContinuationEnumUsage::BYTE_SIZE, 3);
    let mut bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 3);
    assert_eq!(bytes[0], 0b1000_0000);
    assert_eq!(bytes[1], 0b0000_0001);
    assert_eq!(bytes[2], 0b0000_1000);
    #[cfg(feature = "dyn_fns")]
    {
        //peeks
        assert_eq!(
            simple.one,
            SimpleCustomContinuationEnumUsage::read_slice_one(&bytes)?
        );
        assert_eq!(
            simple.two,
            SimpleCustomContinuationEnumUsage::read_slice_two(&bytes)?
        );
        assert_eq!(
            simple.three,
            SimpleCustomContinuationEnumUsage::read_slice_three(&bytes)?
        );
    }

    // from bytes
    let new_simple = SimpleCustomContinuationEnumUsage::from_bytes(bytes);
    assert_eq!(simple, new_simple);

    #[cfg(feature = "dyn_fns")]
    {
        // Setter too
        SimpleCustomContinuationEnumUsage::write_slice_two(
            &mut bytes,
            TestCustomContinuationEnum::CustomZeroContinued,
        )?;
        let expected = SimpleCustomContinuationEnumUsage {
            one: 0x80,
            two: TestCustomContinuationEnum::CustomZeroContinued,
            three: 0x08,
        };
        assert_eq!(
            SimpleCustomContinuationEnumUsage::from_bytes(bytes),
            expected
        );
    }
    Ok(())
}
