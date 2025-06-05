#![no_main]
use bondrewd::Bitfields;
// use libfuzzer_sys::fuzz_target;

#[derive(Bitfields, Clone, Debug)]
#[bondrewd(endianness = "be")]
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

impl std::cmp::PartialEq<TestInner> for TestInner {
    fn eq(&self, other: &TestInner) -> bool {
        self.one == other.one
            && self.two == other.two
            && self.three == other.three
            && self.four == other.four
            && self.five == other.five
            && self.six == other.six
            && self.seven == other.seven
            && self.eight == other.eight
            && self.nine == other.nine
            && self.ten == other.ten
            && (self.f_one == other.f_one
                || (self.f_one.is_nan() && other.f_one.is_nan())
                || (self.f_one.is_infinite() && other.f_one.is_infinite()))
            && (self.f_two == other.f_two
                || (self.f_two.is_nan() && other.f_two.is_nan())
                || (self.f_two.is_infinite() && other.f_two.is_infinite()))
            && self.b_one == other.b_one
    }
}

#[derive(Clone, PartialEq, Debug /*, arbitrary::Arbitrary*/)]
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

#[derive(Bitfields, PartialEq, Clone)]
#[bondrewd(id_bit_length = 3, enforce_bits = 363, endianness = "be")]
pub enum TestEnum {
    Zero {
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
        #[bondrewd(bit_length = 105)]
        ten: i128,
    },
    One {
        #[bondrewd(bit_length = 105)]
        ten: i128,
        #[bondrewd(bit_length = 69)]
        nine: u128,
        #[bondrewd(bit_length = 43)]
        eight: i64,
        #[bondrewd(bit_length = 56)]
        seven: u64,
        #[bondrewd(bit_length = 27)]
        six: i32,
        #[bondrewd(bit_length = 30)]
        five: u32,
        #[bondrewd(bit_length = 14)]
        four: i16,
        #[bondrewd(bit_length = 9)]
        three: u16,
        #[bondrewd(bit_length = 4)]
        two: i8,
        #[bondrewd(bit_length = 3)]
        one: u8,
    },
}

impl TestEnum {
    const ID_MASK: u8 = (2 ^ 3) - 1;
    const ONE_MASK: u8 = (2 ^ 3) - 1;
    const TWO_MASK: i8 = (2 ^ 4) - 1;
    const THREE_MASK: u16 = (2 ^ 9) - 1;
    const FOUR_MASK: i16 = (2 ^ 14) - 1;
    const FIVE_MASK: u32 = (2 ^ 30) - 1;
    const SIX_MASK: i32 = (2 ^ 27) - 1;
    const SEVEN_MASK: u64 = (2 ^ 56) - 1;
    const EIGHT_MASK: i64 = (2 ^ 43) - 1;
    const NINE_MASK: u128 = (2 ^ 69) - 1;
    const TEN_MASK: i128 = (2 ^ 105) - 1;
    pub fn one(&self) -> u8 {
        match self {
            Self::One { one, .. } => *one,
            Self::Zero { one, .. } => *one,
        }
    }
    pub fn two(&self) -> i8 {
        match self {
            Self::One { two, .. } => *two,
            Self::Zero { two, .. } => *two,
        }
    }
    pub fn three(&self) -> u16 {
        match self {
            Self::One { three, .. } => *three,
            Self::Zero { three, .. } => *three,
        }
    }
    pub fn four(&self) -> i16 {
        match self {
            Self::One { four, .. } => *four,
            Self::Zero { four, .. } => *four,
        }
    }
    pub fn five(&self) -> u32 {
        match self {
            Self::One { five, .. } => *five,
            Self::Zero { five, .. } => *five,
        }
    }
    pub fn six(&self) -> i32 {
        match self {
            Self::One { six, .. } => *six,
            Self::Zero { six, .. } => *six,
        }
    }
    pub fn seven(&self) -> u64 {
        match self {
            Self::One { seven, .. } => *seven,
            Self::Zero { seven, .. } => *seven,
        }
    }
    pub fn eight(&self) -> i64 {
        match self {
            Self::One { eight, .. } => *eight,
            Self::Zero { eight, .. } => *eight,
        }
    }
    pub fn nine(&self) -> u128 {
        match self {
            Self::One { nine, .. } => *nine,
            Self::Zero { nine, .. } => *nine,
        }
    }
    pub fn ten(&self) -> i128 {
        match self {
            Self::One { ten, .. } => *ten,
            Self::Zero { ten, .. } => *ten,
        }
    }
}

