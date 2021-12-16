use bondrewd::Bitfields;
#[cfg(feature = "peek_slice")]
use bondrewd::BitfieldPeekError;
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
struct Simple {
    #[bit_length = 3]
    one: u8,
    #[bit_length = 27]
    two: u32,
    #[bit_length = 14]
    three: u16,
    four: u8,
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
struct SimpleWithStruct {
    #[bit_length = 3]
    one: u8,
    #[struct_size = 7]
    two: Simple,
    #[bit_length = 4]
    three: u8,
}

#[test]
fn struct_spanning_multiple_bytes_shift_required() -> anyhow::Result<()> {
    let simple = SimpleWithStruct {
        one: 3,
        two: Simple {
            one: 2,
            two: 6345,
            three: 2145,
            four: 66,
        },
        three: 7,
    };
    assert_eq!(SimpleWithStruct::BYTE_SIZE, 8);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 8);
    assert_eq!(bytes[0], 0b01101000);
    assert_eq!(bytes[1], 0b00000000);
    assert_eq!(bytes[2], 0b00001100);
    assert_eq!(bytes[3], 0b01100100);
    assert_eq!(bytes[4], 0b10010000);
    assert_eq!(bytes[5], 0b11000010);
    assert_eq!(bytes[6], 0b10000100);
    assert_eq!(bytes[7], 0b00001110);

    #[cfg(feature = "peek_slice")]
    {
        //peeks
        assert_eq!(simple.one, SimpleWithStruct::peek_slice_one(&bytes)?);
        assert_eq!(simple.two, SimpleWithStruct::peek_slice_two(&bytes)?);
        assert_eq!(simple.three, SimpleWithStruct::peek_slice_three(&bytes)?);
    }

    // from_bytes
    let new_simple = SimpleWithStruct::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be", flip)]
struct SimpleWithStructWithFlip {
    #[bit_length = 3]
    one: u8,
    #[struct_size = 7]
    two: Simple,
    #[bit_length = 4]
    three: u8,
}

#[test]
fn struct_spanning_multiple_bytes_shift_required_with_flip() -> anyhow::Result<()> {
    let simple = SimpleWithStructWithFlip {
        one: 3,
        two: Simple {
            one: 2,
            two: 6345,
            three: 2145,
            four: 66,
        },
        three: 7,
    };
    assert_eq!(SimpleWithStructWithFlip::BYTE_SIZE, 8);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 8);
    assert_eq!(bytes[7], 0b01101000);
    assert_eq!(bytes[6], 0b00000000);
    assert_eq!(bytes[5], 0b00001100);
    assert_eq!(bytes[4], 0b01100100);
    assert_eq!(bytes[3], 0b10010000);
    assert_eq!(bytes[2], 0b11000010);
    assert_eq!(bytes[1], 0b10000100);
    assert_eq!(bytes[0], 0b00001110);
    #[cfg(feature = "peek_slice")]
    {
        //peeks
        assert_eq!(
            simple.one,
            SimpleWithStructWithFlip::peek_slice_one(&bytes)?
        );
        assert_eq!(
            simple.two,
            SimpleWithStructWithFlip::peek_slice_two(&bytes)?
        );
        assert_eq!(
            simple.three,
            SimpleWithStructWithFlip::peek_slice_three(&bytes)?
        );
    }
    // from_bytes
    let new_simple = SimpleWithStructWithFlip::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
struct SmallStruct {
    one: bool,
    two: bool,
    three: bool,
    four: bool,
    five: bool,
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
struct SimpleWithSingleByteSpanningStruct {
    #[bit_length = 4]
    one: u8,
    #[bondrewd(struct_size = 1, bit_length = 5)]
    two: SmallStruct,
    #[bit_length = 7]
    three: u8,
}
#[test]
fn struct_spanning_two_bytes_shift_required() -> anyhow::Result<()> {
    let small = SmallStruct {
        one: true,
        two: false,
        three: true,
        four: false,
        five: true,
    };
    let simple = SimpleWithSingleByteSpanningStruct {
        one: 0,
        two: small.clone(),
        three: 0,
    };
    assert_eq!(SimpleWithSingleByteSpanningStruct::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes[0], 0b00001010);
    assert_eq!(bytes[1], 0b10000000);
    #[cfg(feature = "peek_slice")]
    {
        //peeks
        assert_eq!(
            simple.one,
            SimpleWithSingleByteSpanningStruct::peek_slice_one(&bytes)?
        );
        assert_eq!(
            simple.two,
            SimpleWithSingleByteSpanningStruct::peek_slice_two(&bytes)?
        );
        assert_eq!(
            simple.three,
            SimpleWithSingleByteSpanningStruct::peek_slice_three(&bytes)?
        );
    }

    // from_bytes
    let new_simple = SimpleWithSingleByteSpanningStruct::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
struct SimpleWithSingleByteNonSpanningStruct {
    #[bit_length = 3]
    one: u8,
    #[bondrewd(struct_size = 1, bit_length = 5)]
    two: SmallStruct,
    three: u8,
}
#[test]
fn struct_within_one_byte_shift_required() -> anyhow::Result<()> {
    let small = SmallStruct {
        one: true,
        two: false,
        three: true,
        four: false,
        five: true,
    };
    let simple = SimpleWithSingleByteNonSpanningStruct {
        one: 2,
        two: small.clone(),
        three: 10,
    };
    assert_eq!(SimpleWithSingleByteNonSpanningStruct::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes[0], 0b01010101);
    assert_eq!(bytes[1], 0b00001010);
    #[cfg(feature = "peek_slice")]
    {
        //peeks
        assert_eq!(
            simple.one,
            SimpleWithSingleByteNonSpanningStruct::peek_slice_one(&bytes)?
        );
        assert_eq!(
            simple.two,
            SimpleWithSingleByteNonSpanningStruct::peek_slice_two(&bytes)?
        );
        assert_eq!(
            simple.three,
            SimpleWithSingleByteNonSpanningStruct::peek_slice_three(&bytes)?
        );
    }

    // from_bytes
    let new_simple = SimpleWithSingleByteNonSpanningStruct::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
