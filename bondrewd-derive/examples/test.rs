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
    let mut bytes = test.into_bytes();
    if let Ok(checked_struct) = Simple::check_slice(&bytes){
        assert_eq!(checked_struct.read_one(), 2);
        assert_eq!(checked_struct.read_two(), true);
        assert_eq!(checked_struct.read_three(), 1);
    }else{
        panic!("check failed");
    };
    assert_eq!(bytes, [0b0010_1001]);
    if let Ok(mut checked_struct) = Simple::check_slice_mut(&mut bytes){
        checked_struct.write_one(4);
        checked_struct.write_two(false);
        checked_struct.write_three(2);
        assert_eq!(checked_struct.read_one(), 4);
        assert_eq!(checked_struct.read_two(), false);
        assert_eq!(checked_struct.read_three(), 2);
    }else{
        panic!("mut check failed");
    };
    assert_eq!(bytes, [0b0100_0010]);
    Ok(())
}