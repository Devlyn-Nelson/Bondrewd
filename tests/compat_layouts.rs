//! Byte layouts copied from bondrewd's own test-suite, to prove wire compatibility.
#![allow(clippy::unusual_byte_groupings)]

use bondrewd::{Bitfields, BitfieldsSlice};

macro_rules! simple {
    ($name:ident, $endian:literal) => {
        #[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
        #[bondrewd(endianness = $endian, enforce_bits = 52)]
        struct $name {
            #[bondrewd(bit_length = 3)]
            one: u8,
            #[bondrewd(bit_length = 27)]
            two: u32,
            #[bondrewd(bit_length = 14)]
            three: u16,
            four: u8,
        }
        impl $name {
            fn sample() -> Self {
                Self { one: 2, two: 6345, three: 2145, four: 66 }
            }
        }
    };
}
simple!(SimpleBe, "be");
simple!(SimpleLe, "le");
simple!(SimpleAle, "ale");

macro_rules! check_simple {
    ($ty:ident, $expected:expr) => {{
        let s = $ty::sample();
        assert_eq!($ty::BYTE_SIZE, 7);
        assert_eq!($ty::BIT_SIZE, 52);
        let bytes = s.clone().into_bytes();
        assert_eq!(bytes, $expected);
        assert_eq!($ty::read_one(&bytes), s.one);
        assert_eq!($ty::read_two(&bytes), s.two);
        assert_eq!($ty::read_three(&bytes), s.three);
        assert_eq!($ty::read_four(&bytes), s.four);
        assert_eq!($ty::read_slice_two(&bytes).unwrap(), s.two);
        assert_eq!($ty::from_bytes(bytes), s);
    }};
}

#[test]
fn be_simple() {
    check_simple!(
        SimpleBe,
        [0b010_00000, 0b0000_0000, 0b0110_0011, 0b0010_0100, 0b1000_0110, 0b0001_0100, 0b0010_0000]
    );
}

#[test]
fn le_simple() {
    check_simple!(
        SimpleLe,
        [0b010_11001, 0b0010_0011, 0b0000_0000, 0b0000_0001, 0b1000_0100, 0b1000_0100, 0b0010_0000]
    );
}

#[test]
fn ale_simple() {
    check_simple!(
        SimpleAle,
        [0b1010_0000, 0b0110_0100, 0b0000_1100, 0b0000_0000, 0b1000_0100, 0b0010_0001, 0b0100_0010]
    );
}

macro_rules! flip {
    ($name:ident, $($attr:tt)*) => {
        #[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
        #[bondrewd($($attr)*)]
        struct $name {
            one: bool,
            #[bondrewd(bit_length = 10)]
            two: u16,
            #[bondrewd(bit_length = 5)]
            three: u8,
        }
        impl $name {
            fn check(expected: [u8; 2]) {
                let s = Self { one: false, two: 0x3FF, three: 0 };
                let bytes = s.clone().into_bytes();
                assert_eq!(bytes, expected, "{}", stringify!($name));
                assert_eq!(Self::read_two(&bytes), 0x3FF);
                assert_eq!(Self::from_bytes(bytes), s);
            }
        }
    };
}
flip!(BeReverse, endianness = "be", reverse);
flip!(LeReverse, endianness = "le", reverse);
flip!(LeBack, endianness = "le", bit_traversal = "back");
flip!(AleReverse, endianness = "ale", reverse);
flip!(AleBack, endianness = "ale", bit_traversal = "back");

