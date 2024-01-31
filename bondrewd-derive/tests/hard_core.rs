use bondrewd_derive::Bitfields as BitfieldsDerive;
use bondrewd::Bitfields;

#[derive(BitfieldsDerive)]
struct Weird {
    #[bondrewd(bit_length = 7)]
    one: u16,
}