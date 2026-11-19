use bondrewd::{BitfieldLengthError, Bitfields, BitfieldsSlice};

// ---------------------------------------------------------------- README example

#[derive(Bitfields, Clone, Eq, PartialEq, Debug)]
#[bondrewd(id_bit_length = 3, endianness = "be")]
enum EyeColor {
    Blue,
    Green,
    Brown,
    Other {
        #[bondrewd(capture_id)]
        test: u8,
    },
}

#[derive(Bitfields, Clone, Eq, PartialEq, Debug)]
#[bondrewd(endianness = "be")]
struct PersonParts {
    head: bool,
    #[bondrewd(bit_length = 2)]
    shoulders: u8,
    #[bondrewd(bit_length = 2)]
    knees: u8,
    #[bondrewd(bit_length = 4)]
    toes: u8,
}

#[derive(Bitfields, BitfieldsSlice, Clone, Eq, PartialEq, Debug)]
#[bondrewd(endianness = "be", reverse)]
struct Person {
    #[bondrewd(element_byte_length = 2)]
    name: [char; 32],
    #[bondrewd(bit_length = 7)]
    age: u8,
    #[bondrewd(bit_length = 3)]
    eye_color: EyeColor,
    #[bondrewd(bit_length = 9)]
    parts: PersonParts,
    #[bondrewd(bit_length = 60)]
    blinks: u64,
}

#[test]
fn readme_person() {
    let person = Person {
        name: ['a'; 32],
        age: 27,
        eye_color: EyeColor::Blue,
        parts: PersonParts { head: true, shoulders: 2, knees: 2, toes: 10 },
        blinks: 10_000_000_000,
    };
    let bytes = person.clone().into_bytes();
    assert_eq!(Person::from_bytes(bytes), person);

    let target = Person {
        name: ['b'; 32],
        age: 72,
        eye_color: EyeColor::Other { test: 6 },
        parts: PersonParts { head: false, shoulders: 1, knees: 3, toes: 7 },
        blinks: 5,
    };
    let mut bytes = person.into_bytes();
    Person::write_name(&mut bytes, target.name);
    Person::write_age(&mut bytes, target.age);
    Person::write_eye_color(&mut bytes, target.eye_color.clone());
    Person::write_parts(&mut bytes, target.parts.clone());
    Person::write_blinks(&mut bytes, target.blinks);
    assert_eq!(Person::read_name(&bytes), target.name);
    assert_eq!(Person::read_eye_color(&bytes), target.eye_color);
    assert_eq!(Person::read_parts(&bytes), target.parts);
    assert_eq!(Person::from_bytes(bytes), target);
}

