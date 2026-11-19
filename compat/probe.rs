use bondrewd::Bitfields;
#[derive(Bitfields, Clone, Debug)]
#[bondrewd(endianness = "le")]
struct B24 { #[bondrewd(block_bit_length = 24)] b: [u8; 3] }
#[derive(Bitfields, Clone, Debug)]
#[bondrewd(endianness = "le")]
struct B21 { #[bondrewd(block_bit_length = 21)] b: [u8; 3] }
#[derive(Bitfields, Clone, Debug)]
#[bondrewd(endianness = "le")]
struct W28 { #[bondrewd(block_bit_length = 28)] b: [u16; 2] }
fn main() {
    for b in [[1u8,0,0],[0,1,0],[0,0,1],[0x80,0,0],[0,0,0x80],[0,0,0x10]] {
        println!("B24 {:02x?} -> {:02x?}", b, B24 { b }.into_bytes());
        println!("B21 {:02x?} -> {:02x?}", b, B21 { b }.into_bytes());
    }
    for b in [[1u16,0],[0,1],[0x8000,0],[0,0x8000],[0, 0x0800]] {
        println!("W28 {:04x?} -> {:02x?}", b, W28 { b }.into_bytes());
    }
}
