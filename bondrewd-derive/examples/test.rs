use bondrewd::*;

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "le")]
struct Simple {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 27)]
    two: u32,
    #[bondrewd(bit_length = 14)]
    three: u16,
    four: u8,
}

fn main() -> anyhow::Result<()> {
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
    assert_eq!(bytes[1], 0b00100011);
    assert_eq!(bytes[2], 0b00000000);
    assert_eq!(bytes[3], 0b000000_01);
    assert_eq!(bytes[4], 0b10000100);
    assert_eq!(bytes[5], 0b1000_0100);
    // this last 4 bits here don't exist in the struct
    assert_eq!(bytes[6], 0b0010_0000);
    #[cfg(feature = "slice_fns")]
    {
        //peeks
        assert_eq!(simple.one, Simple::peek_slice_one(&bytes)?);
        assert_eq!(simple.two, Simple::peek_slice_two(&bytes)?);
        assert_eq!(simple.three, Simple::peek_slice_three(&bytes)?);
        assert_eq!(simple.four, Simple::peek_slice_four(&bytes)?);
    }

    // from_bytes
    let new_simple = Simple::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