#[test]
fn reverse_and_back() {
    BeReverse::check([0b1110_0000, 0b0111_1111]);
    LeReverse::check([0b1110_0000, 0b0111_1111]);
    LeBack::check([0b0000_0111, 0b1111_1110]);
    AleReverse::check([0b0000_0111, 0b1111_1110]);
    AleBack::check([0b1110_0000, 0b0111_1111]);
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
struct AleReserve {
    #[bondrewd(bit_length = 9)]
    one: u16,
    #[bondrewd(bit_length = 3, reserve)]
    reserve: u8,
    #[bondrewd(bit_length = 4)]
    two: i8,
}

#[test]
fn ale_reserve_and_sign_extension() {
    let mut s = AleReserve { one: 341, reserve: u8::MAX, two: -1 };
    let mut bytes = s.clone().into_bytes();
    assert_eq!(bytes, [0b0101_0101, 0b1111_0001]);
    assert_eq!(AleReserve::read_two(&bytes), -1);
    assert_eq!(AleReserve::read_reserve(&bytes), 0);
    AleReserve::write_slice_reserve(&mut bytes, 7).unwrap();
    AleReserve::write_one(&mut bytes, 0);
    AleReserve::write_two(&mut bytes, 0);
    assert_eq!(AleReserve::read_reserve(&bytes), 7);
    s.one = 0;
    s.two = 0;
    s.reserve = 0; // reserve fields are never unpacked by from_bytes
    assert_eq!(AleReserve::from_bytes(bytes), s);
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
#[allow(clippy::struct_excessive_bools)]
struct Duplicate {
    one: bool,
    two: bool,
    three: bool,
    four: bool,
    five: bool,
    six: bool,
    seven: bool,
    eight: bool,
    #[bondrewd(bits = "0..8", redundant)]
    dup: u8,
    nine: u8,
}

#[test]
fn redundant_fields() {
    let bytes = [0b1000_0001, 0xFF];
    let d = Duplicate::from_bytes(bytes);
    assert!(d.one && d.eight && !d.two);
    assert_eq!(d.dup, 0b1000_0001);
    assert_eq!(d.nine, 0xFF);
    assert_eq!(Duplicate::BYTE_SIZE, 2);
}

#[derive(Bitfields, Debug, PartialEq, Eq, Clone)]
#[bondrewd(endianness = "ale", fill_bits)]
struct AlignedFill {
    #[bondrewd(bit_length = 9)]
    number: u16,
}

#[derive(Bitfields, Debug, PartialEq, Eq, Clone)]
#[bondrewd(endianness = "ale", id_bit_length = 2, fill_bits)]
enum AlignedEnum {
    #[bondrewd(id = 1)]
    Thing {
        #[bondrewd(bit_length = 9)]
        number: u16,
    },
    #[bondrewd(id = 0)]
    Thing2 {
        #[bondrewd(bit_length = 9)]
        number: u16,
    },
}

#[derive(Bitfields, Debug, PartialEq, Eq, Clone)]
#[bondrewd(endianness = "ale", id_bit_length = 2)]
enum AlignedEnumNoFill {
    #[bondrewd(id = 1)]
    Thing {
        #[bondrewd(bit_length = 9)]
        number: u16,
    },
    #[bondrewd(id = 0)]
    Thing2 {
        #[bondrewd(bit_length = 9)]
        number: u16,
    },
}

#[test]
fn ale_fill_and_enums() {
    assert_eq!(AlignedFill::BIT_SIZE, 16);
    assert_eq!(AlignedFill { number: u16::MAX }.into_bytes(), [0xFF, 0x01]);
    assert_eq!(AlignedFill { number: 1 }.into_bytes(), [0x01, 0x00]);

    assert_eq!(AlignedEnum::BIT_SIZE, 16);
    let a = AlignedEnum::Thing { number: 0x1FF };
    assert_eq!(a.clone().into_bytes(), [0b1111_1101, 0b0000_0111]);
    assert_eq!(AlignedEnum::from_bytes(a.clone().into_bytes()), a);
    let b = AlignedEnum::Thing2 { number: 0xFE };
    assert_eq!(b.clone().into_bytes(), [0b1111_1000, 0b0000_0011]);
    assert_eq!(AlignedEnum::from_bytes(b.clone().into_bytes()), b);

    assert_eq!(AlignedEnumNoFill::BIT_SIZE, 11);
    let a = AlignedEnumNoFill::Thing { number: 0x1FF };
    assert_eq!(a.clone().into_bytes(), [0b1010_0000, 0b1111_1111]);
    assert_eq!(AlignedEnumNoFill::from_bytes(a.clone().into_bytes()), a);
    let b = AlignedEnumNoFill::Thing2 { number: 0xFE };
    assert_eq!(b.clone().into_bytes(), [0b0000_0000, 0b0111_1111]);
    assert_eq!(AlignedEnumNoFill::from_bytes(b.clone().into_bytes()), b);
}

// The nested ALE structures that bondrewd struggled with.
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

#[test]
fn nested_ale_small() {
    let small = BugInducer { one: true, two: true, three: 0 };
    assert_eq!(small.clone().into_bytes(), [0b0001_1000]);
    let s = NestedBugInducer { one: 5, two: small, three: 76 };
    let bytes = s.clone().into_bytes();
    assert_eq!(bytes, [0b0001_1101, 0b0100_1100]);
    assert_eq!(NestedBugInducer::read_two(&bytes), s.two);
    assert_eq!(NestedBugInducer::from_bytes(bytes), s);
}

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
fn nested_ale_big() {
    let mut s = BigNestedBugInducer { one: 5, two: BigBugInducer { one: true, two: true, three: 0 }, three: 76 };
    assert_eq!(s.two.clone().into_bytes(), [0b0001_1000, 0, 0, 0]);
    let mut bytes = s.clone().into_bytes();
    assert_eq!(bytes, [0b0000_0101, 0b0001_1000, 0, 0, 0, 0b0100_1100]);
    assert_eq!(BigNestedBugInducer::from_bytes(bytes), s);

    s.one = 76;
    s.two.one = false;
    s.two.three = 666;
    s.three = 165;
    BigNestedBugInducer::write_slice_one(&mut bytes, s.one).unwrap();
    BigNestedBugInducer::write_slice_two(&mut bytes, s.two.clone()).unwrap();
    BigNestedBugInducer::write_slice_three(&mut bytes, s.three).unwrap();
    assert_eq!(BigNestedBugInducer::read_slice_two(&bytes).unwrap(), s.two);
    assert_eq!(BigNestedBugInducer::from_bytes(bytes), s);
}

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
fn nested_ale_aligned() {
    let mut s = AlignedNestedBugInducer {
        one: 5,
        two: AlignedBugInducer { one: true, two: true, three: 0 },
        three: 76,
    };
    assert_eq!(s.two.clone().into_bytes(), [3, 0, 0, 0]);
    let mut bytes = s.clone().into_bytes();
    assert_eq!(bytes, [5, 0, 3, 0, 0, 0, 76]);
    s.one = 76;
    s.two.one = false;
    s.two.three = 666;
    s.three = 165;
    AlignedNestedBugInducer::write_one(&mut bytes, s.one);
    AlignedNestedBugInducer::write_two(&mut bytes, s.two.clone());
    AlignedNestedBugInducer::write_three(&mut bytes, s.three);
    assert_eq!(AlignedNestedBugInducer::from_bytes(bytes), s);
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be")]
struct BlockArray {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(block_bit_length = 9)]
    two: [u8; 2],
    #[bondrewd(bit_length = 4)]
    three: u8,
}

#[test]
fn be_block_array() {
    let s = BlockArray { one: 0, two: [1, u8::MAX], three: 0 };
    let bytes = s.clone().into_bytes();
    assert_eq!(bytes, [0b0001_1111, 0b1111_0000]);
    assert_eq!(BlockArray::read_two(&bytes), s.two);
    assert_eq!(BlockArray::from_bytes(bytes), s);
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be")]
struct ElementArray {
    #[bondrewd(bit_length = 4)]
    one: u8,
    #[bondrewd(element_bit_length = 1)]
    two: [bool; 5],
    #[bondrewd(bit_length = 7)]
    three: u8,
}

#[test]
fn be_element_array() {
    let s = ElementArray { one: 0, two: [true, false, true, false, true], three: 0 };
    let bytes = s.clone().into_bytes();
    assert_eq!(bytes, [0b0000_1010, 0b1000_0000]);
    assert_eq!(ElementArray::read_slice_two(&bytes).unwrap(), s.two);
    assert_eq!(ElementArray::from_bytes(bytes), s);
}

#[derive(Eq, PartialEq, Clone, Debug, Bitfields)]
#[bondrewd(endianness = "be", id_bit_length = 8)]
enum CustomContinuation {
    Zero = 0x7F,
    ZeroContinued,
    One = 0x3F,
    OneContinued,
    Invalid,
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be")]
struct CustomContinuationUsage {
    one: u8,
    #[bondrewd(bit_length = 8)]
    two: CustomContinuation,
    three: u8,
}

#[test]
fn enum_discriminant_continuation() {
    let s = CustomContinuationUsage { one: 0x80, two: CustomContinuation::OneContinued, three: 0x08 };
    let mut bytes = s.clone().into_bytes();
    assert_eq!(bytes, [0x80, 0x40, 0x08]);
    assert_eq!(CustomContinuationUsage::from_bytes(bytes), s);
    CustomContinuationUsage::write_slice_two(&mut bytes, CustomContinuation::ZeroContinued).unwrap();
    assert_eq!(bytes[1], 0x80);
    // `Invalid` is the last variant, so it catches unknown ids.
    bytes[1] = 0x01;
    assert_eq!(CustomContinuationUsage::read_two(&bytes), CustomContinuation::Invalid);
}

// Intentional change from bondrewd 0.3: in `ale` enums, a variant shorter than the largest
// one is laid out right after the id like any struct. bondrewd 0.3 pushed it to the top of
// the payload, leaving a gap after the id.
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale", id_bit_length = 3)]
enum AleShortVariants {
    Zero,
    One {
        #[bondrewd(bit_length = 9)]
        a: u16,
        b: bool,
    },
    #[bondrewd(id = 5)]
    Five {
        #[bondrewd(bit_length = 4)]
        c: i8,
    },
    Other {
        #[bondrewd(capture_id)]
        id: u8,
        #[bondrewd(bit_length = 6)]
        d: u8,
    },
}

#[test]
fn ale_enum_short_variants_follow_the_id() {
    assert_eq!(AleShortVariants::BIT_SIZE, 13);
    // 13 bits in 2 bytes: 3 padding bits at the bottom, id at bits 3..6, fields from bit 6.
    let other = AleShortVariants::Other { id: 7, d: 63 };
    assert_eq!(other.clone().into_bytes(), [0xF8, 0x0F]); // bondrewd 0.3: [0x38, 0xFC]
    assert_eq!(AleShortVariants::from_bytes([0xF8, 0x0F]), other);
    let five = AleShortVariants::Five { c: -1 };
    assert_eq!(five.clone().into_bytes(), [0xE8, 0x03]);
    assert_eq!(AleShortVariants::from_bytes([0xE8, 0x03]), five);
}
