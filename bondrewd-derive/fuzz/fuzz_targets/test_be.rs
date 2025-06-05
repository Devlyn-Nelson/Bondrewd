#![no_main]
use bondrewd::{Bitfields, BitfieldsSlice};
use libfuzzer_sys::fuzz_target;

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

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Debug)]
#[bondrewd(endianness = "be", enforce_bits = 959, dump)]
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
    ten: i128, //366
    #[bondrewd(bit_length = 593)]
    test_struct: TestInner,
}

impl Test {
    const ONE_MAX: u8 = 2_u8.pow(3);
    const TWO_MAX: i8 = 2_i8.pow(4- 1);
    const THREE_MAX: u16 = 2_u16.pow(9);
    const FOUR_MAX: i16 = 2_i16.pow(14- 1);
    const FIVE_MAX: u32 = 2_u32.pow(30);
    const SIX_MAX: i32 = 2_i32.pow(27- 1);
    const SEVEN_MAX: u64 = 2_u64.pow(56);
    const EIGHT_MAX: i64 = 2_i64.pow(43- 1);
    const NINE_MAX: u128 = 2_u128.pow(69);
    const TEN_MAX: i128 = 2_i128.pow(111- 1);

    pub fn fix(&self) -> Self {
        Self {
            one: self.one.clamp(0, Self::ONE_MAX - 1),
            two: self.two.clamp(-Self::TWO_MAX, Self::TWO_MAX - 1),
            three: self.three.clamp(0, Self::THREE_MAX - 1),
            four: self.four.clamp(-Self::FOUR_MAX, Self::FOUR_MAX - 1),
            five: self.five.clamp(0, Self::FIVE_MAX - 1),
            six: self.six.clamp(-Self::SIX_MAX, Self::SIX_MAX - 1),
            seven: self.seven.clamp(0, Self::SEVEN_MAX - 1),
            eight: self.eight.clamp(-Self::EIGHT_MAX, Self::EIGHT_MAX - 1),
            nine: self.nine.clamp(0, Self::NINE_MAX - 1),
            ten: self.ten.clamp(-Self::TEN_MAX, Self::TEN_MAX - 1),
            test_struct: self.test_struct.clone(),
        }
    }
}

