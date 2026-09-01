use bondrewd::{Bitfields, BitfieldsSlice};

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be", fill_bits, enforce_bits = 52)]
struct Simple {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 27)]
    two: u32,
    #[bondrewd(bit_length = 14)]
    three: u16,
    four: u8,
}

#[test]
fn simple_inner_struct() {
    assert_eq!(Simple::BIT_SIZE, 56);
    assert_eq!(Simple::BYTE_SIZE, 7);
    let simple = Simple {
        one: 2,
        two: 6345,
        three: 2145,
        four: 66,
    };

    let bytes = simple.clone().into_bytes();
    assert_eq!(
        bytes,
        [
            0b01000000, 0b00000000, 0b01100011, 0b00100100, 0b10000110, 0b00010100, 0b00100000
        ]
    );
    let new = Simple::from_bytes(bytes);
    assert_eq!(new, simple);
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be", fill_bits)]
struct SimpleWithStruct {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 52)]
    two: Simple,
    #[bondrewd(bit_length = 4)]
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
    // check bits
    assert_eq!(
        bytes,
        [
            0b011_010_00,
            0b00000000,
            0b00001100,
            0b01100100,
            0b1_0010000,
            0b11000010,
            0b1000010_0,
            0b11100000,
        ]
    );

    //peeks
    assert_eq!(simple.one, SimpleWithStruct::read_slice_one(&bytes)?);
    assert_eq!(simple.two, SimpleWithStruct::read_slice_two(&bytes)?);
    assert_eq!(simple.three, SimpleWithStruct::read_slice_three(&bytes)?);

    // from_bytes
    let new_simple = SimpleWithStruct::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

/// 33300000
/// 22222223 44444440
/// 22222222 33333334
/// 22222222 23333333
/// 22222222 22222222
/// 22222222 22222222
/// 22222222 22222222
/// 11122222 00011122
#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be", reverse, fill_bits, enforce_bits = 59, dump)]
struct SimpleWithStructWithFlip {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 52)]
    two: Simple,
    #[bondrewd(bit_length = 4)]
    three: u8,
}

fn print_bytes(bytes: &[u8]) {
    print!("[");
    for b in bytes {
        print!("0b{b:08b}, ")
    }
    print!("]\n");
}

// TODO re-impl this test.
#[test]
fn struct_spanning_multiple_bytes_shift_required_with_reverse_1() -> anyhow::Result<()> {
    let simple = SimpleWithStructWithFlip {
        one: 0,
        two: Simple {
            one: 0xFF,
            two: 0xFFFFFFFF,
            three: 0xFFFF,
            four: u8::MAX,
        },
        three: 0,
    };
    assert_eq!(SimpleWithStructWithFlip::BYTE_SIZE, 8);
    // this is not 59 despite enforce_bytes implying that, because `fill_bits` is used
    assert_eq!(SimpleWithStructWithFlip::BIT_SIZE, 64);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 8);

    print_bytes(&bytes);

    // assert_eq!(bytes[0], 0);
    // assert_eq!(bytes[1], 0b1111_1110);
    // assert_eq!(bytes[2], 0b0000_0001);
    // assert_eq!(bytes[3], 0);
    // assert_eq!(bytes[4], 0);
    // assert_eq!(bytes[5], 0);
    // assert_eq!(bytes[6], 0);
    // assert_eq!(bytes[7], 0);

    // assert_eq!(bytes[0], 0);
    // assert_eq!(bytes[1], 0);
    // assert_eq!(bytes[2], 0);
    // assert_eq!(bytes[3], 0);
    // assert_eq!(bytes[4], 0);
    // assert_eq!(bytes[5], 0);
    // assert_eq!(bytes[6], 0b1111_1100);
    // assert_eq!(bytes[7], 0b0000_0011);

    assert_eq!(bytes[0], 0);
    assert_eq!(bytes[1], 0);
    assert_eq!(bytes[2], 0);
    assert_eq!(bytes[3], 0);
    assert_eq!(bytes[4], 0);
    assert_eq!(bytes[5], 0b11100000);
    assert_eq!(bytes[6], 0b00000001);
    assert_eq!(bytes[7], 0b00011110);

    // from_bytes
    let new_simple = SimpleWithStructWithFlip::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

// TODO re-impl this test.
// #[test]
fn struct_spanning_multiple_bytes_shift_required_with_reverse_0() -> anyhow::Result<()> {
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
    assert_eq!(bytes[7], 0b011_010_00);
    assert_eq!(bytes[6], 0b00000000);
    assert_eq!(bytes[5], 0b00001100);
    assert_eq!(bytes[4], 0b01100100);
    assert_eq!(bytes[3], 0b1_0010000);
    assert_eq!(bytes[2], 0b11000010);
    assert_eq!(bytes[1], 0b1000010_0);
    assert_eq!(bytes[0], 0b11100000);

    //peeks
    assert_eq!(
        simple.one,
        SimpleWithStructWithFlip::read_slice_one(&bytes)?
    );
    assert_eq!(
        simple.two,
        SimpleWithStructWithFlip::read_slice_two(&bytes)?
    );
    assert_eq!(
        simple.three,
        SimpleWithStructWithFlip::read_slice_three(&bytes)?
    );

    // from_bytes
    let new_simple = SimpleWithStructWithFlip::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
#[allow(clippy::struct_excessive_bools)]
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be")]
struct SmallStruct {
    one: bool,
    two: bool,
    three: bool,
    four: bool,
    five: bool,
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be")]
struct SimpleWithSingleByteSpanningStruct {
    #[bondrewd(bit_length = 4)]
    one: u8,
    #[bondrewd(bit_length = 5)]
    two: SmallStruct,
    #[bondrewd(bit_length = 7)]
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
        two: small,
        three: 0,
    };
    assert_eq!(SimpleWithSingleByteSpanningStruct::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes[0], 0b0000_1010);
    assert_eq!(bytes[1], 0b1000_0000);

    //peeks
    assert_eq!(
        simple.one,
        SimpleWithSingleByteSpanningStruct::read_slice_one(&bytes)?
    );
    assert_eq!(
        simple.two,
        SimpleWithSingleByteSpanningStruct::read_slice_two(&bytes)?
    );
    assert_eq!(
        simple.three,
        SimpleWithSingleByteSpanningStruct::read_slice_three(&bytes)?
    );

    // from_bytes
    let new_simple = SimpleWithSingleByteSpanningStruct::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be")]
struct SimpleWithSingleByteNonSpanningStruct {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 5)]
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
        two: small,
        three: 10,
    };
    assert_eq!(SimpleWithSingleByteNonSpanningStruct::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes[0], 0b0101_0101);
    assert_eq!(bytes[1], 0b0000_1010);

    //peeks
    assert_eq!(
        simple.one,
        SimpleWithSingleByteNonSpanningStruct::read_slice_one(&bytes)?
    );
    assert_eq!(
        simple.two,
        SimpleWithSingleByteNonSpanningStruct::read_slice_two(&bytes)?
    );
    assert_eq!(
        simple.three,
        SimpleWithSingleByteNonSpanningStruct::read_slice_three(&bytes)?
    );

    // from_bytes
    let new_simple = SimpleWithSingleByteNonSpanningStruct::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
