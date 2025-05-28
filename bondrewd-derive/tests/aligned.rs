use bondrewd::Bitfields;

#[derive(Bitfields)]
#[bondrewd(endianness = "ale", fill_bits)]
struct Aligned {
    #[bondrewd(bit_length = 9)]
    number: u16,
}

#[test]
fn aligned() {
    assert_eq!(Aligned::BIT_SIZE, 9);
    assert_eq!(Aligned::BYTE_SIZE, 2);
    let ex = Aligned { number: u16::MAX };
    let bytes = ex.into_bytes();
    assert_eq!(bytes, [0b11111111, 0b00000001]);
    let ex = Aligned { number: 1 };
    let bytes = ex.into_bytes();
    assert_eq!(bytes, [0b00000001, 0b00000000]);
}

// the original ale problem was structs passing in total bit length for the
// structure's fields instead of the total bits used (including fill) because
// thats what flip is based on. Need same test with enum type.

#[derive(Bitfields)]
#[bondrewd(endianness = "ale", id_bit_length = 2)]
enum AlignedEnum {
    #[bondrewd(id = 1)]
    Thing {
        #[bondrewd(bit_length = 9)]
        number: u16,
    },
}
#[test]
fn aligned_enum() {
    assert_eq!(AlignedEnum::BIT_SIZE, 11);
    assert_eq!(AlignedEnum::BYTE_SIZE, 2);
    let ex = AlignedEnum::Thing { number: u16::MAX };
    let bytes = ex.into_bytes();
    assert_eq!(bytes[0], 0b1010_0000);
    assert_eq!(bytes[1], 0b1111_1111);
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
struct BugInducer {
    one: bool,
    two: bool,
    #[bondrewd(bit_length = 3)]
    three: u8,
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
struct NestedBugInducer {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 5)]
    two: BugInducer,
    three: u8,
}
// Maybe flip needs to not used total byte count, but rather the total amount of bits used. currently
#[test]
fn bug_of_my_nightmares() -> anyhow::Result<()> {
    let small = BugInducer {
        one: true,
        two: true,
        three: 0,
    };
    let simple = NestedBugInducer {
        one: 0b0000_0101,
        two: small.clone(),
        three: 76,
    };

    let small_bytes = small.clone().into_bytes();
    assert_eq!(&small_bytes, &[0b0001_1000]);
    assert_eq!(small, BugInducer::from_bytes(small_bytes));

    assert_eq!(NestedBugInducer::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes[0], 0b0001_1101);
    assert_eq!(bytes[1], 0b0100_1100);

    //peeks
    assert_eq!(simple.one, NestedBugInducer::read_slice_one(&bytes)?);
    assert_eq!(simple.two, NestedBugInducer::read_slice_two(&bytes)?);
    assert_eq!(simple.three, NestedBugInducer::read_slice_three(&bytes)?);

    // from_bytes
    let new_simple = NestedBugInducer::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}



#[allow(clippy::struct_excessive_bools)]
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
struct BigBugInducer {
    one: bool,
    two: bool,
    #[bondrewd(bit_length = 27)]
    three: u32,
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
struct BigNestedBugInducer {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 29)]
    two: BigBugInducer,
    three: u8,
}
// Maybe flip needs to not used total byte count, but rather the total amount of bits used. currently
#[test]
fn bug_of_my_nightmares_but_bigger() -> anyhow::Result<()> {
    let small = BigBugInducer {
        one: true,
        two: true,
        three: 0,
    };
    let simple = BigNestedBugInducer {
        one: 0b0000_0101,
        two: small.clone(),
        three: 76,
    };

    let small_bytes = small.clone().into_bytes();
    assert_eq!(&small_bytes, &[0b0001_1000, 0b0000_0000, 0b0000_0000, 0b0000_0000]);
    assert_eq!(small, BigBugInducer::from_bytes(small_bytes));

    assert_eq!(BigNestedBugInducer::BYTE_SIZE, 5);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 5);
    assert_eq!(bytes[0], 0b0000_0101);
    assert_eq!(bytes[1], 0b0000_0000);
    assert_eq!(bytes[2], 0b0000_0000);
    assert_eq!(bytes[3], 0b0001_1000);
    assert_eq!(bytes[4], 0b0100_1100);

    //peeks
    assert_eq!(simple.one, BigNestedBugInducer::read_slice_one(&bytes)?);
    assert_eq!(simple.two, BigNestedBugInducer::read_slice_two(&bytes)?);
    assert_eq!(simple.three, BigNestedBugInducer::read_slice_three(&bytes)?);

    // from_bytes
    let new_simple = BigNestedBugInducer::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}