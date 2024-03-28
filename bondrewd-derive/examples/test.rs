use bondrewd::Bitfields;

#[derive(Bitfields)]
#[bondrewd(default_endianness = "be")]
struct Simple {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 27)]
    two: char,
    #[bondrewd(bit_length = 14)]
    three: u16,
    four: i8,
}
#[derive(Bitfields)]
#[bondrewd(default_endianness = "be")]
struct SimpleWithStruct {
    #[bondrewd(byte_length = 7)]
    one: Simple,
    // structs can also be used in arrays.
    #[bondrewd(element_byte_length = 7)]
    two: [Simple; 2],
}

fn main() {}
