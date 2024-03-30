use bondrewd::Bitfields;

#[derive(Bitfields, Clone)]
#[bondrewd(default_endianness = "le")]
struct Packed {
    #[bondrewd(bit_length = 9)]
    number: u16,
}

#[derive(Bitfields)]
#[bondrewd(default_endianness = "ale")]
struct Aligned {
    #[bondrewd(bit_length = 9)]
    number: u16,
    #[bondrewd(bit_length = 7, reserve)]
    reserve: u8,
}

impl From<Packed> for Aligned {
    fn from(value: Packed) -> Self {
        Self {
            number: value.number,
            reserve: 0,
        }
    }
}

fn main() {
    // Packed
    assert_eq!(Packed::BIT_SIZE, 9);
    assert_eq!(Packed::BYTE_SIZE, 2);
    let ex = Packed { number: u16::MAX };

    let bytes = ex.clone().into_bytes();
    assert_eq!(bytes, [0b11111111, 0b10000000]);

    // Aligned
    assert_eq!(Aligned::BIT_SIZE, 16);
    assert_eq!(Aligned::BYTE_SIZE, 2);
    let ex: Aligned = ex.into();

    let bytes = ex.into_bytes();
    if bytes != [0b11111111, 0b00000001] {
        panic!(
            "[{:#08b}, {:#08b}] != [0b11111111, 0b00000001]",
            bytes[0], bytes[1]
        );
    }
}
