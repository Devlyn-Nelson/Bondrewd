use bondrewd::Bitfields;
use bondrewd_derive::Bitfields as BitfieldsDerive;

#[derive(BitfieldsDerive)]
struct Weird {
    #[bondrewd(bit_length = 7)]
    one: u16,
}
