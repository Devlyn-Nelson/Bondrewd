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

// fill bytes is used here to make the total output byte size 3 bytes.
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be", fill_bytes = 3)]
struct FilledBytes {
    #[bondrewd(bit_length = 7)]
    one: u8,
    #[bondrewd(bit_length = 7)]
    two: u8,
}

fn main() {
    assert_eq!(3, FilledBytes::BYTE_SIZE);
    assert_eq!(24, FilledBytes::BIT_SIZE);
}
