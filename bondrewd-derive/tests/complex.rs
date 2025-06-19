use bondrewd::{Bitfields, BitfieldsSlice};

// TODO add the ability to mark a field in the variants as the id which will contain the value the id and
// be ignored as a field of the struct.
// TODO add a functions that get and set the id.
#[derive(Bitfields, BitfieldsSlice, Clone, Debug, PartialEq, Eq)]
#[bondrewd(endianness = "be", id_bit_length = 14)]
enum ComplexEnum {
    One {
        test: u32,
    },
    Two {
        test: u8,
        test_two: u8,
    },
    Three {
        // TODO: fix
        /// DO NOT CHANGE THIS. i believe it produces un-optimized code because it
        /// rotates the bits right 6 times.
        #[bondrewd(bit_length = 30)]
        test: u32,
    },
    Invalid {
        #[bondrewd(capture_id)]
        id: u16,
    },
}

#[derive(Bitfields, Clone, Debug, PartialEq, Eq)]
#[bondrewd(endianness = "be", id_bit_length = 3)]
enum SimpleEnum {
    Alpha,
    Beta,
    Charley = 3,
    Invalid,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Bitfields, Clone, Debug, PartialEq)]
#[bondrewd(endianness = "be")]
struct SimpleExample {
    // fields that are as expected do not require attributes.
    one: bool,
    two: f32,
    #[bondrewd(bit_length = 14)]
    three: i16,
    #[bondrewd(redundant, bit_length = 6)]
    flags: u8,
    flag_one: bool,
    flag_two: bool,
    flag_three: bool,
    flag_four: bool,
    flag_five: bool,
    flag_six: bool,
    // TODO make it so the struct_size attribute can be defined with bits.
    // currently the backend used byte size for all of the math surrounding
    // the extraction of nested bondrewd structures, i believe this could
    // become bit_size based.
    // this would allow us to either:
    // - enforce that nested bondrewd Bitfields use either `bit-length`
    //      or `byte-length` instead of `struct_size`. it would be kind to still
    //      allow struct_size but just have the underlying code do what
    //      `byte-length` does.
    // - make `struct-bit-length` and `struct-byte-length` replace `struct_size`.
    #[bondrewd(bit_length = 46)]
    enum_field: ComplexEnum,
    #[bondrewd(bit_length = 3)]
    other_enum_field: SimpleEnum,
}

#[allow(clippy::float_cmp)]
#[test]
fn complex_stuff() {
    // this is to test capturing the id in the invalid.
    let mut bytes = ComplexEnum::Invalid { id: 53 }.into_bytes();
    assert_eq!(6, ComplexEnum::BYTE_SIZE);
    assert_eq!(46, ComplexEnum::BIT_SIZE);
    assert_eq!([0b0000_0000, 0b1101_0100, 0, 0, 0, 0,], bytes);
    ComplexEnum::write_variant_id(&mut bytes, 35);
    assert_eq!([0b0000_0000, 0b1000_1100, 0, 0, 0, 0,], bytes);
    let reconstructed = ComplexEnum::from_bytes(bytes);
    match reconstructed {
        ComplexEnum::Invalid { id } => {
            assert_eq!(id, 35);
        }
        _ => panic!("ComplexEnum from bytes did not get the invalid variant when it should have."),
    }

    // test full structure with Enums that are `Bitfields` NOT `BitfieldEnum`.
    assert_eq!(13, SimpleExample::BYTE_SIZE);
    // if you are wondering why the 2 is there. it is because bondrewd currently does
    // not support nested `Bitfields` to use bit sizing. read TODO above the declaration
    // of the `SimpleExample::enum_field` field.
    assert_eq!(53 + 46 + 3, SimpleExample::BIT_SIZE);
    let og = SimpleExample {
        one: false,
        two: -4.25,
        three: -1034,
        flags: 0,
        flag_one: true,
        flag_two: true,
        flag_three: true,
        flag_four: true,
        flag_five: true,
        flag_six: true,
        enum_field: ComplexEnum::Two {
            test: 3,
            test_two: 3,
        },
        other_enum_field: SimpleEnum::Charley,
    };
    let enum_field_bytes = og.enum_field.clone().into_bytes();
    assert_eq!(
        enum_field_bytes,
        [
            0b00000000,
            0b000001_00,
            0b000011_00,
            0b000011_00,
            0b00000000,
            0b00000000,
        ]
    );
    let mut bytes = og.clone().into_bytes();
    // check the output binary is correct. (i did math by hand
    // to get the binary). each field is separated by a underscore
    // in the binary assert to make it easy to see.
    assert_eq!(
        bytes,
        [
            0b0_1100000,  // one - two,
            0b01000100,   // two,
            0b00000000,   // two,
            0b00000000,   // two,
            0b0_1110111,  // two - three,
            0b1110110_1,  // three - flags,
            0b11111_000,  // flags - enum_field_id
            0b00000000,   // enum_field.id
            0b001_00000,  // enum_field.id - enum_field::Two.test
            0b011_00000,  // enum_field::Two.test - enum_field::Two.test_two
            0b011_00000,  // enum_field::Two.test_two - enum_field::Two.fill
            0b00000000,   // enum_field::Two.fill
            0b000_011_00, // enum_field::Two.fill - other_enum_field -- unused
        ],
    );
    // use read functions to get the fields value without
    // doing a from_bytes call.
    assert!(!SimpleExample::read_one(&bytes));
    assert_eq!(-4.25, SimpleExample::read_two(&bytes));
    assert_eq!(-1034, SimpleExample::read_three(&bytes));
    assert_eq!(63, SimpleExample::read_flags(&bytes));
    // overwrite the values with new ones in the byte array.
    SimpleExample::write_one(&mut bytes, true);
    SimpleExample::write_two(&mut bytes, 5.5);
    SimpleExample::write_three(&mut bytes, 511);
    SimpleExample::write_flags(&mut bytes, 0);
    // from bytes uses the read function so there is no need to
    // assert the read functions again.
    let reconstructed = SimpleExample::from_bytes(bytes);
    // check the values read by from bytes and check if they are
    // what we wrote to the bytes NOT the origanal values.
    assert!(reconstructed.one);
    assert_eq!(5.5, reconstructed.two);
    assert_eq!(511, reconstructed.three);
    assert_eq!(0, reconstructed.flags);
    assert_eq!(1, reconstructed.enum_field.id());
    assert_eq!(3, reconstructed.other_enum_field.id());
}

