// #[test]
fn hard_core_test() {
    use bondrewd::{Bitfields, BitfieldsSlice};
    use current::Weird;
    let w = Weird::default();
    let mut bytes = w.into_bytes();
    if let Ok(checked) = Weird::check_slice(&bytes) {
        assert_eq!(checked.read_one(), 0);
    } else {
        panic!("failed size check");
    }
    if let Ok(mut checked) = Weird::check_slice_mut(&mut bytes) {
        checked.write_one(4);
        assert_eq!(checked.read_one(), 4);
    } else {
        panic!("failed size check");
    }
}

#[test]
fn three_hard_core() {
    use bondrewd::Bitfields;
    use current::{Three, ThreeAlt};
    assert_eq!(Three::BYTE_SIZE, 2);
    // assert_eq!(Three::BIT_SIZE, 9);
    assert_eq!(ThreeAlt::BYTE_SIZE, 2);
    // assert_eq!(ThreeAlt::BIT_SIZE, 9);
    let three = Three::Four(0);
    let three_alt = ThreeAlt {
        id: 0,
        data: 0b01111111,
    };
    let three_bytes = three.clone().into_bytes();
    let three_alt_bytes = three_alt.clone().into_bytes();
    print!("struct: ");
    print_bytes(&three_bytes);
    print!("enum  : ");
    print_bytes(&three_alt_bytes);
    assert_eq!(three_bytes, three_alt_bytes);
    assert_eq!(three_bytes, [0xFF, 0b1000_0000]);
}
// #[test]
fn super_hard_code() {
    use bondrewd::Bitfields;
    use current::{One, ReallyHardcore, Three, Two};
    // assert_eq!(Three::BIT_SIZE, 16);
    // assert_eq!(Three::BYTE_SIZE, 2);
    assert_eq!(ReallyHardcore::BYTE_SIZE, 3);
    assert_eq!(ReallyHardcore::BIT_SIZE, 24);
    let zero = ReallyHardcore {
        one: One { one: false, two: 0 },
        two: Two::One { one: false, two: 0 },
        three: Three::First(false, false),
        four: 0,
    };
    let mut test_field_three = zero.clone().into_bytes();
    let three = Three::Four(0);
    let test_three = three.clone().into_bytes();
    print_bytes(&test_three);
    // assert_eq!(test_three, [0xFF, 0b1000_0000]);
    assert_eq!(test_three, [0xFF, 0b1000_0000]);
    ReallyHardcore::write_three(&mut test_field_three, three);
    //
    // assert_eq!(test_field_three, [0, 0b11110000, 0b00011111 ]);
    // let thing_1 = ReallyHardcore {
    //     one: One { one: true, two: 7 },
    //     two: Two::One { one: false, two: 0 },
    //     three: Three::Invalid { id: 3, other: 127 },
    //     four: 0,
    // };
    // let thing_2 = ReallyHardcore {
    //     one: One { one: false, two: 0 },
    //     two: Two::Invalid(7, 31),
    //     three: Three::First(false, false),
    //     four: 7,
    // };

    // let bytes_1 = thing_1.clone().into_bytes();
    // let bytes_2 = thing_2.clone().into_bytes();
    // let mut test_field_two = zero.clone().into_bytes();
    // TESTS
    // print_bytes(&test_field_three);
    // assert_eq!(ReallyHardcore::read_three(&mut bytes_zero), three);
    // let two = Two::full();
    // let test_two = two.clone().into_bytes();
    // ReallyHardcore::write_two(&mut test_field_two, two);
    // print_bytes(&test_two);
    // print_bytes(&test_field_two);
    // assert_eq!(ReallyHardcore::read_two(&mut bytes_zero), two);

    // assert_eq!(bytes_zero, [0b00000000, 0b00000000,0b00000000]);
    //
    // let half_bytes_1 = thing_1.two.clone().into_bytes();
    //
    // let correct_bytes_1 = [0b0000_1111, 0b1111_0000, 0b00011111];
    // let correct_bytes_1 = [0b1111_0000, 0b0000_1111, 0b11111000];
    // assert_eq!(bytes_1, correct_bytes_1);
    // assert_eq!(
    //     bytes_2,
    //     [
    //         !correct_bytes_1[0],
    //         !correct_bytes_1[1],
    //         !correct_bytes_1[2]
    //     ]
    // );

    // let new_1 = ReallyHardcore::from_bytes(bytes_1);
    // let new_2 = ReallyHardcore::from_bytes(bytes_2);

    // assert_eq!(thing_1, new_1);
    // assert_eq!(thing_2, new_2);
    // TODO finish this test.
}

fn print_bytes(bytes: &[u8]) {
    print!("[");
    for b in bytes {
        print!("0b{b:08b}, ")
    }
    print!("]\n");
}

mod current {
    use bondrewd::{Bitfields, BitfieldsSlice};
    #[derive(Bitfields, BitfieldsSlice, Default)]
    #[bondrewd(endianness = "ale")]
    pub struct Weird {
        #[bondrewd(bit_length = 7)]
        pub one: u16,
    }

    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(endianness = "ale")]
    pub struct One {
        pub one: bool,
        #[bondrewd(bit_length = 3)]
        pub two: u8,
    }

    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(endianness = "ale", id_bit_length = 3, enforce_bytes = 1)]
    pub enum Two {
        One {
            one: bool,
            #[bondrewd(bit_length = 4)]
            two: u8,
        },
        Two(#[bondrewd(bit_length = 4)] u8),
        Three {
            one: bool,
            #[bondrewd(bit_length = 3)]
            two: u8,
            three: bool,
        },
        Invalid(#[bondrewd(capture_id)] u8, #[bondrewd(bit_length = 5)] u8),
    }
    impl Two {
        pub fn full() -> Self {
            Self::Invalid(7, 31)
        }
    }

    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(endianness = "ale", fill_bits)]
    pub struct ThreeAlt {
        #[bondrewd(bit_length = 2)]
        pub id: u8,
        #[bondrewd(bit_length = 7)]
        pub data: u8,
    }

    /// START_HERE when dumping this code, it is obvious that the wrong bits are used, even the comments
    /// used the wrong bits. the ThreeAlt above also gets the incorrect bits, which maybe be easier to
    /// look at than the enum, and they likely suffer from the same issue.
    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(endianness = "ale", id_bit_length = 2, fill_bits)]
    pub enum Three {
        First(bool, bool),
        Second,
        #[bondrewd(invalid)]
        Invalid {
            #[bondrewd(capture_id)]
            id: u8,
            #[bondrewd(bit_length = 7)]
            other: u8,
        },
        Four(#[bondrewd(bit_length = 7)] u8),
    }
    impl Three {
        pub fn full() -> Self {
            Self::Four(0b01111111)
        }
    }

    // 22221111 33332222 44433333
    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(endianness = "ale", enforce_bytes = 3)]
    pub struct ReallyHardcore {
        #[bondrewd(bit_length = 4)]
        pub one: One,
        #[bondrewd(bit_length = 8)]
        pub two: Two,
        #[bondrewd(bit_length = 9)]
        pub three: Three,
        #[bondrewd(bit_length = 3)]
        pub four: u8,
    }
}
