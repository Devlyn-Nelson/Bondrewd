use bondrewd::Bitfields;

fn print_bytes(bytes: &[u8]) {
    print!("[");
    for b in bytes {
        print!("0b{b:08b}, ")
    }
    print!("]\n");
}

/// TODO test without fill_bits
#[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(endianness = "ale", fill_bits, dump)]
pub struct MyStruct {
    #[bondrewd(bit_length = 2)]
    pub id: u8,
    #[bondrewd(bit_length = 7)]
    pub data: u8,
}

/// START_HERE when dumping this code, it is obvious that the wrong bits are used, even the comments
/// used the wrong bits. the MyStruct above also gets the incorrect bits, which maybe be easier to
/// look at than the enum, and they likely suffer from the same issue.
#[derive(Bitfields, Clone, Copy, Debug, PartialEq, Eq)]
#[bondrewd(endianness = "ale", id_bit_length = 2, fill_bits, dump)]
pub enum MyEnum {
    Zero(#[bondrewd(bit_length = 7)] u8),
    One(
        #[bondrewd(bit_length = 7)] u8,
    ),
    #[bondrewd(invalid)]
    Invalid {
        #[bondrewd(capture_id)]
        id: u8,
        #[bondrewd(bit_length = 7)]
        other: u8,
    },
    Three(#[bondrewd(bit_length = 7)] u8),
}

#[test]
fn enum_vs_struct_id_0() {
    assert_eq!(MyEnum::BYTE_SIZE, 2);
    assert_eq!(MyStruct::BYTE_SIZE, 2);
    let my_struct = MyEnum::Zero(0b00111111);
    let my_enum = MyStruct {
        id: 0,
        data: 0b00111111,
    };
    let my_struct_bytes = my_struct.clone().into_bytes();
    let my_enum_bytes = my_enum.clone().into_bytes();
    print!("struct: ");
    print_bytes(&my_struct_bytes);
    print!("enum  : ");
    print_bytes(&my_enum_bytes);
    assert_eq!(my_struct_bytes, my_enum_bytes);
    assert_eq!(my_struct_bytes, [0b11111100, 0b00000000]);
}

#[test]
fn enum_vs_struct_id_1() {
    assert_eq!(MyEnum::BYTE_SIZE, 2);
    assert_eq!(MyStruct::BYTE_SIZE, 2);
    let my_struct = MyEnum::One(0b0100_0001);
    let my_enum = MyStruct {
        id: 1,
        data: 0b0100_0001,
    };
    let my_struct_bytes = my_struct.clone().into_bytes();
    let my_enum_bytes = my_enum.clone().into_bytes();
    print!("struct: ");
    print_bytes(&my_struct_bytes);
    print!("enum  : ");
    print_bytes(&my_enum_bytes);
    assert_eq!(my_struct_bytes, my_enum_bytes);
    assert_eq!(my_struct_bytes, [0b00000101, 0b00000001]);
}

#[test]
fn enum_vs_struct_id_3() {
    assert_eq!(MyEnum::BYTE_SIZE, 2);
    assert_eq!(MyStruct::BYTE_SIZE, 2);
    let my_struct = MyEnum::Three(0);
    let my_enum = MyStruct {
        id: 3,
        data: 0,
    };
    let my_struct_bytes = my_struct.clone().into_bytes();
    let my_enum_bytes = my_enum.clone().into_bytes();
    print!("struct: ");
    print_bytes(&my_struct_bytes);
    print!("enum  : ");
    print_bytes(&my_enum_bytes);
    assert_eq!(my_struct_bytes, my_enum_bytes);
    assert_eq!(my_struct_bytes, [0xFF, 0b1000_0000]);
}
