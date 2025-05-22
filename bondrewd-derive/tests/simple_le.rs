use bondrewd::Bitfields;

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "le", enforce_bits = 52)]
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
fn le_into_bytes_simple() -> anyhow::Result<()> {
    let simple = Simple {
        one: 2,
        two: 6345,
        three: 2145,
        four: 66,
    };
    assert_eq!(Simple::BYTE_SIZE, 7);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 7);
    assert_eq!(bytes[0], 0b010_11001);
    assert_eq!(bytes[1], 0b0010_0011);
    assert_eq!(bytes[2], 0b0000_0000);
    assert_eq!(bytes[3], 0b0000_0001);
    assert_eq!(bytes[4], 0b1000_0100);
    assert_eq!(bytes[5], 0b1000_0100);
    // this last 4 bits here don't exist in the struct
    assert_eq!(bytes[6], 0b0010_0000);
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

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "le", reverse)]
struct SimpleWithFlip {
    one: bool,
    #[bondrewd(bit_length = 10)]
    two: u16,
    #[bondrewd(bit_length = 5)]
    three: u8,
}
#[test]
fn le_into_bytes_simple_with_reverse() -> anyhow::Result<()> {
    let simple = SimpleWithFlip {
        one: false,
        two: u16::MAX & 0b0000_0011_1111_1111,
        three: 0,
    };
    assert_eq!(SimpleWithFlip::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);

    assert!(
        !(bytes[0] != 0b1110_0000 || bytes[1] != 0b0111_1111),
        "[{:08b}, {:08b}]!=[0b1110_0000, 0b0111_1111]",
        bytes[0],
        bytes[1]
    );
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

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "le", bit_traversal = "back")]
struct SimpleWithReadFromBack {
    one: bool,
    #[bondrewd(bit_length = 10)]
    two: u16,
    #[bondrewd(bit_length = 5)]
    three: u8,
}
#[test]
fn le_into_bytes_simple_with_read_from_back() -> anyhow::Result<()> {
    let simple = SimpleWithReadFromBack {
        one: false,
        two: u16::MAX & 0b0000_0011_1111_1111,
        three: 0,
    };
    assert_eq!(SimpleWithReadFromBack::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);

    assert!(
        !(bytes[0] != 0b0000_0111 || bytes[1] != 0b1111_1110),
        "[{:08b}, {:08b}]!=[0b0000_0111, 0b1111_1110]",
        bytes[0],
        bytes[1]
    );
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

#[derive(Bitfields, Clone, PartialEq, Debug)]
#[bondrewd(default_endianness = "le")]
struct SimpleWithFloats {
    #[bondrewd(bit_length = 32)]
    one: f32,
    #[bondrewd(bit_length = 64)]
    two: f64,

    three: f32,
}
#[allow(clippy::float_cmp)]
#[test]
fn le_into_bytes_simple_floating_point() -> anyhow::Result<()> {
    let simple = SimpleWithFloats {
        one: f32::from_bits(0x0000_0000_u32),
        two: f64::from_bits(0x09A1_D45E_E54D_1A90_u64),
        three: f32::from_bits(0x0001_D45E_u32),
    };
    let bytes = simple.clone().into_bytes();
    {
        //peeks
        assert_eq!(simple.one, SimpleWithFloats::read_slice_one(&bytes)?);
        //assert_eq!(simple.two, SimpleWithFloats::read_slice_two(&bytes)?);
        //assert_eq!(simple.three, SimpleWithFloats::read_slice_three(&bytes)?);
    }

    // from_bytes
    let new_simple = SimpleWithFloats::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
