use bondrewd::Bitfields;
use rand::{Rng, RngCore};

#[derive(bondrewd_derive::Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(bit_traversal = "back", reverse)]
pub struct Mixed {
    #[bondrewd(bit_length = 20, endianness = "le")]
    pub one: u32,
    #[bondrewd(bit_length = 29, endianness = "ale")]
    pub two: u32,
    #[bondrewd(bit_length = 23, endianness = "be")]
    pub three: u32,
}

#[test]
fn mixed() {
    let mut r = rand::thread_rng();
    let max = 2u32.pow(24) - 1;
    let one: u32 = r.gen_range(0..=2u32.pow(20) - 1);
    let two: u32 = r.gen_range(0..=2u32.pow(29) - 1);
    let three: u32 = r.gen_range(0..=2u32.pow(23) - 1);
    println!("{one}");
    let correct = Mixed { one, two, three };

    let bytes = correct.clone().into_bytes();

    let mut out = String::new();
    for byte in bytes {
        out.push_str(&format!("{byte:08b}, "))
    }

    assert_eq!(Mixed::from_bytes(bytes), correct)
}
