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

#[derive(Clone, PartialEq, Debug, arbitrary::Arbitrary)]
pub struct TestInnerArb {
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

fuzz_target!(|data: [TestInnerArb;2]| {
    assert_eq!(959, Test::BIT_SIZE);
    assert_eq!(120, Test::BYTE_SIZE);
    let mut test = Test {
        one:0,
        two:0,
        three:0,
        four:0,
        five:0,
        six:0,
        seven:0,
        eight:0,
        nine:0,
        ten:0,
        test_struct: TestInner{
            one:0,
            two:0,
            three:0,
            four:0,
            five:0,
            six:0,
            seven:0,
            eight:0,
            nine:0,
            ten:0,
            f_one: 0.0,
            f_two: 0.0,
            b_one: false,
        },
    };
    test.set_one(data[1].one);
    test.set_two(data[1].two);
    test.set_three(data[1].three);
    test.set_four(data[1].four);
    test.set_five(data[1].five);
    test.set_six(data[1].six);
    test.set_seven(data[1].seven);
    test.set_eight(data[1].eight);
    test.set_nine(data[1].nine);
    test.set_ten(data[1].ten);

    test.test_struct.set_one(data[0].one);
    test.test_struct.set_two(data[0].two);
    test.test_struct.set_three(data[0].three);
    test.test_struct.set_four(data[0].four);
    test.test_struct.set_five(data[0].five);
    test.test_struct.set_six(data[0].six);
    test.test_struct.set_seven(data[0].seven);
    test.test_struct.set_eight(data[0].eight);
    test.test_struct.set_nine(data[0].nine);
    test.test_struct.set_ten(data[0].ten);
    test.test_struct.set_f_one(data[0].f_one);
    test.test_struct.set_f_two(data[0].f_two);
    test.test_struct.set_b_one(data[0].b_one);
    let new_test = Test::from_bytes(test.clone().into_bytes());
    assert_eq!(new_test, test);
});

left: `Test { one: 0, two: 0, three: 0, four: 0, five: 0, six: 0, seven: 0, eight: 0, nine: 0, ten: 0, test_struct: TestInner { one: 255, two: -1, three: 65535, four: -1, five: 4294967295, six: -1, seven: 18446744073709551615, eight: -1, nine: 340282366920938463463374607431768211455, ten: -45370982246221608147500238791709434403, f_one: -1.9983972e18, f_two: NaN, b_one: true } }`,
righ: `Test { one: 0, two: 0, three: 0, four: 0, five: 0, six: 0, seven: 0, eight: 0, nine: 0, ten: 0, test_struct: TestInner { one: 255, two: -1, three: 65535, four: -1, five: 4294967295, six: -1, seven: 18446744073709551615, eight: -1, nine: 340282366920938463463374607431768211455, ten: -45370982246221608147500238791709434403, f_one: -1.9983972e18, f_two: NaN, b_one: true } }`',
