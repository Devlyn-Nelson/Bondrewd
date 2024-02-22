use bondrewd::Bitfields;

#[derive(Bitfields, Clone, Default, PartialEq, Eq)]
#[bondrewd(read_from = "msb0", default_endianness = "le", enforce_bytes = 2)]
pub struct Packet {
    #[bondrewd(bit_length = 7, reserve)]
    #[allow(dead_code)]
    pub reserved: u8,
    #[bondrewd(bit_length = 9)]
    pub number: u16,
}

#[test]
fn issue_12(){
    // The 1s here mark the `reserve` field.
    let test_bytes = [0b1111_1110, 0b0000_0000];
    let packet = Packet::from_bytes(test_bytes);

    assert_eq!(packet.reserved, 0);
    assert_eq!(packet.number, 0);

    // The 1s here mark the `number` field and we should get the maximum value possible.
    let test_bytes = [0b0000_0001, 0b1111_1111];
    let packet = Packet::from_bytes(test_bytes);

    assert_eq!(packet.reserved, 0);
    assert_eq!(packet.number, 511);

    // `number` most-significant bit (first bit in the last byte, last byte only has 1 bit so...). Little
    // endian stores small end first, So putting a single 1 at this bit position should give `number` a value of 256.
    let test_bytes = [0b0000_0000, 0b0000_0001];
    let packet = Packet::from_bytes(test_bytes);

    assert_eq!(packet.reserved, 0);
    assert_eq!(packet.number, 256);

    // `number` most-significant bit of first byte (first bit in the first byte). filling only this bit
    // should give `number` a value of 128.
    let test_bytes = [0b0000_0001, 0b0000_0000];
    let packet = Packet::from_bytes(test_bytes);

    assert_eq!(packet.reserved, 0);
    assert_eq!(packet.number, 128);

    // `number` least-significant bit. filling least-significant bit means value of 1.
    let test_bytes = [0b0000_0000, 0b0000_0010];
    let packet = Packet::from_bytes(test_bytes);

    assert_eq!(packet.reserved, 0);
    assert_eq!(packet.number, 1);
}