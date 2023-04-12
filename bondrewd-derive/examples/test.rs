use bondrewd::*;

#[derive(Bitfields)]
#[repr(u8)]
#[bondrewd(default_endianness = "be", id_bits = 2, enforce_bytes = 3)]
enum Thing {
    One {
        a: u16,
    },
    Two {
        a: u16,
        #[bondrewd(bit_length = 6)]
        b: u8,
    },
    Three {
        #[bondrewd(bit_length = 7)]
        d: u8,
        #[bondrewd(bit_length = 15)]
        e: u16,
    },
    Idk = 0,
}

fn main() {
    let thing = Thing::One { a: 1 };
    let bytes = thing.into_bytes();
    // the first two bits are the id followed by Variant One's `a` field.
    assert_eq!(bytes[0], 0b01_000000);
    assert_eq!(bytes[1], 0b00000000);
    // because Variant One doesn't use the full amount of bytes so the last 6 bytes are just filler.
    assert_eq!(bytes[2], 0b01_000000);
}