impl From<&TestInnerArb> for Test {
    fn from(src: &TestInnerArb) -> Self {
        Self {
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
            test_struct: TestInner {
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
                f_one: src.f_one,
                f_two: src.f_two,
                b_one: src.b_one,
            },
        }
    }
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

#[derive(Bitfields, BitfieldsSlice, PartialEq, Clone, Debug)]
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
    const ONE_MAX: u8 = 2_u8.pow(3);
    const TWO_MAX: i8 = 2_i8.pow(4-1);
    const THREE_MAX: u16 = 2_u16.pow(9);
    const FOUR_MAX: i16 = 2_i16.pow(14-1);
    const FIVE_MAX: u32 = 2_u32.pow(30);
    const SIX_MAX: i32 = 2_i32.pow(27-1);
    const SEVEN_MAX: u64 = 2_u64.pow(56);
    const EIGHT_MAX: i64 = 2_i64.pow(43-1);
    const NINE_MAX: u128 = 2_u128.pow(69);
    const TEN_MAX: i128 = 2_i128.pow(105-1);
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
    pub fn reverse(&self) -> Self {
        match self {
            TestEnum::Zero {
                one,
                two,
                three,
                four,
                five,
                six,
                seven,
                eight,
                nine,
                ten,
            } => TestEnum::One {
                ten: *ten,
                nine: *nine,
                eight: *eight,
                seven: *seven,
                six: *six,
                five: *five,
                four: *four,
                three: *three,
                two: *two,
                one: *one,
            },
            TestEnum::One {
                ten,
                nine,
                eight,
                seven,
                six,
                five,
                four,
                three,
                two,
                one,
            } => TestEnum::Zero {
                one: *one,
                two: *two,
                three: *three,
                four: *four,
                five: *five,
                six: *six,
                seven: *seven,
                eight: *eight,
                nine: *nine,
                ten: *ten,
            },
        }
    }
    pub fn fix(&mut self) -> Self {
        match self {
            TestEnum::Zero {
                one,
                two,
                three,
                four,
                five,
                six,
                seven,
                eight,
                nine,
                ten,
            } => {
                let one = (*one).clamp(0, Self::ONE_MAX - 1);
                let two = (*two).clamp(-Self::TWO_MAX, Self::TWO_MAX - 1);
                let three = (*three).clamp(0, Self::THREE_MAX - 1);
                let four = (*four).clamp(-Self::FOUR_MAX, Self::FOUR_MAX - 1);
                let five = (*five).clamp(0, Self::FIVE_MAX - 1);
                let six = (*six).clamp(-Self::SIX_MAX, Self::SIX_MAX - 1);
                let seven = (*seven).clamp(0, Self::SEVEN_MAX - 1);
                let eight = (*eight).clamp(-Self::EIGHT_MAX, Self::EIGHT_MAX - 1);
                let nine = (*nine).clamp(0, Self::NINE_MAX - 1);
                let ten = (*ten).clamp(-Self::TEN_MAX, Self::TEN_MAX - 1);
                TestEnum::Zero {
                    one,
                    two,
                    three,
                    four,
                    five,
                    six,
                    seven,
                    eight,
                    nine,
                    ten,
                }
            },
            TestEnum::One {
                ten,
                nine,
                eight,
                seven,
                six,
                five,
                four,
                three,
                two,
                one,
            } => {
                let one = (*one).clamp(0, Self::ONE_MAX - 1);
                let two = (*two).clamp(-Self::TWO_MAX, Self::TWO_MAX - 1);
                let three = (*three).clamp(0, Self::THREE_MAX - 1);
                let four = (*four).clamp(-Self::FOUR_MAX, Self::FOUR_MAX - 1);
                let five = (*five).clamp(0, Self::FIVE_MAX - 1);
                let six = (*six).clamp(-Self::SIX_MAX, Self::SIX_MAX - 1);
                let seven = (*seven).clamp(0, Self::SEVEN_MAX - 1);
                let eight = (*eight).clamp(-Self::EIGHT_MAX, Self::EIGHT_MAX - 1);
                let nine = (*nine).clamp(0, Self::NINE_MAX - 1);
                let ten = (*ten).clamp(-Self::TEN_MAX, Self::TEN_MAX - 1);
                TestEnum::One {
                    one,
                    two,
                    three,
                    four,
                    five,
                    six,
                    seven,
                    eight,
                    nine,
                    ten,
                }
            },
        }
    }
}

impl From<&TestInnerArb> for TestEnum {
    fn from(src: &TestInnerArb) -> Self {
        if src.b_one {
            Self::One {
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
            }
        } else {
            Self::Zero {
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
            }
        }
    }
}

