use bondrewd::Bitfields;

#[derive(Bitfields, Clone)]
#[bondrewd(default_endianness = "le")]
struct Packed {
    #[bondrewd(bit_length = 9)]
    number: u16,
    #[bondrewd(bit_length = 7, reserve)]
    reserve: u16,
}

#[derive(Bitfields)]
#[bondrewd(default_endianness = "le", bit_traversal = "back", reverse)]
struct Aligned {
    #[bondrewd(bit_length = 9)]
    number: u16,
    #[bondrewd(bit_length = 7, reserve)]
    reserve: u16,
}

impl From<Packed> for Aligned {
    fn from(value: Packed) -> Self {
        Self {
            // one: value.one,
            number: value.number,
            reserve: value.reserve,
        }
    }
}

fn main() {
    // Packed
    assert_eq!(Packed::BIT_SIZE, 16);
    assert_eq!(Packed::BYTE_SIZE, 2);
    let ex = Packed {
        // one: 0,
        number: u16::MAX,
        reserve: 0,
    };

    let bytes = ex.clone().into_bytes();
    print_bytes(&bytes);

    // Aligned
    assert_eq!(Aligned::BIT_SIZE, 16);
    assert_eq!(Aligned::BYTE_SIZE, 2);
    let ex: Aligned = ex.into();

    let bytes = ex.into_bytes();
    print_bytes(&bytes);
}

fn print_bytes(bytes: &[u8]) {
    for b in bytes {
        print!("{b:08b}, ");
    }
    print!("\n");
}
