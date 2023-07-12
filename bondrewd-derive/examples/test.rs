use bondrewd::Bitfields;

#[derive(Bitfields)]
#[bondrewd(default_endianness = "be", id_bit_length = 8)]
enum SimpleInner {
    One { little_payload: [u8; 10] },
    Two { big_payload: [u8; 100] },
}

#[derive(Bitfields)]
#[bondrewd(enforce_bytes = 104)]
struct SimpleEnforced {
    header: [u8; 3],
    #[bondrewd(byte_length = 101)]
    packet: SimpleInner,
}

fn main() {}