fuzz_target!(|input: TestInnerArb| {
    // Struct test
    assert_eq!(959, Test::BIT_SIZE);
    assert_eq!(120, Test::BYTE_SIZE);
    let test = Into::<Test>::into(&input).fix();
    let test_bytes = test.clone().into_bytes();
    // let test = test.fix();
    if let Ok(checked) = Test::check_slice(&test_bytes) {
        assert_eq!(checked.read_one(), test.one);
        assert_eq!(checked.read_two(), test.two);
        assert_eq!(checked.read_three(), test.three);
        assert_eq!(checked.read_four(), test.four);
        assert_eq!(checked.read_five(), test.five);
        assert_eq!(checked.read_six(), test.six);
        assert_eq!(checked.read_seven(), test.seven);
        assert_eq!(checked.read_eight(), test.eight);
        assert_eq!(checked.read_nine(), test.nine);
        assert_eq!(checked.read_ten(), test.ten);
    } else {
        panic!("checking slice failed");
    }

    let new_test = Test::from_bytes(test_bytes);
    assert_eq!(new_test, test);
    
    // Enum test
    assert_eq!(363, TestEnum::BIT_SIZE);
    assert_eq!(46, TestEnum::BYTE_SIZE);
    let test = Into::<TestEnum>::into(&input).fix();
    let mut bytes = test.clone().into_bytes();
    if let Ok(thing) = TestEnum::check_slice_mut(&mut bytes) {
        if input.b_one {
            let TestEnumCheckedMut::One(mut checked) = thing else {
                panic!("incorrect variant during check slice")
            };
            assert_eq!(checked.read_one(), test.one());
            assert_eq!(checked.read_two(), test.two());
            assert_eq!(checked.read_three(), test.three());
            assert_eq!(checked.read_four(), test.four());
            assert_eq!(checked.read_five(), test.five());
            assert_eq!(checked.read_six(), test.six());
            assert_eq!(checked.read_seven(), test.seven());
            assert_eq!(checked.read_eight(), test.eight());
            assert_eq!(checked.read_nine(), test.nine());
            assert_eq!(checked.read_ten(), test.ten());
            checked.write_one(0);
            checked.write_two(0);
            checked.write_three(0);
            checked.write_four(0);
            checked.write_five(0);
            checked.write_six(0);
            checked.write_seven(0);
            checked.write_eight(0);
            checked.write_nine(0);
            checked.write_ten(0);
        }else{
            let TestEnumCheckedMut::Zero(mut checked) = thing else {
                panic!("incorrect variant during check slice")
            };
            assert_eq!(checked.read_one(), test.one());
            assert_eq!(checked.read_two(), test.two());
            assert_eq!(checked.read_three(), test.three());
            assert_eq!(checked.read_four(), test.four());
            assert_eq!(checked.read_five(), test.five());
            assert_eq!(checked.read_six(), test.six());
            assert_eq!(checked.read_seven(), test.seven());
            assert_eq!(checked.read_eight(), test.eight());
            assert_eq!(checked.read_nine(), test.nine());
            assert_eq!(checked.read_ten(), test.ten());
            checked.write_one(0);
            checked.write_two(0);
            checked.write_three(0);
            checked.write_four(0);
            checked.write_five(0);
            checked.write_six(0);
            checked.write_seven(0);
            checked.write_eight(0);
            checked.write_nine(0);
            checked.write_ten(0);
        }
    } else {
        panic!("checking slice failed");
    }
    TestEnum::write_variant_id(&mut bytes, 0);
    assert_eq!(bytes, [0;TestEnum::BYTE_SIZE]);
    if input.b_one {
        // check that the zeros were written
        assert_eq!(TestEnum::read_variant_id(&bytes), 0);
        assert_eq!(TestEnum::read_one_one(&bytes), 0);
        assert_eq!(TestEnum::read_one_two(&bytes), 0);
        assert_eq!(TestEnum::read_one_three(&bytes), 0);
        assert_eq!(TestEnum::read_one_four(&bytes), 0);
        assert_eq!(TestEnum::read_one_five(&bytes), 0);
        assert_eq!(TestEnum::read_one_six(&bytes), 0);
        assert_eq!(TestEnum::read_one_seven(&bytes), 0);
        assert_eq!(TestEnum::read_one_eight(&bytes), 0);
        assert_eq!(TestEnum::read_one_nine(&bytes), 0);
        assert_eq!(TestEnum::read_one_ten(&bytes), 0);
        // write alt data (reverse of the original)
        TestEnum::write_zero_one(&mut bytes, test.one());
        TestEnum::write_zero_two(&mut bytes, test.two());
        TestEnum::write_zero_three(&mut bytes, test.three());
        TestEnum::write_zero_four(&mut bytes, test.four());
        TestEnum::write_zero_five(&mut bytes, test.five());
        TestEnum::write_zero_six(&mut bytes, test.six());
        TestEnum::write_zero_seven(&mut bytes, test.seven());
        TestEnum::write_zero_eight(&mut bytes, test.eight());
        TestEnum::write_zero_nine(&mut bytes, test.nine());
        TestEnum::write_zero_ten(&mut bytes, test.ten());
        // read back alt data.
        assert_eq!(TestEnum::read_variant_id(&bytes), 0);
        assert_eq!(TestEnum::read_zero_one(&bytes), test.one());
        assert_eq!(TestEnum::read_zero_two(&bytes), test.two());
        assert_eq!(TestEnum::read_zero_three(&bytes), test.three());
        assert_eq!(TestEnum::read_zero_four(&bytes), test.four());
        assert_eq!(TestEnum::read_zero_five(&bytes), test.five());
        assert_eq!(TestEnum::read_zero_six(&bytes), test.six());
        assert_eq!(TestEnum::read_zero_seven(&bytes), test.seven());
        assert_eq!(TestEnum::read_zero_eight(&bytes), test.eight());
        assert_eq!(TestEnum::read_zero_nine(&bytes), test.nine());
        assert_eq!(TestEnum::read_zero_ten(&bytes), test.ten());
    }else{
        // check that the zeros were written
        assert_eq!(TestEnum::read_variant_id(&bytes), 0);
        assert_eq!(TestEnum::read_zero_one(&bytes), 0);
        assert_eq!(TestEnum::read_zero_two(&bytes), 0);
        assert_eq!(TestEnum::read_zero_three(&bytes), 0);
        assert_eq!(TestEnum::read_zero_four(&bytes), 0);
        assert_eq!(TestEnum::read_zero_five(&bytes), 0);
        assert_eq!(TestEnum::read_zero_six(&bytes), 0);
        assert_eq!(TestEnum::read_zero_seven(&bytes), 0);
        assert_eq!(TestEnum::read_zero_eight(&bytes), 0);
        assert_eq!(TestEnum::read_zero_nine(&bytes), 0);
        assert_eq!(TestEnum::read_zero_ten(&bytes), 0);
        // write alt data (reverse of the original)
        TestEnum::write_variant_id(&mut bytes, 1);
        TestEnum::write_one_one(&mut bytes, test.one());
        TestEnum::write_one_two(&mut bytes, test.two());
        TestEnum::write_one_three(&mut bytes, test.three());
        TestEnum::write_one_four(&mut bytes, test.four());
        TestEnum::write_one_five(&mut bytes, test.five());
        TestEnum::write_one_six(&mut bytes, test.six());
        TestEnum::write_one_seven(&mut bytes, test.seven());
        TestEnum::write_one_eight(&mut bytes, test.eight());
        TestEnum::write_one_nine(&mut bytes, test.nine());
        TestEnum::write_one_ten(&mut bytes, test.ten());
        // read back alt data.
        assert_eq!(TestEnum::read_variant_id(&bytes), 1);
        assert_eq!(TestEnum::read_one_one(&bytes), test.one());
        assert_eq!(TestEnum::read_one_two(&bytes), test.two());
        assert_eq!(TestEnum::read_one_three(&bytes), test.three());
        assert_eq!(TestEnum::read_one_four(&bytes), test.four());
        assert_eq!(TestEnum::read_one_five(&bytes), test.five());
        assert_eq!(TestEnum::read_one_six(&bytes), test.six());
        assert_eq!(TestEnum::read_one_seven(&bytes), test.seven());
        assert_eq!(TestEnum::read_one_eight(&bytes), test.eight());
        assert_eq!(TestEnum::read_one_nine(&bytes), test.nine());
        assert_eq!(TestEnum::read_one_ten(&bytes), test.ten());
    }
    let new_test = TestEnum::from_bytes(bytes);
    assert_eq!(new_test, test.reverse());
});