impl TryFrom<&TestInnerArb> for TestEnum {
    type Error = ();

    fn try_from(src: &TestInnerArb) -> Result<Self, Self::Error> {
        if src.one > Self::ONE_MASK
            || src.two > Self::TWO_MASK
            || src.three > Self::THREE_MASK
            || src.four > Self::FOUR_MASK
            || src.five > Self::FIVE_MASK
            || src.six > Self::SIX_MASK
            || src.seven > Self::SEVEN_MASK
            || src.eight > Self::EIGHT_MASK
            || src.nine > Self::NINE_MASK
            || src.ten > Self::TEN_MASK
        {
            Err(())
        } else if src.b_one {
            Ok(Self::One {
                one: src.one,
                two: src.two,
                three: src.three,
                four: src.four,
                five: src.five,
                six: src.six,
                seven: src.seven,
                eight: src.eight,
                nine: src.nine,
                ten: src.ten,
            })
        } else {
            Ok(Self::Zero {
                one: src.one,
                two: src.two,
                three: src.three,
                four: src.four,
                five: src.five,
                six: src.six,
                seven: src.seven,
                eight: src.eight,
                nine: src.nine,
                ten: src.ten,
            })
        }
    }
}

impl Default for TestEnum {
    fn default() -> Self {
        Self::Zero {
            one: 0,
            two: 0,
            three: 0,
            four: 0,
            five: 0,
            six: 0,
            seven: 0,
            eight: 0,
            nine: 0,
            ten: 0,
        }
    }
}

// 593
#[derive(Bitfields, Clone, PartialEq, Debug)]
#[bondrewd(endianness = "be", enforce_bits = 959)]
pub struct Test {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 4)]
    two: i8,
    #[bondrewd(bit_length = 9)] //0
    three: u16,
    #[bondrewd(bit_length = 14)] //2
    four: i16,
    #[bondrewd(bit_length = 30)] //4
    five: u32,
    #[bondrewd(bit_length = 27)] //7
    six: i32,
    #[bondrewd(bit_length = 56)] //
    seven: u64,
    #[bondrewd(bit_length = 43)]
    eight: i64,
    #[bondrewd(bit_length = 69)]
    nine: u128,
    #[bondrewd(bit_length = 111)]
    ten: i128, //366
    #[bondrewd(bit_length = 593)]
    test_struct: TestInner,
}

// START_HERE finish new fuzz test.

// fuzz_target!(|data: TestInnerArb| {
//     // Struct test
//     assert_eq!(959, Test::BIT_SIZE);
//     assert_eq!(120, Test::BYTE_SIZE);
//     let mut test = Test {
//         one: 0,
//         two: 0,
//         three: 0,
//         four: 0,
//         five: 0,
//         six: 0,
//         seven: 0,
//         eight: 0,
//         nine: 0,
//         ten: 0,
//         test_struct: TestInner {
//             one: 0,
//             two: 0,
//             three: 0,
//             four: 0,
//             five: 0,
//             six: 0,
//             seven: 0,
//             eight: 0,
//             nine: 0,
//             ten: 0,
//             f_one: 0.0,
//             f_two: 0.0,
//             b_one: false,
//         },
//     };
//     test.set_one(data.one);
//     test.set_two(data.two);
//     test.set_three(data.three);
//     test.set_four(data.four);
//     test.set_five(data.five);
//     test.set_six(data.six);
//     test.set_seven(data.seven);
//     test.set_eight(data.eight);
//     test.set_nine(data.nine);
//     test.set_ten(data.ten);

