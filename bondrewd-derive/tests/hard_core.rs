use bondrewd::Bitfields;
use bondrewd_derive::Bitfields as BitfieldsDerive;

#[derive(BitfieldsDerive, Default)]
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
