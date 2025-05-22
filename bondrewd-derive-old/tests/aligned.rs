use bondrewd::Bitfields;

#[derive(bondrewd_derive_old::Bitfields)]
#[bondrewd(default_endianness = "be", bit_traversal = "back", reverse, fill_bits)]
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
