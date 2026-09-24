use bondrewd::Bitfields;

fn print_bytes(bytes: &[u8]) {
    print!("[");
    for b in bytes {
        print!("0b{b:08b}, ")
    }
    print!("]\n");
}

// #[test]
fn super_hard_code() {
    // TODO fix problems with this test.
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

    // Test field `one`
    {
        let mut rhc_test_field_one_bytes = rhc_zero.clone().into_bytes();
        let new_one = HCOne::full();
        let new_one_bytes = new_one.clone().into_bytes();
        assert_eq!(new_one_bytes, [0xF0]);

        ReallyHardcore::write_one(&mut rhc_test_field_one_bytes, new_one);
        let correct_one_field_bytes = [0b00001111, 0, 0];
        assert_eq!(rhc_test_field_one_bytes, correct_one_field_bytes);
        assert_eq!(
            ReallyHardcore::read_one(&mut rhc_test_field_one_bytes),
            new_one
        );
    }

    // Test field `two`
    {
        let mut rhc_test_field_two_bytes = rhc_zero.clone().into_bytes();
        let new_two = HCTwo::full();
        let new_two_bytes = new_two.clone().into_bytes();
        assert_eq!(new_two_bytes, [0xFF]);

        ReallyHardcore::write_two(&mut rhc_test_field_two_bytes, new_two);
        let correct_two_field_bytes = [0b11110000, 0b00001111, 0];
        assert_eq!(rhc_test_field_two_bytes, correct_two_field_bytes);
        assert_eq!(
            ReallyHardcore::read_two(&mut rhc_test_field_two_bytes),
            new_two
        );
    }

    // Test field `three`
    {
        // make clone for writing the three field over.
        let mut rhc_test_field_three_bytes = rhc_zero.clone().into_bytes();
        // make and test a three field where id and fields are all 1's, but since not all of the struct is used there
        // should still be some zeros.
        let new_three = HCThree::full();
        let new_three_bytes = new_three.clone().into_bytes();
        // assert_eq!(new_three_bytes, [0xFF, 0b0000_0001]);
        assert_eq!(new_three_bytes, [0b1000_0000, 0xFF]);

        // write full three field to hard core struct then test that it reads the same as it was written.
        ReallyHardcore::write_three(&mut rhc_test_field_three_bytes, new_three);
        let correct_three_field_bytes = [0, 0b11110000, 0b00011111];
        print!("test   : ");
        print_bytes(&rhc_test_field_three_bytes);
        print!("correct: ");
        print_bytes(&correct_three_field_bytes);
        // TODO this assert fails because the nested derive function maker doesn't know the alignment
        // of bytes. for example field `one` only works when `fill_bits` isn't used, which outputs the
        // same bytes as any other 4-bit structure. if `fill_bits` is used the 4 bits will be aligned
        // to the right (0b0000_1111) because the fill will be the second field and `ale` endianness
        // reverses the field order. you can also see above the assert_eq for `new_three_bytes`
        // has a commented out version of it directly above with is the `fill_bits` version.
        // with fill bits the bits that are un-used are still considered part if the bit fields
        // where-as with out fill bits they are not considered part of the struct and cause the
        // bits to be in a different location because the reversal of field order no longer considers
        // those bits as part of the bit field. then due to `reverse` bytes, which is also assumed when
        // using `ale` for endianess, the unused bytes show up in the first byte.
        assert_eq!(rhc_test_field_three_bytes, correct_three_field_bytes);
        assert_eq!(
            ReallyHardcore::read_three(&mut rhc_test_field_three_bytes),
            new_three
        );
    }

    // Test field `four`
    {
        // write full four field to hard core struct then test that it reads the same as it was written.
        let mut rhc_test_field_four_bytes = rhc_zero.clone().into_bytes();
        ReallyHardcore::write_four(&mut rhc_test_field_four_bytes, 7);
        assert_eq!(ReallyHardcore::read_four(&mut rhc_test_field_four_bytes), 7);
        assert_eq!(rhc_test_field_four_bytes, [0, 0, 0b1110_0000]);
    }

    // create 2 structs with opposite bytes.
    let thing_1 = ReallyHardcore {
        one: HCOne::full(),
        two: HCTwo::empty(),
        three: HCThree::full(),
        four: 0,
    };
    let thing_2 = ReallyHardcore {
        one: HCOne::empty(),
        two: HCTwo::full(),
        three: HCThree::empty(),
        four: 7,
    };

    let bytes_1 = thing_1.clone().into_bytes();
    let bytes_2 = thing_2.clone().into_bytes();

    let correct_bytes_1 = [0b0000_1111, 0b1111_0000, 0b00000111];
    let correct_bytes_2 = [0b1111_0000, 0b0000_1111, 0b11111000];
    assert_eq!(bytes_1, correct_bytes_1);
    assert_eq!(bytes_2, correct_bytes_2);

    let new_1 = ReallyHardcore::from_bytes(bytes_1);
    let new_2 = ReallyHardcore::from_bytes(bytes_2);

    assert_eq!(thing_1, new_1);
    assert_eq!(thing_2, new_2);
}

#[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(endianness = "ale")]
pub struct HCOne {
    pub one: bool,
    #[bondrewd(bit_length = 3)]
    pub two: u8,
}

impl HCOne {
    pub fn full() -> Self {
        Self { one: true, two: 7 }
    }
    pub fn empty() -> Self {
        Self { one: false, two: 0 }
    }
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
    pub fn empty() -> Self {
        Self::Zero { one: false, two: 0 }
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
#[bondrewd(endianness = "ale", id_bit_length = 2)]
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
    pub fn empty() -> Self {
        Self::Zero(false, false)
    }
}

// 22221111 33332222 44433333
// TODO changing this to be and using the reverse attribute causes panic
#[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(endianness = "ale", enforce_bytes = 3, fill_bits, dump)]
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
