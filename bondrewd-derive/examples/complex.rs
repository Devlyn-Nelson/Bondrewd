use bondrewd::{BitfieldEnum, Bitfields};
#[cfg(feature = "slice_fns")]
use bondrewd::BitfieldSliceError;

#[derive(BitfieldEnum, Clone, Eq, PartialEq, Debug)]
enum EyeColor {
    Blue,
    Green,
    Brown,
    Other(u8),
}

#[derive(Bitfields, Clone, Eq, PartialEq, Debug)]
#[bondrewd(default_endianness = "be")]
struct PersonParts {
    head: bool,
    #[bondrewd(bit_length = 2)]
    shoulders: u8,
    #[bondrewd(bit_length = 2)]
    knees: u8,
    #[bondrewd(bit_length = 4)]
    toes: u8,
}

#[derive(Bitfields, Clone, Eq, PartialEq, Debug)]
#[bondrewd(default_endianness = "be", reverse)]
struct PersonStuff {
    // English only?
    // TODO this element array does all be the first element wrong
    #[bondrewd(element_byte_length = 2)]
    name: [char; 32],
    // Nobody lives past 127
    #[bondrewd(bit_length = 7)]
    age: u8,
    // Eyes have color
    #[bondrewd(enum_primitive = "u8", bit_length = 3)]
    eye_color: EyeColor,
    // Standard components
    #[bondrewd(bit_length = 9, struct_size = 2)]
    parts: PersonParts,
    // how many times they have blinked each of their eyes
    #[bondrewd(bit_length = 60)]
    blinks: u64,
}

fn main() {
    println!(
        "overall bits/bytes used: {}/{}",
        PersonStuff::BIT_SIZE,
        PersonStuff::BYTE_SIZE
    );

    let person = PersonStuff {
        name: ['a'; 32],
        age: 27,
        eye_color: EyeColor::Blue,
        parts: PersonParts {
            head: true,
            shoulders: 2,
            knees: 2,
            toes: 10,
        },
        blinks: 10000000000,
    };

    let bytes = person.clone().into_bytes();
    let cloned_and_transferred_person = PersonStuff::from_bytes(bytes);
    assert_eq!(cloned_and_transferred_person, person);
    println!("");
}
