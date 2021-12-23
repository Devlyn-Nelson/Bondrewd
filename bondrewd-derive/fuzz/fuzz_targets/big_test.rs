#![no_main]
use libfuzzer_sys::fuzz_target;
use bondrewd::{Bitfields, BitfieldEnum};

#[derive(BitfieldEnum, Clone, PartialEq,  Debug)]
#[bondrewd_enum(u8)]
pub enum Test2Bits {
    One,
    Two,
    Three,
    FourInvalid,
}

/// 3 bitt field describing the version number of Ccsds standard to use.
#[derive(BitfieldEnum, Clone, PartialEq,  Debug)]
pub enum TestInvalid {
    One,
    Two,
    Invalid(u8),
}

#[derive(Bitfields, Clone, PartialEq, Debug)]
#[bondrewd(default_endianness = "be")]
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
#[derive(Bitfields, Clone, PartialEq,  Debug)]
#[bondrewd(default_endianness = "be")]
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
    ten: i128,//366
    #[bondrewd(struct_size = 75, bit_length = 593)]
    test_struct: TestInner,
}

fuzz_target!(|data: [u8;Test::BYTE_SIZE]| {
    assert_eq!(959, Test::BIT_SIZE);
    assert_eq!(120, Test::BYTE_SIZE);
    assert_eq!(Test::from_bytes(data.clone()).into_bytes(), data);
});
