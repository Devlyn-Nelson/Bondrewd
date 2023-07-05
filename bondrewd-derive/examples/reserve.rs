use bondrewd::{BitfieldHex, Bitfields};

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be", read_from = "msb0")]
struct SimpleWithReserve {
    #[bondrewd(bit_length = 9)]
    one: u16,
    #[bondrewd(bit_length = 3, reserve)]
    reserve: u8,
    #[bondrewd(bit_length = 4)]
    two: i8,
    #[bondrewd(element_byte_length = 1)]
    test: [char; 17],
}

fn main() {
    let mut test = ['a'; 17];
    for (i, c) in test.iter_mut().enumerate() {
        if let Some(new_c) = char::from_u32(*c as u32 + i as u32) {
            *c = new_c;
        }
    }
    let testy = SimpleWithReserve {
        one: 5,
        reserve: 0,
        two: -1,
        test,
    };
    let bytes = testy.clone().into_bytes();
    for b in &bytes {
        println!("{b:02X}, ");
    }
    let new_testy = SimpleWithReserve::from_bytes(bytes);
    assert_eq!(testy, new_testy);
}
