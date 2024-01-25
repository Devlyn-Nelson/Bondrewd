#[cfg(feature = "setters")]
mod getter_setter_tests {
    use bondrewd::Bitfields;
    use rand::random;

    #[derive(Bitfields, Clone, Debug)]
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

    #[derive(Bitfields, Clone, PartialEq, Debug)]
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
        ten: i128, //366
        #[bondrewd(bit_length = 593)]
        test_struct: TestInner,
    }

    #[derive(Clone, PartialEq, Debug)]
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

    impl TestInnerArb {
        pub fn rand() -> Self {
            Self {
                one: random(),
                two: random(),
                three: random(),
                four: random(),
                five: random(),
                six: random(),
                seven: random(),
                eight: random(),
                nine: random(),
                ten: random(),
                f_one: random(),
                f_two: random(),
                b_one: random(),
            }
        }
    }

    #[test]
    fn setters_and_getters_basic() {
        let data = [TestInnerArb::rand(), TestInnerArb::rand()];
        assert_eq!(959, Test::BIT_SIZE);
        assert_eq!(120, Test::BYTE_SIZE);
        let mut test = Test {
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
            test_struct: TestInner {
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
        let bytes = test.clone().into_bytes();

        if let Ok(checked) = Test::check_slice(&bytes) {
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

        let new_test = Test::from_bytes(bytes);
        assert_eq!(new_test.one(), test.one);
        assert_eq!(new_test.two(), test.two);
        assert_eq!(new_test.three(), test.three);
        assert_eq!(new_test.four(), test.four);
        assert_eq!(new_test.five(), test.five);
        assert_eq!(new_test.six(), test.six);
        assert_eq!(new_test.seven(), test.seven);
        assert_eq!(new_test.eight(), test.eight);
        assert_eq!(new_test.nine(), test.nine);
        assert_eq!(new_test.ten(), test.ten);
        assert_eq!(new_test, test);
    }
}
