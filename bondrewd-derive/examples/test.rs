use bondrewd::*;

#[derive(Bitfields)]
#[repr(u8)]
#[bondrewd(default_endianness = "be", id_bit_length = 2, enforce_bits = 18)]
enum Thing {
    One {
        a: u16,
    } = 1,
    Two {
        #[bondrewd(bit_length = 10)]
        a: u16,
        #[bondrewd(bit_length = 6)]
        b: u8,
    } = 2,
    Idk {
        #[bondrewd(capture_id)]
        id: u8,
        a: u16,
    } = 0,
}

fn main() {
    // fields with capture_id will use the id_bit_length so defining the bit_length is unnecessary.
    assert_eq!(Thing::BYTE_SIZE, 3);
    assert_eq!(Thing::BIT_SIZE, 18);
    // fields that are capturing the id do not write.
    let mut bytes = Thing::Idk { id: 3, a: 0 }.into_bytes();
    // despite setting the id to 3 it will be 0 on output, this is to prevent
    // users from providing a valid id when it should not be.
    assert_eq!(bytes[0], 0b00000000);
    assert_eq!(bytes[1], 0b00000000);
    assert_eq!(bytes[2], 0b00000000);
    // but the id can be set to anything using the write_variant_id function.
    Thing::write_variant_id(&mut bytes, 3);
    // the id is now 3
    assert_eq!(bytes[0], 0b11000000);
    assert_eq!(bytes[1], 0b00000000);
    assert_eq!(bytes[2], 0b00000000);
    let reconstructed = Thing::from_bytes(bytes);
    // other than into_bytes everything else with give you the stored value.
    assert_eq!(reconstructed.id(), 3);
    match reconstructed {
        Thing::Idk { id, .. } => assert_eq!(id, 3),
        _ => panic!("id wasn't 3"),
    }
}
