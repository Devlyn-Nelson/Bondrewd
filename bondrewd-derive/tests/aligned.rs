use bondrewd::Bitfields;

#[derive(Bitfields)]
#[bondrewd(default_endianness = "ale")]
struct Aligned {
    #[bondrewd(bit_length = 9)]
    number: u16,
}

#[test]
fn aligned() {
    assert_eq!(Aligned::BIT_SIZE, 9);
    assert_eq!(Aligned::BYTE_SIZE, 2);
    let ex = Aligned { number: u16::MAX };
    let bytes = ex.into_bytes();
    assert_eq!(bytes, [0b11111111, 0b00000001]);
}

// the original ale problem was structs passing in total bit length for the
// structure's fields instead of the total bits used (including fill) because
// thats what flip is based on. Need same test with enum type.

// #[derive(Bitfields)]
// #[bondrewd(default_endianness = "ale", dump)]
// enum Aligned {
//  Thing {
//     #[bondrewd(bit_length = 9)]
//     number: u16,
//  }
// }
