use bondrewd::*;

#[derive(Bitfields, Clone, PartialEq, Debug)]
#[bondrewd(default_endianness = "be", enforce_bits = "593")]
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
#[bondrewd(default_endianness = "be", enforce_bits = 959)]
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

fn main(){
    /*assert_eq!(7, SimpleExample::BYTE_SIZE);
    assert_eq!(53, SimpleExample::BIT_SIZE);
    let mut bytes = SimpleExample {
        one: false,
        two: 0,
        three: -1034,
        four: 63,
    }.into_bytes();
    // one_two_three_four in binary. the last 3 bits are unused.
    assert_eq!([
        0b0_1100000,
        0b01000100,
        0b00000000,
        0b00000000,
        0b0_1110111,
        0b1110110_1,
        0b11111000
    ], bytes);
    assert_eq!(false, SimpleExample::read_one(&bytes));
    assert_eq!(-4.5, SimpleExample::read_two(&bytes));
    assert_eq!(-1034, SimpleExample::read_three(&bytes));
    assert_eq!(63, SimpleExample::read_four(&bytes));
    SimpleExample::write_one(&mut bytes, true);
    SimpleExample::write_two(&mut bytes, 5.5);
    SimpleExample::write_three(&mut bytes, 511);
    SimpleExample::write_four(&mut bytes, 0);
    let reconstructed = SimpleExample::from_bytes(bytes);
    assert_eq!(true,reconstructed.one);
    assert_eq!(5.5,reconstructed.two);
    assert_eq!(511,reconstructed.three);
    assert_eq!(0,reconstructed.four);*/
}