use bondrewd::Bitfields;
use bondrewd_derive::Bitfields as BitfieldsDerive;

#[derive(BitfieldsDerive, Default)]
#[bondrewd(default_endianness = "msb")]
struct Weird {
    #[bondrewd(bit_length = 7)]
    one: u16,
}

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

#[derive(BitfieldsDerive, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(default_endianness = "be", bit_traversal = "back")]
struct OneHalf {
    one: bool,
    #[bondrewd(bit_length = 3)]
    two: u8,
}

#[derive(BitfieldsDerive, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(
    default_endianness = "be",
    bit_traversal = "back",
    id_bit_length = 2,
    enforce_bits = 4
)]
enum OneQuarter {
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
enum OtherQuarter {
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
#[bondrewd(default_endianness = "be", bit_traversal = "back", reverse)]
struct ReallyHardcore {
    #[bondrewd(bit_length = 4)]
    one: OneHalf,
    #[bondrewd(bit_length = 4)]
    two: OneQuarter,
    #[bondrewd(bit_length = 8)]
    three: OtherQuarter,
}

fn print_bytes(bytes: &[u8]) {
    print!("[");
    for b in bytes {
        print!("0b{b:08b}, ")
    }
    print!("]\n");
}

#[test]
fn super_hard_code() {
    let thing_1 = ReallyHardcore {
        one: OneHalf { one: true, two: 7 },
        two: OneQuarter::One(false, false),
        three: OtherQuarter::Invalid(3, 7),
    };
    let thing_2 = ReallyHardcore {
        one: OneHalf { one: false, two: 0 },
        two: OneQuarter::Invalid {
            id: u8::MAX,
            other: u8::MAX,
        },
        three: OtherQuarter::Invalid(0, 0),
    };

    let bytes_1 = thing_1.clone().into_bytes();
    let bytes_2 = thing_2.clone().into_bytes();

    print_bytes(&bytes_1);
    print_bytes(&bytes_2);

    // assert_eq!(bytes_1, [0b0000_1111, 0b1111_1111]);

    let new_1 = ReallyHardcore::from_bytes(bytes_1);

    assert_eq!(thing_1, new_1);
}
