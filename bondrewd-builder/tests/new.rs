use bondrewd::Bitfields;
use bondrewd_builder::Bitfields as BitfieldsDerive;
use bondrewd_test as bondrewd;

// START_HERE start fixing.
#[derive(BitfieldsDerive)]
#[bondrewd(dump)]
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
    assert_eq!(test, [1]);
}
