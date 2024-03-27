use bondrewd::Bitfields;
#[derive(Bitfields, Clone, Debug, PartialEq)]
#[bondrewd(default_endianness = "le")]
struct Packed {
    #[bondrewd(bit_length = 4)]
    one: u16,
    #[bondrewd(bit_length = 10)]
    two: u16,
    #[bondrewd(bit_length = 2)]
    three: u16,
}

// #[derive(Bitfields, Clone, Debug, PartialEq)]
// #[bondrewd(default_endianness = "le", aligned)]
// struct Aligned {
//     #[bondrewd(bit_length = 4)]
//     one: u16,
//     #[bondrewd(bit_length = 10)]
//     two: u16,
//     #[bondrewd(bit_length = 2)]
//     three: u16,
// }

// #[derive(Bitfields, Clone, Debug, PartialEq)]
// #[bondrewd(default_endianness = "le")]
// struct OneAligned {
//     #[bondrewd(bit_length = 4, aligned)]
//     one: u16,
//     #[bondrewd(bit_length = 10)]
//     two: u16,
//     #[bondrewd(bit_length = 2)]
//     three: u16,
// }

fn main() {
    // Packed
    assert_eq!(Packed::BIT_SIZE, 16);
    assert_eq!(Packed::BYTE_SIZE, 2);
    let ex = Packed {
        one: 5,
        two: 260,
        three: 2,
    };

    let bytes = ex.clone().into_bytes();
    assert_eq!(bytes, [0b0101_0000, 0b0100_0110]);

    // // Aligned
    // assert_eq!(Aligned::BIT_SIZE, 16);
    // assert_eq!(Aligned::BYTE_SIZE, 4);
    // let ex = Aligned {
    //     one: 5,
    //     two: 260,
    //     three: 2,
    // };

    // let bytes = ex.clone().into_bytes();
    // assert_eq!(bytes, [0b00000101, 0b00000001, 0b00000100, 0b00000010]);

    // // OneAligned
    // assert_eq!(OneAligned::BIT_SIZE, 16);
    // assert_eq!(OneAligned::BYTE_SIZE, 4);
    // let ex = OneAligned {
    //     one: 5,
    //     two: 260,
    //     three: 2,
    // };

    // let bytes = ex.clone().into_bytes();
    // assert_eq!(bytes, [0b00000101, 0b01000001, 0b00100000]);
}
