use bondrewd::Bitfields;
use bondrewd_derive::Bitfields as BitfieldsDerive;

#[derive(BitfieldsDerive, Default)]
struct Weird {
    #[bondrewd(bit_length = 7)]
    one: u16,
}

#[test]
fn hard_core_test() {
    let w = Weird::default();
    let bytes = w.into_bytes();
    if let Ok(checked) = Weird::check_slice(&bytes) {
        assert!(checked.read_one());
    } else {
    }
}
