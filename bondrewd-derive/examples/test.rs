use bondrewd::Bitfields;

#[derive(Bitfields)]
#[bondrewd(id_bit_length = 8)]
enum SimpleInner {
    One { little_payload: [u8; 10] },
    Two { big_payload: [u8; 100] },
}

fn main() {}
