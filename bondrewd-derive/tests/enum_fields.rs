use bondrewd::{BitfieldEnum, BitfieldHex, Bitfields};

#[derive(Eq, PartialEq, Clone, Debug, BitfieldEnum)]
enum TestEnum {
    Zero,
    One,
    Two,
    Three,
    Other(u8),
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
struct SimpleWithSingleByteSpanningEnum {
    #[bondrewd(bit_length = 6)]
    one: u8,
    #[bondrewd(enum_primitive = "u8", bit_length = 3)]
    two: TestEnum,
    #[bondrewd(bit_length = 7)]
    three: u8,
}
#[test]
fn to_bytes_simple_with_enum_spanning() -> anyhow::Result<()> {
    let simple = SimpleWithSingleByteSpanningEnum {
        one: 0,
        two: TestEnum::Three,
        three: 0,
    };
    assert_eq!(SimpleWithSingleByteSpanningEnum::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes[0], 0b0000_0001);
    assert_eq!(bytes[1], 0b1000_0000);
    #[cfg(feature = "slice_fns")]
    {
        //peeks
        assert_eq!(
            simple.one,
            SimpleWithSingleByteSpanningEnum::read_slice_one(&bytes)?
        );
        assert_eq!(
            simple.two,
            SimpleWithSingleByteSpanningEnum::read_slice_two(&bytes)?
        );
        assert_eq!(
            simple.three,
            SimpleWithSingleByteSpanningEnum::read_slice_three(&bytes)?
        );
    }

    // from_bytes
    let new_simple = SimpleWithSingleByteSpanningEnum::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