// ---------------------------------------------------------------- enums

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be")]
enum Command {
    Nop,
    Move { #[bondrewd(bit_length = 4)] x: i8, #[bondrewd(bit_length = 4)] y: i8 },
    Say(char),
    Unknown(#[bondrewd(capture_id)] u8),
}

#[test]
fn enum_default_id_bits_and_layout() {
    // 4 variants -> 2 id bits, biggest variant is `Say` with 32 bits.
    assert_eq!(Command::BIT_SIZE, 34);
    assert_eq!(Command::BYTE_SIZE, 5);
    let m = Command::Move { x: -3, y: 7 };
    let bytes = m.clone().into_bytes();
    // id 01, x = 1101, y = 0111
    assert_eq!(bytes[0], 0b0111_0101);
    assert_eq!(bytes[1], 0b1100_0000);
    assert_eq!(Command::read_variant_id(&bytes), 1);
    assert_eq!(Command::read_move_x(&bytes), -3);
    assert_eq!(Command::from_bytes(bytes), m);

    let s = Command::Say('λ');
    assert_eq!(Command::from_bytes(s.clone().into_bytes()), s);
    assert_eq!(Command::read_say_field_0(&s.into_bytes()), 'λ');

    // The last variant is the fallback and captures the id.
    let u = Command::Unknown(3);
    assert_eq!(Command::from_bytes(u.clone().into_bytes()), u);
    assert_eq!(Command::Nop.into_bytes(), [0; 5]);
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be", id_byte_length = 3)]
enum Inner {
    NoRightAnswers(#[bondrewd(capture_id)] u32),
}

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be")]
struct ElementArrayOfEnums {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(element_byte_length = 3)]
    two: [Inner; 5],
    #[bondrewd(bit_length = 5)]
    three: u8,
}

#[test]
fn element_array_of_enums() {
    let s = ElementArrayOfEnums {
        one: 2,
        two: [234, 847, 135, 69, 78].map(Inner::NoRightAnswers),
        three: 9,
    };
    assert_eq!(ElementArrayOfEnums::BIT_SIZE, 3 + 5 * 24 + 5);
    let bytes = s.clone().into_bytes();
    assert_eq!(ElementArrayOfEnums::read_slice_two(&bytes).unwrap(), s.two);
    assert_eq!(ElementArrayOfEnums::from_bytes(bytes), s);
}

#[derive(Bitfields, Clone, Copy, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "le", id_bit_length = 4)]
enum Mode {
    #[bondrewd(id = 9)]
    Fast,
    #[bondrewd(id = 3)]
    Slow,
    #[bondrewd(invalid)]
    Broken,
    #[bondrewd(id = 12)]
    Off,
}

#[test]
fn explicit_ids_and_invalid_variant() {
    assert_eq!(Mode::BIT_SIZE, 4);
    assert_eq!(Mode::Fast.into_bytes(), [0b1001_0000]);
    assert_eq!(Mode::Off.into_bytes(), [0b1100_0000]);
    assert_eq!(Mode::from_bytes([0b0011_0000]), Mode::Slow);
    assert_eq!(Mode::from_bytes([0b0111_0000]), Mode::Broken);
    assert_eq!(Mode::from_bytes([0b1100_0000]), Mode::Off);
}

// ---------------------------------------------------------------- slices

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be")]
struct Header {
    #[bondrewd(bit_length = 3)]
    version: u8,
    #[bondrewd(bit_length = 13)]
    id: u16,
    length: u16,
    #[bondrewd(bit_length = 2, reserve)]
    spare: u8,
}

#[test]
fn slice_functions() {
    let h = Header { version: 5, id: 0x1ABC, length: 300, spare: 0 };
    let bytes = h.clone().into_bytes();
    assert_eq!(Header::BYTE_SIZE, 5);

    // `id` only needs the first two bytes.
    assert_eq!(Header::read_slice_id(&bytes[..2]), Ok(0x1ABC));
    assert_eq!(Header::read_slice_length(&bytes[..3]), Err(BitfieldLengthError(3, 4)));

    let mut short = [0u8; 2];
    Header::write_slice_id(&mut short, 0x0123).unwrap();
    assert_eq!(Header::read_slice_id(&short), Ok(0x0123));
    assert!(Header::write_slice_spare(&mut short, 1).is_err());

    assert_eq!(Header::from_slice(&bytes), Ok(h.clone()));
    assert_eq!(Header::from_slice(&bytes[..4]), Err(BitfieldLengthError(4, 5)));
}

#[test]
fn checked_views() {
    let h = Header { version: 1, id: 77, length: 0xBEEF, spare: 0 };
    let mut bytes = h.clone().into_bytes().to_vec();
    bytes.push(0xAA);

    let checked = Header::check_slice(&bytes).unwrap();
    assert_eq!(checked.read_version(), 1);
    assert_eq!(checked.read_id(), 77);
    assert_eq!(checked.read_length(), 0xBEEF);
    let back: Header = checked.into();
    assert_eq!(back, h);

    let mut checked = Header::check_slice_mut(&mut bytes).unwrap();
    checked.write_length(7);
    checked.write_version(6);
    assert_eq!(checked.read_length(), 7);
    let changed: Header = checked.into();
    assert_eq!(changed, Header { version: 6, length: 7, ..h });

    assert!(Header::check_slice(&bytes[..3]).is_err());
}

#[test]
fn take_from_vec() {
    let a = Header { version: 1, id: 2, length: 3, spare: 0 };
    let b = Header { version: 4, id: 5, length: 6, spare: 0 };
    let mut v = a.clone().into_bytes().to_vec();
    v.extend_from_slice(&b.clone().into_bytes());
    v.extend_from_slice(&[1, 2]);
    assert_eq!(Header::take_from_vec(&mut v), Ok(a));
    assert_eq!(Header::take_from_vec(&mut v), Ok(b));
    assert_eq!(v, [1, 2]);
    assert_eq!(Header::take_from_vec(&mut v), Err(BitfieldLengthError(2, 5)));
    assert_eq!(v, [1, 2]);
}

#[derive(BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "le")]
struct SliceOnly {
    #[bondrewd(bit_length = 12)]
    a: u16,
    #[bondrewd(bit_length = 4)]
    b: u8,
}

#[test]
fn slice_only_derive() {
    let bytes = [0x34, 0x12];
    let s = SliceOnly::from_slice(&bytes).unwrap();
    assert_eq!(<SliceOnly as BitfieldsSlice>::BYTE_SIZE, 2);
    assert_eq!(SliceOnly::read_slice_a(&bytes), Ok(s.a));
    assert_eq!(SliceOnly::read_slice_b(&bytes), Ok(s.b));
}

// ---------------------------------------------------------------- arrays

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Debug)]
#[bondrewd(endianness = "le")]
struct Arrays {
    #[bondrewd(element_bit_length = 12)]
    words: [u16; 3],
    #[bondrewd(element_bit_length = 12)]
    pairs: [[u8; 2]; 2],
    #[bondrewd(block_bit_length = 20)]
    block: [u16; 2],
    signed: [i8; 2],
    floats: [f32; 2],
    #[bondrewd(element_bit_length = 5)]
    nested: [Small; 3],
}

#[derive(Bitfields, Clone, Copy, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be")]
struct Small {
    flag: bool,
    #[bondrewd(bit_length = 4)]
    val: u8,
}

#[test]
fn array_kinds() {
    let a = Arrays {
        words: [0xABC, 0x123, 0xFFF],
        pairs: [[0x02, 0x34], [0x0F, 0xFF]], // 12-bit blocks: element 0 keeps 4 bits
        block: [0x000F, 0xFFFF], // 20-bit block: element 0 keeps 4 bits
        signed: [-128, 127],
        floats: [1.5, -0.25],
        nested: [Small { flag: true, val: 3 }, Small { flag: false, val: 15 }, Small { flag: true, val: 0 }],
    };
    assert_eq!(Arrays::BIT_SIZE, 36 + 24 + 20 + 16 + 64 + 15);
    let bytes = a.clone().into_bytes();
    assert_eq!(Arrays::read_words(&bytes), a.words);
    assert_eq!(Arrays::read_pairs(&bytes), a.pairs);
    assert_eq!(Arrays::read_block(&bytes), a.block);
    assert_eq!(Arrays::read_slice_nested(&bytes).unwrap(), a.nested);
    assert_eq!(Arrays::from_bytes(bytes), a);
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "le")]
struct LeBlock {
    #[bondrewd(block_bit_length = 12)]
    block: [u8; 2],
}

