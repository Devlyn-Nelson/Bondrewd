use bondrewd::Bitfields;
#[derive(Bitfields, PartialEq)]
#[bondrewd(default_endianness = "le", id_bit_length = 3, enforce_bits = 366)]
pub enum TestEnum {
    Zero {
        #[bondrewd(bit_length = 3)]
        one: u8,
        #[bondrewd(bit_length = 4)]
        two: i8,
        #[bondrewd(bit_length = 9)] //0
        three: u16,
        #[bondrewd(bit_length = 14)] //2
        four: i16,
        #[bondrewd(bit_length = 30)] //4
        five: u32,
        #[bondrewd(bit_length = 27)] //7
        six: i32,
        #[bondrewd(bit_length = 56)] //
        seven: u64,
        #[bondrewd(bit_length = 43)]
        eight: i64,
        #[bondrewd(bit_length = 69)]
        nine: u128,
        #[bondrewd(bit_length = 105)]
        ten: i128,
    },
    One {
        #[bondrewd(bit_length = 105)]
        ten: i128,
        #[bondrewd(bit_length = 69)]
        nine: u128,
        #[bondrewd(bit_length = 43)]
        eight: i64,
        #[bondrewd(bit_length = 56)] //
        seven: u64,
        #[bondrewd(bit_length = 27)] //7
        six: i32,
        #[bondrewd(bit_length = 30)] //4
        five: u32,
        #[bondrewd(bit_length = 14)] //2
        four: i16,
        #[bondrewd(bit_length = 9)] //0
        three: u16,
        #[bondrewd(bit_length = 4)]
        two: i8,
        #[bondrewd(bit_length = 3)]
        one: u8,
    },
}
fn main() {}
