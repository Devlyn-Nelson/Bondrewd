use bondrewd::*;

#[derive(Bitfields)]
#[bondrewd(default_endianness = "be")]
struct SimpleWithArray {
    #[bondrewd(element_bit_length = 4)]
    one: [u8; 4],
    two: [bool; 5],
    #[bondrewd(block_bit_length = 20)]
    three: [u8; 3],
}

fn main(){
    let test = SimpleWithArray {
        one: [0b11110000, 0b00001111, 0b11110000, 0b00001001],
        two: [false, true, false, true, false],
        three: [u8::MAX, 0, 0b10101010],
    };
    assert_eq!(test.into_bytes(), [0b0000_1111, 0b0000_1001, 0b01010_111, 0b1_0000000, 0b0_1010101, 0b0_0000000]);
}