use std::{u16, u32};

use bondrewd::{Bitfields, BitfieldsSlice};

#[derive(Bitfields, Debug, PartialEq, Eq, Clone)]
#[bondrewd(endianness = "ale", fill_bits)]
struct Aligned {
    #[bondrewd(bit_length = 9)]
    number: u16,
}

#[test]
fn aligned() {
    assert_eq!(Aligned::BIT_SIZE, 16);
    assert_eq!(Aligned::BYTE_SIZE, 2);
    let ex = Aligned { number: u16::MAX };
    let bytes = ex.clone().into_bytes();
    assert_eq!(bytes, [0b11111111, 0b00000001]);
    // let new = Aligned::from_bytes(bytes);
    // assert_eq!(new, ex);
    let ex = Aligned { number: 1 };
    let bytes = ex.clone().into_bytes();
    assert_eq!(bytes, [0b00000001, 0b00000000]);
    // let new = Aligned::from_bytes(bytes);
    // assert_eq!(new, ex);
}

// the original ale problem was structs passing in total bit length for the
// structure's fields instead of the total bits used (including fill) because
// thats what flip is based on. Need same test with enum type.

#[derive(Bitfields, Debug, PartialEq, Eq, Clone)]
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
    let bytes = ex.clone().into_bytes();
    assert_eq!(bytes[0], 0b1010_0000);
    assert_eq!(bytes[1], 0b1111_1111);
    // let new = AlignedEnum::from_bytes(bytes);
    // assert_eq!(new, ex);
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

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
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

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
struct BigNestedBugInducer {
    #[bondrewd(bit_length = 11)]
    one: u16,
    #[bondrewd(bit_length = 29)]
    two: BigBugInducer,
    three: u8,
}
#[test]
fn bug_of_my_nightmares_but_bigger() -> anyhow::Result<()> {
    let mut simple = BigNestedBugInducer {
        one: 0b0000_0101,
        two: BigBugInducer {
            one: true,
            two: true,
            three: 0,
        },
        three: 76,
    };

    let small_bytes = simple.two.clone().into_bytes();
    assert_eq!(
        &small_bytes,
        &[0b0001_1000, 0b0000_0000, 0b0000_0000, 0b0000_0000]
    );
    assert_eq!(simple.two, BigBugInducer::from_bytes(small_bytes));

    assert_eq!(BigNestedBugInducer::BYTE_SIZE, 6);
    let mut bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 6);
    assert_eq!(bytes[0], 0b0000_0101);
    assert_eq!(bytes[1], 0b0001_1000);
    assert_eq!(bytes[2], 0b0000_0000);
    assert_eq!(bytes[3], 0b0000_0000);
    assert_eq!(bytes[4], 0b0000_0000);
    assert_eq!(bytes[5], 0b0100_1100);

    //peeks
    assert_eq!(simple.one, BigNestedBugInducer::read_slice_one(&bytes)?);
    assert_eq!(simple.two, BigNestedBugInducer::read_slice_two(&bytes)?);
    assert_eq!(simple.three, BigNestedBugInducer::read_slice_three(&bytes)?);

    // from_bytes
    let new_simple = BigNestedBugInducer::from_bytes(bytes);
    assert_eq!(simple, new_simple);

    //write
    simple.one = 76;
    simple.two.one = false;
    simple.two.three = 666;
    simple.three = 165;
    BigNestedBugInducer::write_slice_one(&mut bytes, simple.one)?;
    BigNestedBugInducer::write_slice_two(&mut bytes, simple.two.clone())?;
    BigNestedBugInducer::write_slice_three(&mut bytes, simple.three)?;

    //peeks
    assert_eq!(simple.one, BigNestedBugInducer::read_slice_one(&bytes)?);
    assert_eq!(simple.two, BigNestedBugInducer::read_slice_two(&bytes)?);
    assert_eq!(simple.three, BigNestedBugInducer::read_slice_three(&bytes)?);

    // from_bytes
    let new_simple = BigNestedBugInducer::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
struct AlignedBugInducer {
    one: bool,
    two: bool,
    #[bondrewd(bit_length = 30)]
    three: u32,
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
struct AlignedNestedBugInducer {
    one: u16,
    #[bondrewd(byte_length = 4)]
    two: AlignedBugInducer,
    three: u8,
}
#[test]
fn bug_of_my_nightmares_but_aligned() -> anyhow::Result<()> {
    let mut simple = AlignedNestedBugInducer {
        one: 0b0000_0101,
        two: AlignedBugInducer {
            one: true,
            two: true,
            three: 0,
        },
        three: 76,
    };

    let small_bytes = simple.two.clone().into_bytes();
    assert_eq!(
        &small_bytes,
        &[0b0000_0011, 0b0000_0000, 0b0000_0000, 0b0000_0000]
    );
    assert_eq!(simple.two, AlignedBugInducer::from_bytes(small_bytes));

    assert_eq!(AlignedNestedBugInducer::BYTE_SIZE, 7);
    let mut bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 7);
    assert_eq!(bytes[0], 0b0000_0101);
    assert_eq!(bytes[1], 0b0000_0000);
    assert_eq!(bytes[2], 0b0000_0011);
    assert_eq!(bytes[3], 0b0000_0000);
    assert_eq!(bytes[4], 0b0000_0000);
    assert_eq!(bytes[5], 0b0000_0000);
    assert_eq!(bytes[6], 0b0100_1100);

    //peeks
    assert_eq!(simple.one, AlignedNestedBugInducer::read_slice_one(&bytes)?);
    assert_eq!(simple.two, AlignedNestedBugInducer::read_slice_two(&bytes)?);
    assert_eq!(
        simple.three,
        AlignedNestedBugInducer::read_slice_three(&bytes)?
    );

    // from_bytes
    let new_simple = AlignedNestedBugInducer::from_bytes(bytes);
    assert_eq!(simple, new_simple);

    //write
    simple.one = 76;
    simple.two.one = false;
    simple.two.three = 666;
    simple.three = 165;
    AlignedNestedBugInducer::write_slice_one(&mut bytes, simple.one)?;
    AlignedNestedBugInducer::write_slice_two(&mut bytes, simple.two.clone())?;
    AlignedNestedBugInducer::write_slice_three(&mut bytes, simple.three)?;

    //peeks
    assert_eq!(simple.one, AlignedNestedBugInducer::read_slice_one(&bytes)?);
    assert_eq!(simple.two, AlignedNestedBugInducer::read_slice_two(&bytes)?);
    assert_eq!(
        simple.three,
        AlignedNestedBugInducer::read_slice_three(&bytes)?
    );

    // from_bytes
    let new_simple = AlignedNestedBugInducer::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
struct DocTest {
    #[bondrewd(bit_length = 4)]
    one: u8,
    two: u32,
    three: u16,
    four: u8,
    #[bondrewd(bit_length = 4)]
    five: u8,
    #[bondrewd(bit_length = 15)]
    six: u16,
    #[bondrewd(bit_length = 9)]
    seven: u16,
    eight: u16,
    nine: u8,
    #[bondrewd(bit_length = 9)]
    ten: u16,
    #[bondrewd(bit_length = 15)]
    eleven: u16,
}

// #[test]
// fn asdf() {
//     let bytes = DocTest {
//         one: 0,
//         two: 0,
//         three: 0,
//         four: 0,
//         five: 0,
//         six: 0,
//         seven: 0,
//         eight: 0,
//         nine: 0,
//         ten: u16::MAX,
//         eleven: 0,
//     }
//     .into_bytes();
//     print!("[");
//     for b in bytes {
//         print!("0b{b:08b}, ")
//     }
//     print!("]\n");
// }