#[derive(Bitfields)]
#[repr(u8)]
#[bondrewd(endianness = "be", id_bit_length = 2, enforce_bits = 18)]
enum Thing {
    #[bondrewd(enforce_bits = 16)]
    One { a: u16 } = 1,
    #[bondrewd(enforce_bits = 16)]
    Two {
        #[bondrewd(bit_length = 10)]
        a: u16,
        #[bondrewd(bit_length = 6)]
        b: u8,
    } = 2,
    // TODO below attribute will break things. it should not.
    // #[bondrewd(enforce_bits = 16)]
    Idk {
        // TODO make sure capture id automatically gets the correct bit size.
        #[bondrewd(capture_id)]
        id: u8,
        a: u16,
    } = 0,
}

#[test]
fn not_invalid_behavior() {
    assert_eq!(Thing::BYTE_SIZE, 3);
    assert_eq!(Thing::BIT_SIZE, 18);
    let thing = Thing::One { a: 1 };
    let bytes = thing.into_bytes();
    // the first two bits are the id followed by Variant One's `a` field.
    assert_eq!(bytes[0], 0b01_000000);
    assert_eq!(bytes[1], 0b0000_0000);
    // because Variant One doesn't use the full amount of bytes so the last 6 bytes are just filler.
    assert_eq!(bytes[2], 0b01_000000);

    // fields that are capturing the id do not write.
    let mut bytes = Thing::Idk { id: 3, a: 0 }.into_bytes();
    // despite setting the id to 3 it will be 0 on output, this is to prevent
    // users from providing a valid id when it should not be.
    assert_eq!(bytes[0], 0b1100_0000);
    assert_eq!(bytes[1], 0b0000_0000);
    assert_eq!(bytes[2], 0b0000_0000);
    // but the id can be set to anything using the write_variant_id function.
    Thing::write_variant_id(&mut bytes, 3);
    // the id is now 3
    assert_eq!(bytes[0], 0b1100_0000);
    assert_eq!(bytes[1], 0b0000_0000);
    assert_eq!(bytes[2], 0b0000_0000);
    let reconstructed = Thing::from_bytes(bytes);
    // other than into_bytes everything else with give you the stored value.
    assert_eq!(reconstructed.id(), 3);
    match reconstructed {
        Thing::Idk { id, .. } => assert_eq!(id, 3),
        _ => panic!("id wasn't 3"),
    }
}
