use bondrewd::Bitfields;

#[derive(Bitfields)]
#[bondrewd(endianness = "ale")]
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

#[derive(Bitfields)]
#[bondrewd(endianness = "ale", id_bit_length = 2)]
enum AlignedEnum {
    #[bondrewd(id = 0)]
    Thing {
        #[bondrewd(bit_length = 9)]
        number: u16,
    }
}
#[test]
fn aligned_enum() {
    assert_eq!(AlignedEnum::BIT_SIZE, 11);
    assert_eq!(AlignedEnum::BYTE_SIZE, 2);
    let ex = AlignedEnum::Thing{ number: u16::MAX };
    let bytes = ex.into_bytes();
    assert_eq!(bytes, [0b11111100, 0b0000111]);
}
