use bondrewd::Bitfields;
#[cfg(feature = "slice_fns")]
use bondrewd::BitfieldSliceError;

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
struct SimpleWithBlockArray {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(array_bit_length = 9)]
    two: [u8; 2],
    #[bondrewd(bit_length = 4)]
    three: u8,
}
#[test]
fn to_bytes_simple_with_block_array_spanning() -> anyhow::Result<()> {
    let simple = SimpleWithBlockArray {
        one: 0,
        two: [1, u8::MAX],
        three: 0,
    };
    assert_eq!(SimpleWithBlockArray::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);

    assert_eq!(bytes[0], 0b00011111);
    assert_eq!(bytes[1], 0b11110000);
    #[cfg(feature = "slice_fns")]
    {
        //peeks
        assert_eq!(simple.one, SimpleWithBlockArray::peek_slice_one(&bytes)?);
        assert_eq!(simple.two, SimpleWithBlockArray::peek_slice_two(&bytes)?);
        assert_eq!(
            simple.three,
            SimpleWithBlockArray::peek_slice_three(&bytes)?
        );
    }

    // from_bytes
    let new_simple = SimpleWithBlockArray::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