#[test]
fn le_block_truncates_first_element() {
    // Block arrays drop the top bits of element 0, for every endianness (matches bondrewd 0.3).
    let s = LeBlock { block: [0xAB, 0xFC] };
    let bytes = s.into_bytes();
    assert_eq!(bytes, [0b1011_1111, 0b1100_0000]);
    assert_eq!(LeBlock::from_bytes(bytes), LeBlock { block: [0x0B, 0xFC] });
}

// ---------------------------------------------------------------- mixing

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be")]
struct MixedEndian {
    #[bondrewd(endianness = "le")]
    little: u16,
    big: u16,
}

#[test]
fn per_field_endianness() {
    let m = MixedEndian { little: 0x1234, big: 0x1234 };
    assert_eq!(m.clone().into_bytes(), [0x34, 0x12, 0x12, 0x34]);
    assert_eq!(MixedEndian::from_bytes(m.clone().into_bytes()), m);
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "ale")]
struct AleParent {
    #[bondrewd(bit_length = 3)]
    a: u8,
    #[bondrewd(bit_length = 9)]
    be_child: PersonParts,
    #[bondrewd(bit_length = 5)]
    b: u8,
}

#[test]
fn be_child_inside_ale_parent() {
    let p = AleParent { a: 5, be_child: PersonParts { head: true, shoulders: 1, knees: 2, toes: 9 }, b: 17 };
    let bytes = p.clone().into_bytes();
    assert_eq!(AleParent::read_be_child(&bytes), p.be_child);
    assert_eq!(AleParent::from_bytes(bytes), p);
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be", enforce_bytes = 3, fill_bits = 24)]
struct Tuple(#[bondrewd(bit_length = 7)] u8, bool, #[bondrewd(bit_length = 2)] i8);

#[test]
fn tuple_struct_and_fill() {
    assert_eq!(Tuple::BIT_SIZE, 24);
    let t = Tuple(0x7F, true, -2);
    let bytes = t.clone().into_bytes();
    assert_eq!(bytes, [0b1111_1111, 0b1000_0000, 0]);
    assert_eq!(Tuple::read_field_2(&bytes), -2);
    assert_eq!(Tuple::from_bytes(bytes), t);
}
