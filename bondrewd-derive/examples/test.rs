use bondrewd::*;

#[derive(Bitfields, Clone)]
#[bondrewd(default_endianness = "be")]
struct Simple {
    #[bondrewd(bit_length = 4)]
    one: u8,
    two: bool,
    #[bondrewd(bit_length = 3)]
    three: u8,
}

fn main() -> anyhow::Result<()> {
    let test = Simple {
        one: 2,
        two: true,
        three: 1,
    };
    let bytes = test.into_bytes();
    if let Ok(checked_struct) = Simple::check_slice(&bytes){
        assert_eq!(checked_struct.one(), 2);
        assert_eq!(checked_struct.two(), true);
        assert_eq!(checked_struct.three(), 1);
    }else{
        panic!("check failed");
    };
    assert_eq!(bytes, [0b0010_1001]);
    Ok(())
}