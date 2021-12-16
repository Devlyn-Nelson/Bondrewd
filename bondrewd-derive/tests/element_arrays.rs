use bondrewd::Bitfields;
#[cfg(feature = "peek_slice")]
use bondrewd::BitfieldPeekError;

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
struct SimpleWithArray {
    #[bondrewd(bit_length = 4)]
    one: u8,
    #[bondrewd(element_bit_length = 1)]
    two: [bool; 5],
    #[bondrewd(bit_length = 7)]
    three: u8,
}
#[test]
fn to_bytes_simple_with_element_array_spanning() -> anyhow::Result<()> {
    let simple = SimpleWithArray {
        one: 0,
        two: [true, false, true, false, true],
        three: 0,
    };
    assert_eq!(SimpleWithArray::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes[0], 0b00001010);
    assert_eq!(bytes[1], 0b10000000);
    #[cfg(feature = "peek_slice")]
    {
        //peeks
        assert_eq!(simple.one, SimpleWithArray::peek_slice_one(&bytes)?);
        assert_eq!(simple.two, SimpleWithArray::peek_slice_two(&bytes)?);
        assert_eq!(simple.three, SimpleWithArray::peek_slice_three(&bytes)?);
    }

    // from_bytes
    let new_simple = SimpleWithArray::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
