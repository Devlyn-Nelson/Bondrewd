use bitfields::Bitfields;
use bitfields_derive::Bitfields as BitfieldsDerive;

#[derive(BitfieldsDerive, Clone, PartialEq, Eq, Debug)]
#[bitfields(default_endianness = "be")]
struct Simple {
    #[bit_length = 3]
    one: u8,
    #[bit_length = 27]
    two: u32,
    #[bit_length = 14]
    three: u16,
    four: u8,
}

#[test]
fn to_bytes_simple() -> anyhow::Result<()> {
    let simple = Simple {
        one: 2,
        two: 6345,
        three: 2145,
        four: 66,
    };
    assert_eq!(Simple::BYTE_SIZE, 7);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 7);
    assert_eq!(bytes[0], 0b010_00000);
    assert_eq!(bytes[1], 0b00000000);
    assert_eq!(bytes[2], 0b01100011);
    assert_eq!(bytes[3], 0b001001_00);
    assert_eq!(bytes[4], 0b10000110);
    assert_eq!(bytes[5], 0b0001_0100);
    // this last 4 bits here don't exist in the struct
    assert_eq!(bytes[6], 0b0010_0000);

    //peeks
    assert_eq!(simple.one, Simple::peek_slice_one(&bytes)?);
    assert_eq!(simple.two, Simple::peek_slice_two(&bytes)?);
    assert_eq!(simple.three, Simple::peek_slice_three(&bytes)?);
    assert_eq!(simple.four, Simple::peek_slice_four(&bytes)?);

    // from_bytes
    let new_simple = Simple::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

#[derive(BitfieldsDerive, Clone, PartialEq, Eq, Debug)]
#[bitfields(default_endianness = "be", flip)]
struct SimpleWithFlip {
    one: bool,
    #[bit_length = 10]
    two: u16,
    #[bit_length = 5]
    three: u8,
}
#[test]
fn to_bytes_simple_with_flip() -> anyhow::Result<()> {
    let simple = SimpleWithFlip {
        one: false,
        two: u16::MAX & 0b0000001111111111,
        three: 0,
    };
    assert_eq!(SimpleWithFlip::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);

    assert_eq!(bytes[1], 0b01111111);
    assert_eq!(bytes[0], 0b11100000);
    //peeks
    assert_eq!(simple.one, SimpleWithFlip::peek_slice_one(&bytes)?);
    assert_eq!(simple.two, SimpleWithFlip::peek_slice_two(&bytes)?);
    assert_eq!(simple.three, SimpleWithFlip::peek_slice_three(&bytes)?);

    // from_bytes
    let new_simple = SimpleWithFlip::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

#[derive(BitfieldsDerive, Clone, PartialEq, Eq, Debug)]
#[bitfields(default_endianness = "be", read_from = "back")]
struct SimpleWithReadFromBack {
    one: bool,
    #[bit_length = 10]
    two: u16,
    #[bit_length = 5]
    three: u8,
}
#[test]
fn to_bytes_simple_with_read_from_back() -> anyhow::Result<()> {
    let simple = SimpleWithReadFromBack {
        one: false,
        two: u16::MAX & 0b0000001111111111,
        three: 0,
    };
    assert_eq!(SimpleWithReadFromBack::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);

    assert_eq!(bytes[0], 0b00000111);
    assert_eq!(bytes[1], 0b11111110);
    //peeks
    assert_eq!(simple.one, SimpleWithReadFromBack::peek_slice_one(&bytes)?);
    assert_eq!(simple.two, SimpleWithReadFromBack::peek_slice_two(&bytes)?);
    assert_eq!(
        simple.three,
        SimpleWithReadFromBack::peek_slice_three(&bytes)?
    );

    // from_bytes
    let new_simple = SimpleWithReadFromBack::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