//     test.test_struct.set_one(data.one);
//     test.test_struct.set_two(data.two);
//     test.test_struct.set_three(data.three);
//     test.test_struct.set_four(data.four);
//     test.test_struct.set_five(data.five);
//     test.test_struct.set_six(data.six);
//     test.test_struct.set_seven(data.seven);
//     test.test_struct.set_eight(data.eight);
//     test.test_struct.set_nine(data.nine);
//     test.test_struct.set_ten(data.ten);
//     test.test_struct.set_f_one(data.f_one);
//     test.test_struct.set_f_two(data.f_two);
//     test.test_struct.set_b_one(data.b_one);

//     let bytes = test.clone().into_bytes();

//     if let Ok(checked) = Test::check_slice(&bytes) {
//         assert_eq!(checked.read_one(), test.one);
//         assert_eq!(checked.read_two(), test.two);
//         assert_eq!(checked.read_three(), test.three);
//         assert_eq!(checked.read_four(), test.four);
//         assert_eq!(checked.read_five(), test.five);
//         assert_eq!(checked.read_six(), test.six);
//         assert_eq!(checked.read_seven(), test.seven);
//         assert_eq!(checked.read_eight(), test.eight);
//         assert_eq!(checked.read_nine(), test.nine);
//         assert_eq!(checked.read_ten(), test.ten);
//     } else {
//         panic!("checking slice failed");
//     }

//     let new_test = Test::from_bytes(bytes);
//     assert_eq!(new_test, test);

//     // Enum test
//     assert_eq!(366, Test::BIT_SIZE);
//     assert_eq!(46, Test::BYTE_SIZE);
//     let mut test: TestEnum = TestEnum::default();
//     test.set_one(data.one);
//     test.set_two(data.two);
//     test.set_three(data.three);
//     test.set_four(data.four);
//     test.set_five(data.five);
//     test.set_six(data.six);
//     test.set_seven(data.seven);
//     test.set_eight(data.eight);
//     test.set_nine(data.nine);
//     test.set_ten(data.ten);

//     let mut bytes = test.clone().into_bytes();

//     if let Ok(mut checked) = TestEnum::check_slice_mut(&mut bytes) {
//         assert_eq!(checked.read_zero_one(), test.one());
//         assert_eq!(checked.read_zero_two(), test.two());
//         assert_eq!(checked.read_zero_three(), test.three());
//         assert_eq!(checked.read_zero_four(), test.four());
//         assert_eq!(checked.read_zero_five(), test.five());
//         assert_eq!(checked.read_zero_six(), test.six());
//         assert_eq!(checked.read_zero_seven(), test.seven());
//         assert_eq!(checked.read_zero_eight(), test.eight());
//         assert_eq!(checked.read_zero_nine(), test.nine());
//         assert_eq!(checked.read_zero_ten(), test.ten());
//         checked.write_one_one(test.one());
//         checked.write_one_two(test.two());
//         checked.write_one_three(test.three());
//         checked.write_one_four(test.four());
//         checked.write_one_five(test.five());
//         checked.write_one_six(test.six());
//         checked.write_one_seven(test.seven());
//         checked.write_one_eight(test.eight());
//         checked.write_one_nine(test.nine());
//         checked.write_one_ten(test.ten());
//     } else {
//         panic!("checking slice failed");
//     }
//     TestEnum::write_variant_id(&mut bytes, 1);
//     let new_test = TestEnum::from_bytes(bytes);
//     if let (
//         TestEnum::One {
//             one: new_one,
//             two: new_two,
//             three: new_three,
//             four: new_four,
//             five: new_five,
//             six: new_six,
//             seven: new_seven,
//             eight: new_eight,
//             nine: new_nine,
//             ten: new_ten,
//         },
//         TestEnum::Zero {
//             one,
//             two,
//             three,
//             four,
//             five,
//             six,
//             seven,
//             eight,
//             nine,
//             ten,
//         },
//     ) = (new_test, test)
//     {
//         assert_eq!(new_one, one);
//         assert_eq!(new_two, two);
//         assert_eq!(new_three, three);
//         assert_eq!(new_four, four);
//         assert_eq!(new_five, five);
//         assert_eq!(new_six, six);
//         assert_eq!(new_seven, seven);
//         assert_eq!(new_eight, eight);
//         assert_eq!(new_nine, nine);
//         assert_eq!(new_ten, ten);
//     }
// });
