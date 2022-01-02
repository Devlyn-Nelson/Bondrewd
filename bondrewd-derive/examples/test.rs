use bondrewd::*;

#[derive(Bitfields, Clone, PartialEq, Debug)]
#[bondrewd(default_endianness = "le", enforce_bits = "593")]
pub struct TestInner {
    one: u8,
    two: i8,
    three: u16,
    four: i16,
    five: u32,
    six: i32,
    seven: u64,
    eight: i64,
    nine: u128,
    ten: i128,
    f_one: f32,
    f_two: f64,
    b_one: bool,
}
// 593
#[derive(Bitfields, Clone, PartialEq, Debug)]
#[bondrewd(default_endianness = "le", enforce_bits = 959)]
pub struct Test {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 4)]
    two: i8,
    #[bondrewd(bit_length = 9)]
    three: u16,
    #[bondrewd(bit_length = 14)]
    four: i16,
    #[bondrewd(bit_length = 30)]
    five: u32,
    #[bondrewd(bit_length = 27)]
    six: i32,
    #[bondrewd(bit_length = 56)]
    seven: u64,
    #[bondrewd(bit_length = 43)]
    eight: i64,
    #[bondrewd(bit_length = 69)]
    nine: u128,
    #[bondrewd(bit_length = 111)]
    ten: i128,
    #[bondrewd(struct_size = 75, bit_length = 593)]
    test_struct: TestInner,
}
//00000000_11011101_11011101_11110111
//00000000_11011101_11011101_11111111 - good
//00000000-11011101-11011101-11111111-1111_11111111_11111110_11110100_00000000_00000000_00000000_00000000_00000000_00000001_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111110_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111100_00000000_00000000_00000000_00000000_00000000_00000011_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111100_10111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111100_10000000_10000000_10000000_10000011_11111111_11111111_11111111_11111111_11111111_01110111_01110111_01110111_01110111_01110111_01110111_01110111_01110111_01110111_01110111_01110111_01110100_10100011_01110111_01110111_01110111_01110111_01110111_01110100_00000000_00000000_00000000_00000000_00000000_00000011_11100001_11111110
fn main() {
}
