use bondrewd::Bitfields;
use current::*;

#[cfg(feature = "dyn_fns")]
#[test]
fn hard_core_test() {
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
    // TODO add multi-byte nested structure.
    // let thing_1 = ReallyHardcore {
    //     one: OneHalf { one: true, two: 7 },
    //     two: OneQuarter::One(false, false),
    //     three: OtherQuarter::One { one: true, two: 7 },
    // };
    let thing_2 = ReallyHardcore {
        one: OneHalf { one: true, two: 7 },
        two: OneQuarter::Invalid {
            id: u8::MAX,
            other: u8::MAX,
        },
        three: OtherQuarter::Invalid(0, 0),
    };
    let thing_1 = ReallyHardcore {
        one: OneHalf { one: true, two: 0 },
        two: OneQuarter::Two,
        three: OtherQuarter::One { one: false, two: 0 },
    };

    let bytes_1 = thing_1.clone().into_bytes();
    let bytes_2 = thing_2.clone().into_bytes();

    // print_bytes(&bytes_1);

    let test_bytes = thing_1.one.clone().into_bytes();
    let old: old::OneHalf = thing_1.one.clone().into();
    let old_bytes = bondrewd_1::Bitfields::into_bytes(old);
    print_bytes(&test_bytes);
    print_bytes(&old_bytes);

    // assert_eq!(bytes_1, [0b0000_1111, 0b1111_1111]);

    let new_1 = ReallyHardcore::from_bytes(bytes_1);
    let new_2 = ReallyHardcore::from_bytes(bytes_2);

    assert_eq!(thing_1, new_1);
    assert_eq!(thing_2, new_2);
}

fn print_bytes(bytes: &[u8]) {
    print!("[");
    for b in bytes {
        print!("0b{b:08b}, ")
    }
    print!("]\n");
}

impl From<OneHalf> for old::OneHalf {
    fn from(value: OneHalf) -> Self {
        Self {
            one: value.one,
            two: value.two,
        }
    }
}

mod current {
    use bondrewd::Bitfields;
    use bondrewd_derive::Bitfields as BitfieldsDerive;

    #[derive(BitfieldsDerive, Default)]
    #[bondrewd(default_endianness = "msb")]
    pub struct Weird {
        #[bondrewd(bit_length = 7)]
        pub one: u16,
    }

    #[derive(BitfieldsDerive, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", bit_traversal = "back")]
    pub struct OneHalf {
        pub one: bool,
        #[bondrewd(bit_length = 3)]
        pub two: u8,
    }

    #[derive(BitfieldsDerive, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(
        default_endianness = "be",
        bit_traversal = "back",
        id_bit_length = 2,
        enforce_bits = 4
    )]
    pub enum OneQuarter {
        One(bool, bool),
        Two,
        Invalid {
            #[bondrewd(capture_id)]
            id: u8,
            #[bondrewd(bit_length = 2)]
            other: u8,
        },
    }

    #[derive(BitfieldsDerive, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(
        default_endianness = "be",
        bit_traversal = "back",
        id_bit_length = 3,
        enforce_bytes = 1
    )]
    pub enum OtherQuarter {
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

    #[derive(BitfieldsDerive, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", bit_traversal = "back", reverse, dump)]
    pub struct ReallyHardcore {
        #[bondrewd(bit_length = 4)]
        pub one: OneHalf,
        #[bondrewd(bit_length = 4)]
        pub two: OneQuarter,
        #[bondrewd(bit_length = 8)]
        pub three: OtherQuarter,
    }
}
mod old {
    use bondrewd_1 as bondrewd;
    use bondrewd_1::Bitfields as OldBitfieldsDerive;

    #[derive(OldBitfieldsDerive, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", read_from = "lsb0")]
    pub struct OneHalf {
        pub one: bool,
        #[bondrewd(bit_length = 3)]
        pub two: u8,
    }

    #[derive(OldBitfieldsDerive, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", read_from = "lsb0", enforce_bits = 4)]
    pub struct OneQuarter {
        #[bondrewd(capture_id, bit_length = 2)]
        pub id: u8,
        #[bondrewd(bit_length = 2)]
        pub other: u8,
    }

    #[derive(OldBitfieldsDerive, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", read_from = "lsb0", enforce_bytes = 1)]
    pub struct OtherQuarter {
        pub one: bool,
        #[bondrewd(bit_length = 7)]
        pub two: u8,
    }

    #[derive(OldBitfieldsDerive, Clone, Copy, Debug, PartialEq, Eq)]
    #[bondrewd(default_endianness = "be", read_from = "lsb0", reverse)]
    pub struct ReallyHardcore {
        #[bondrewd(struct_size = 1, bit_length = 4)]
        pub one: OneHalf,
        #[bondrewd(struct_size = 1, bit_length = 4)]
        pub two: OneQuarter,
        #[bondrewd(struct_size = 1, bit_length = 8)]
        pub three: OtherQuarter,
    }
}
