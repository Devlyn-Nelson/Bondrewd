use bondrewd::*;
#[derive(Bitfields, Clone, Debug, PartialEq)]
#[bondrewd(default_endianness = "be")]
struct SimpleExample {
    // fields that are as expected do not require attributes.
    one: bool,
    two: f32,
    #[bondrewd(bit_length = 14)]
    three: i16,
    #[bondrewd(bit_length = 6)]
    four: u8,
}

fn main() {
    let ex = SimpleExample {
        one: true,
        two: 34.0,
        three: 45,
        four: 5,
    };

    let bytes = ex.clone().into_bytes();
    let new_ex = SimpleExample::from_bytes(bytes);
    assert_eq!(new_ex, ex);
}
