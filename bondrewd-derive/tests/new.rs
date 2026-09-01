use bondrewd::{Bitfields, BitfieldsDyn};

#[derive(Bitfields)]
pub enum Test {
    One {
        #[bondrewd(bit_length = 5)]
        one: u8,
    },
    Two {
        one: bool,
        #[bondrewd(bit_length = 4)]
        two: u8,
    },
    Three(#[bondrewd(bit_length = 5)] u8),
    Invalid(#[bondrewd(capture_id)] u8),
}

#[test]
fn test_fn() {
    let test = Test::One { one: 1 }.into_bytes();
    assert_eq!(test, [2]);
    let test = Test::Two { one: false, two: 4 }.into_bytes();
    assert_eq!(test, [0b_0100_1000]);
}

#[derive(Bitfields, BitfieldsDyn)]
#[bondrewd(endianness = "be", fill_bits = 3, enforce_bits = 14)]
struct FilledBytesEnforced {
    #[bondrewd(bit_length = 7)]
    one: u8,
    #[bondrewd(bit_length = 7)]
    two: u8,
}

#[test]
fn fill_test() {
    let mut input = vec![0, 0, 0, 0xFF];
    let thing = FilledBytesEnforced::from_vec(&mut input);
    // we are enforcing 14 bits but fill_bytes is creating
    // an imaginary reserve field from bit index 14 to
    // index 23
    assert_eq!(17, FilledBytesEnforced::BIT_SIZE);
    assert_eq!(3, FilledBytesEnforced::BYTE_SIZE);
    assert_eq!(input.len(), 1);
    assert_eq!(input[0], 0xFF);
    assert!(thing.is_ok());
    let thing = thing.unwrap();
    assert_eq!(thing.one, 0);
    assert_eq!(thing.two, 0);
}
