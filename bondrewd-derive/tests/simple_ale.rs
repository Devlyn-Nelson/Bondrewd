use bondrewd::{Bitfields, BitfieldsSlice};

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale", enforce_bits = 52)]
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
fn ale_into_bytes_simple() -> anyhow::Result<()> {
    let simple = Simple {
        one: 2,
        two: 6345,
        three: 2145,
        four: 66,
    };
    assert_eq!(Simple::BYTE_SIZE, 7);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 7);
    assert_eq!(
        bytes,
        [0b10100000, 0b01100100, 0b00001100, 0b00000000, 0b10000100, 0b00100001, 0b01000010]
    );
    {
        //peeks
        assert_eq!(simple.one, Simple::read_slice_one(&bytes)?);
        assert_eq!(simple.two, Simple::read_slice_two(&bytes)?);
        assert_eq!(simple.three, Simple::read_slice_three(&bytes)?);
        assert_eq!(simple.four, Simple::read_slice_four(&bytes)?);
    }

    // from_bytes
    let new_simple = Simple::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale", reverse)]
struct SimpleWithFlip {
    one: bool,
    #[bondrewd(bit_length = 10)]
    two: u16,
    #[bondrewd(bit_length = 5)]
    three: u8,
}
#[test]
fn ale_into_bytes_simple_with_reverse() -> anyhow::Result<()> {
    let simple = SimpleWithFlip {
        one: false,
        two: u16::MAX & 0b0000_0011_1111_1111,
        three: 0,
    };
    assert_eq!(SimpleWithFlip::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);

    assert_eq!(bytes, [0b00000111, 0b11111110,]);
    {
        //peeks
        assert_eq!(simple.one, SimpleWithFlip::read_slice_one(&bytes)?);
        assert_eq!(simple.two, SimpleWithFlip::read_slice_two(&bytes)?);
        assert_eq!(simple.three, SimpleWithFlip::read_slice_three(&bytes)?);
    }

    // from_bytes
    let new_simple = SimpleWithFlip::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale", bit_traversal = "back")]
struct SimpleWithReadFromBack {
    one: bool,
    #[bondrewd(bit_length = 10)]
    two: u16,
    #[bondrewd(bit_length = 5)]
    three: u8,
}
#[test]
fn ale_into_bytes_simple_with_read_from_back() -> anyhow::Result<()> {
    let simple = SimpleWithReadFromBack {
        one: false,
        two: u16::MAX & 0b0000_0011_1111_1111,
        three: 0,
    };
    assert_eq!(SimpleWithReadFromBack::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);

    assert_eq!(bytes, [0b11100000, 0b01111111,]);
    {
        //peeks
        assert_eq!(simple.one, SimpleWithReadFromBack::read_slice_one(&bytes)?);
        assert_eq!(simple.two, SimpleWithReadFromBack::read_slice_two(&bytes)?);
        assert_eq!(
            simple.three,
            SimpleWithReadFromBack::read_slice_three(&bytes)?
        );
    }

    // from_bytes
    let new_simple = SimpleWithReadFromBack::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale", bit_traversal = "front")]
struct SimpleWithReserve {
    #[bondrewd(bit_length = 9)]
    one: u16,
    #[bondrewd(bit_length = 3, reserve)]
    reserve: u8,
    #[bondrewd(bit_length = 4)]
    two: i8,
}

#[test]
fn ale_into_bytes_simple_with_reserve_field() -> anyhow::Result<()> {
    let mut simple = SimpleWithReserve {
        one: 341,
        reserve: u8::MAX,
        two: -1,
    };
    assert_eq!(SimpleWithReserve::BYTE_SIZE, 2);
    let mut bytes: [u8; 2] = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);

    assert_eq!(bytes, [0b01010101, 0b11110001,]);
    {
        //peeks
        assert_eq!(simple.one, SimpleWithReserve::read_slice_one(&bytes)?);
        assert_eq!(0, SimpleWithReserve::read_slice_reserve(&bytes)?);
        assert_eq!(simple.two, SimpleWithReserve::read_slice_two(&bytes)?);
        // TODO write more set slice tests
        SimpleWithReserve::write_slice_one(&mut bytes, 0)?;
        SimpleWithReserve::write_slice_reserve(&mut bytes, 7)?;
        SimpleWithReserve::write_slice_two(&mut bytes, 0)?;
        simple.one = 0;
        simple.two = 0;
    }
    assert_eq!(7, SimpleWithReserve::read_reserve(&bytes));
    assert!(SimpleWithReserve::read_reserve(&bytes) != simple.reserve);
    simple.reserve = 0;
    // from_bytes
    let new_simple = SimpleWithReserve::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
#[allow(clippy::struct_excessive_bools)]
struct SimpleDuplicateData {
    one: bool,
    two: bool,
    three: bool,
    four: bool,
    five: bool,
    six: bool,
    seven: bool,
    eight: bool,
    #[bondrewd(bits = "0..8", redundant)]
    dup: u8,
    nine: u8,
}

#[test]
fn ale_duplicate_data() {
    let data = SimpleDuplicateData {
        one: false,
        two: false,
        three: false,
        four: false,
        five: false,
        six: false,
        seven: false,
        eight: false,
        dup: 0,
        nine: u8::MAX,
    };
    assert_eq!(SimpleDuplicateData::BYTE_SIZE, 2);
    let bytes = data.into_bytes();
    assert_eq!(bytes[0], 0);
    assert_eq!(bytes[1], u8::MAX);
    let new_data = SimpleDuplicateData::from_bytes(bytes);

    assert!(!new_data.one);
    assert!(!new_data.two);
    assert!(!new_data.three);
    assert!(!new_data.four);
    assert!(!new_data.five);
    assert!(!new_data.six);
    assert!(!new_data.seven);
    assert!(!new_data.eight);
    assert_eq!(new_data.dup, 0);
    assert_eq!(new_data.nine, u8::MAX);
}
