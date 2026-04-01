use bondrewd::{Bitfields, BitfieldsSlice};

fn print_bytes(bytes: &[u8]) {
    print!("[");
    for b in bytes {
        print!("0b{b:08b}, ")
    }
    print!("]\n");
}


#[test]
fn three_hard_core() {
    assert_eq!(HCThree::BYTE_SIZE, 2);
    // assert_eq!(Three::BIT_SIZE, 9);
    assert_eq!(ThreeAlt::BYTE_SIZE, 2);
    // assert_eq!(ThreeAlt::BIT_SIZE, 9);
    let three = HCThree::Three(0b01111111);
    let three_alt = ThreeAlt {
        id: 3,
        data: 0b01111111,
    };
    let three_bytes = three.clone().into_bytes();
    let three_alt_bytes = three_alt.clone().into_bytes();
    print!("struct: ");
    print_bytes(&three_bytes);
    print!("enum  : ");
    print_bytes(&three_alt_bytes);
    assert_eq!(three_bytes, three_alt_bytes);
    assert_eq!(three_bytes, [0xFF, 0b0000_0001]);
}

#[test]
fn super_hard_code() {
    assert_eq!(ReallyHardcore::BYTE_SIZE, 3);
    assert_eq!(ReallyHardcore::BIT_SIZE, 24);
    // Start with all zeros output
    let rhc_zero = ReallyHardcore {
        one: HCOne { one: false, two: 0 },
        two: HCTwo::Zero { one: false, two: 0 },
        three: HCThree::Zero(false, false),
        four: 0,
    };

    // quick check that it did in fact write all zeros.
    assert_eq!(rhc_zero.clone().into_bytes(), [0, 0, 0]);

    // Test field `four`
    {
        // write full four field to hard core struct then test that it reads the same as it was written.
        let mut rhc_test_field_four_bytes = rhc_zero.clone().into_bytes();
        ReallyHardcore::write_four(&mut rhc_test_field_four_bytes, 7);
        assert_eq!(ReallyHardcore::read_four(&mut rhc_test_field_four_bytes), 7);
        assert_eq!(rhc_test_field_four_bytes, [0, 0, 0b1110_0000 ]);
    }

    // Test field `three`
    {
        // make clone for writing the three field over.
        let mut rhc_test_field_three_bytes = rhc_zero.clone().into_bytes();
        // make and test a three field where id and fields are all 1's, but since not all of the struct is used there
        // should still be some zeros.
        let new_three = HCThree::full();
        let new_three_bytes = new_three.clone().into_bytes();
        assert_eq!(new_three_bytes, [0xFF, 0b0000_0001]);

        // write full three field to hard core struct then test that it reads the same as it was written.
        ReallyHardcore::write_three(&mut rhc_test_field_three_bytes, new_three);
        let correct_three_field_bytes = [0, 0b11110000, 0b00011111];
        print!("test   : ");
        print_bytes(&rhc_test_field_three_bytes);
        print!("correct: ");
        print_bytes(&correct_three_field_bytes);
        assert_eq!(rhc_test_field_three_bytes, correct_three_field_bytes);
        assert_eq!(ReallyHardcore::read_three(&mut rhc_test_field_three_bytes), new_three);
    }

    // create 2 struct with opposite bytes.
    let thing_1 = ReallyHardcore {
        one: HCOne { one: true, two: 7 },
        two: HCTwo::Zero { one: false, two: 0 },
        three: HCThree::TwoAndInvalid { id: 3, other: 127 },
        four: 0,
    };
    let thing_2 = ReallyHardcore {
        one: HCOne { one: false, two: 0 },
        two: HCTwo::ThreeAndInvalid(7, 31),
        three: HCThree::Zero(false, false),
        four: 7,
    };

    let bytes_1 = thing_1.clone().into_bytes();
    let bytes_2 = thing_2.clone().into_bytes();
    let mut rhc_test_2_bytes = rhc_zero.clone().into_bytes();
    
    let two = HCTwo::full();
    let test_two = two.clone().into_bytes();
    ReallyHardcore::write_two(&mut rhc_test_2_bytes, two);
    assert_eq!(ReallyHardcore::read_two(&mut rhc_test_2_bytes), two);
    
    let half_bytes_1 = thing_1.two.clone().into_bytes();
    
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

    let new_1 = ReallyHardcore::from_bytes(bytes_1);
    let new_2 = ReallyHardcore::from_bytes(bytes_2);

    assert_eq!(thing_1, new_1);
    assert_eq!(thing_2, new_2);
    // TODO finish this test.
}

#[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(endianness = "ale")]
pub struct HCOne {
    pub one: bool,
    #[bondrewd(bit_length = 3)]
    pub two: u8,
}

#[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(endianness = "ale", id_bit_length = 3, enforce_bytes = 1)]
pub enum HCTwo {
    Zero {
        one: bool,
        #[bondrewd(bit_length = 4)]
        two: u8,
    },
    One(#[bondrewd(bit_length = 4)] u8),
    Two {
        one: bool,
        #[bondrewd(bit_length = 3)]
        two: u8,
        three: bool,
    },
    ThreeAndInvalid(#[bondrewd(capture_id)] u8, #[bondrewd(bit_length = 5)] u8),
}
impl HCTwo {
    pub fn full() -> Self {
        Self::ThreeAndInvalid(7, 31)
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
pub enum HCThree {
    Zero(bool, bool),
    One,
    #[bondrewd(invalid)]
    TwoAndInvalid {
        #[bondrewd(capture_id)]
        id: u8,
        #[bondrewd(bit_length = 7)]
        other: u8,
    },
    Three(#[bondrewd(bit_length = 7)] u8),
}
impl HCThree {
    pub fn full() -> Self {
        Self::Three(0b01111111)
    }
}

// 22221111 33332222 44433333
// TODO changing this to be and using the reverse attribute causes panic
#[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(endianness = "ale", enforce_bytes = 3, dump)]
pub struct ReallyHardcore {
    #[bondrewd(bit_length = 4)]
    pub one: HCOne,
    #[bondrewd(bit_length = 8)]
    pub two: HCTwo,
    #[bondrewd(bit_length = 9)]
    pub three: HCThree,
    #[bondrewd(bit_length = 3)]
    pub four: u8,
}