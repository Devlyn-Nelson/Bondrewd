use bondrewd::Bitfields;

// TODO add the ability to mark a field in the variants as the id which will contain the value the id and
// be ignored as a field of the struct.
// TODO add a functions that get and set the id.
#[derive(Bitfields)]
#[bondrewd(default_endianness = "be", id_bit_length = 14)]
enum ComplexEnum {
    One {
        test: u32,
    },
    Two {
        test: u8,
        test_two: u8,
    },
    Three {
        #[bondrewd(bit_length = 30)]
        test: u32,
    },
    Invalid {
        #[bondrewd(capture_id)]
        id: u16,
    },
}

#[derive(Bitfields)]
#[bondrewd(default_endianness = "be", id_bit_length = 3)]
enum SimpleEnum {
    Alpha,
    Beta,
    Charley = 3,
    Invalid,
}

#[derive(Bitfields)]
#[bondrewd(default_endianness = "be")]
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

fn main() {
    // this is to test capturing the id in the invalid.
    let mut bytes = ComplexEnum::Invalid { id: 53 }.into_bytes();
    assert_eq!(6, ComplexEnum::BYTE_SIZE);
    assert_eq!(46, ComplexEnum::BIT_SIZE);
    assert_eq!([0b0000_0000, 0b0000_1100, 0, 0, 0, 0,], bytes);
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
    let mut bytes = SimpleExample {
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
    }
    .into_bytes();
    // check the output binary is correct. (i did math by hand
    // to get the binary). each field is separated by a underscore
    // in the binary assert to make it easy to see.
    assert_eq!(
        [
            0b0_1100000, // one - two,
            0b0100_0100, // two,
            0b0000_0000, // two,
            0b0000_0000, // two,
            0b0_1110111, // two - three,
            0b1110_1101, // three - flags,
            0b1111_1000, // flags - enum_field_id
            0b0000_0000, // enum_field_id
            0b001_00000, // enum_field_id - enum_field_TWO_test
            0b011_00000, // enum_field_TWO_test - enum_field_TWO_test_two
            0b011_00000, // enum_field_TWO_test_two - unused
            0b0000_0000, // unused
            0b0000_1100, // unused - other_enum_field -- unused
        ],
        bytes
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
