#[test]
fn hard_core_test() {
    use bondrewd::Bitfields;
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
fn super_hard_code() {
    use bondrewd::Bitfields;
    use current::{One, ReallyHardcore, Three, Two};
    // TODO add multi-byte nested structure.
    // make it a changing test like the HAL protocol tests do.
    let thing_1 = ReallyHardcore {
        one: One { one: true, two: 7 },
        two: Two::One { one: false, two: 0 },
        three: Three::Invalid { id: 3, other: 127 },
        four: 0,
    };
    let thing_2 = ReallyHardcore {
        one: One { one: false, two: 0 },
        two: Two::Invalid(7, 31),
        three: Three::One(false, false),
        four: 7,
    };

    // let half_bytes_1 = thing_1.two.clone().into_bytes();

    let bytes_1 = thing_1.clone().into_bytes();
    let bytes_2 = thing_2.clone().into_bytes();

    let correct_bytes_1 = [0b0000_1111, 0b1111_0000, 0b00011111];
    // let correct_bytes_1 = [0b1111_0000, 0b0000_1111, 0b11111000];
    assert_eq!(bytes_1, correct_bytes_1);
    assert_eq!(
        bytes_2,
        [
            !correct_bytes_1[0],
            !correct_bytes_1[1],
            !correct_bytes_1[2]
        ]
    );

    let new_1 = ReallyHardcore::from_bytes(bytes_1);
    let new_2 = ReallyHardcore::from_bytes(bytes_2);

    assert_eq!(thing_1, new_1);
    assert_eq!(thing_2, new_2);
}

// fn print_bytes(bytes: &[u8]) {
//     print!("[");
//     for b in bytes {
//         print!("0b{b:08b}, ")
//     }
//     print!("]\n");
// }

mod current {
    use bondrewd::Bitfields;
    #[derive(Bitfields, Default)]
    #[bondrewd(default_endianness = "msb")]
    pub struct Weird {
        #[bondrewd(bit_length = 7)]
        pub one: u16,
    }

    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", bit_traversal = "back")]
    pub struct One {
        pub one: bool,
        #[bondrewd(bit_length = 3)]
        pub two: u8,
    }

    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(
        default_endianness = "be",
        bit_traversal = "back",
        id_bit_length = 3,
        enforce_bytes = 1
    )]
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

    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(
        default_endianness = "be",
        bit_traversal = "back",
        id_bit_length = 2,
        enforce_bits = 9
    )]
    pub enum Three {
        One(bool, bool),
        Two,
        Invalid {
            #[bondrewd(capture_id)]
            id: u8,
            #[bondrewd(bit_length = 7)]
            other: u8,
        },
    }

    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", bit_traversal = "front", reverse)]
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

    impl From<One> for crate::old::One {
        fn from(value: One) -> Self {
            Self {
                one: value.one,
                two: value.two,
            }
        }
    }
}
mod old {
    use bondrewd_old as bondrewd;
    use bondrewd_old::Bitfields;
    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", read_from = "lsb0")]
    pub struct One {
        pub one: bool,
        #[bondrewd(bit_length = 3)]
        pub two: u8,
    }

    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", read_from = "lsb0", enforce_bits = 4)]
    pub struct Three {
        #[bondrewd(bit_length = 2)]
        pub id: u8,
        #[bondrewd(bit_length = 2)]
        pub other: u8,
    }

    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", read_from = "lsb0", enforce_bytes = 1)]
    pub struct Two {
        pub one: bool,
        #[bondrewd(bit_length = 7)]
        pub two: u8,
    }

    #[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", read_from = "lsb0", reverse)]
    pub struct ReallyHardcore {
        #[bondrewd(struct_size = 1, bit_length = 4)]
        pub one: One,
        #[bondrewd(struct_size = 1, bit_length = 8)]
        pub two: Two,
        #[bondrewd(struct_size = 1, bit_length = 9)]
        pub three: Three,
        #[bondrewd(bit_length = 3)]
        pub four: u8,
    }
}
