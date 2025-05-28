use bondrewd::Bitfields;

#[derive(Bitfields)]
#[bondrewd(endianness = "ale")]
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
}

// the original ale problem was structs passing in total bit length for the
// structure's fields instead of the total bits used (including fill) because
// thats what flip is based on. Need same test with enum type.

#[derive(Bitfields)]
#[bondrewd(endianness = "ale", id_bit_length = 2)]
enum AlignedEnum {
    #[bondrewd(id = 0)]
    Thing {
        #[bondrewd(bit_length = 9)]
        number: u16,
    }
}
#[test]
fn aligned_enum() {
    assert_eq!(AlignedEnum::BIT_SIZE, 11);
    assert_eq!(AlignedEnum::BYTE_SIZE, 2);
    let ex = AlignedEnum::Thing{ number: u16::MAX };
    let bytes = ex.into_bytes();
    assert_eq!(bytes, [0b11111100, 0b0000111]);
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be", bit_traversal = "back")]
struct BugInducer {
    one: bool,
    two: bool,
    #[bondrewd(bit_length = 3)]
    three: u8,
}

// TODO this causes a zero bit shift
// TODO one and two are effecting the same bits
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale", dump)]
struct NestedBugInducer {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 5)]
    two: BugInducer,
    three: u8,
}
#[test]
fn bug_of_my_nightmares() -> anyhow::Result<()> {
    let small = BugInducer {
        one: true,
        two: false,
        three: 0,
    };
    let simple = NestedBugInducer {
        one: 0b00000111,
        two: small,
        three: 0,
    };
    assert_eq!(NestedBugInducer::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);
    println!("{:08b}", bytes[0]);
    assert_eq!(bytes[0], 0b0000_0000);
    assert_eq!(bytes[1], 0b0000_0000);

    //peeks
    assert_eq!(
        simple.one,
        NestedBugInducer::read_slice_one(&bytes)?
    );
    assert_eq!(
        simple.two,
        NestedBugInducer::read_slice_two(&bytes)?
    );
    assert_eq!(
        simple.three,
        NestedBugInducer::read_slice_three(&bytes)?
    );

    // from_bytes
    let new_simple = NestedBugInducer::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
