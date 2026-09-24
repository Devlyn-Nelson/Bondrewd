use bondrewd::Bitfields;

fn print_bytes(bytes: &[u8]) {
    print!("[");
    for b in bytes {
        print!("0b{b:08b}, ")
    }
    print!("]\n");
}

#[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(endianness = "ale")]
pub struct Temp {
    #[bondrewd(bit_length = 4)]
    pub zero: u8,
    #[bondrewd(bit_length = 4)]
    pub one: u8,
    #[bondrewd(bit_length = 4)]
    pub two: u8,
}

/// TODO This subtract underflows.
#[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(endianness = "ale")]
pub struct Nested {
    #[bondrewd(bit_length = 12)]
    pub zero: Temp,
    #[bondrewd(bit_length = 4)]
    pub one: u8,
}

#[test]
fn temp() {
    let temp = Temp {
        zero: 127,
        one: 127,
        two: 127,
    };
    let nested = Nested {
        zero: temp.clone(),
        one: 127,
    };
    let bytes = temp.into_bytes();
    print_bytes(&bytes);
    assert_eq!(bytes, [0b1111_0000, 0b1111_1111]);
}
