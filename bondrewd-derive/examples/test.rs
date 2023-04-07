use bondrewd::*;

// TODO add the ability to mark a field in the variants as the id which will contain the value the id and
// be ignored as a field of the struct.
// TODO add a functions that get and set the id.
#[derive(Bitfields)]
#[bondrewd(default_endianness = "be", id_bits = 8)]
enum SimpleEnum {
    One {
        test: u32,
    },
    Two {
        test: u16,
        test_two: u8,
    },
    Three {
        // TODO: fix
        /// DO NOT CHANGE THIS. i believe it produces optimized code because it
        /// rotates the bits right 6 times.
        #[bondrewd(bit_length = 30)]
        test: u32,
    },
}

// #[derive(Bitfields)]
// #[bondrewd(default_endianness = "be")]
// struct SimpleExample {
//     // fields that are as expected do not require attributes.
//     one: bool,
//     two: f32,
//     #[bondrewd(bit_length = 14)]
//     three: i16,
//     #[bondrewd(redundant, bit_length = 6)]
//     flags: u8,
//     flag_one: bool,
//     flag_two: bool,
//     flag_three: bool,
//     flag_four: bool,
//     flag_five: bool,
//     flag_six: bool,
// }

fn main() {
    // assert_eq!(7, SimpleExample::BYTE_SIZE);
    // assert_eq!(53, SimpleExample::BIT_SIZE);
    // let mut bytes = SimpleExample {
    //     one: false,
    //     two: -4.25,
    //     three: -1034,
    //     flags: 0,
    //     flag_one: true,
    //     flag_two: true,
    //     flag_three: true,
    //     flag_four: true,
    //     flag_five: true,
    //     flag_six: true,
    // }
    // .into_bytes();
    // // check the output binary is correct. (i did math by hand
    // // to get the binary). each field is separated by a underscore
    // // in the binary assert to make it easy to see.
    // assert_eq!(
    //     [
    //         0b0_1100000, // one_two,
    //         0b01000100,  // two,
    //         0b00000000,  // two,
    //         0b00000000,  // two,
    //         0b0_1110111, // two_three,
    //         0b1110110_1, // three_four,
    //         0b11111_000, // four_unused
    //     ],
    //     bytes
    // );
    // // use read functions to get the fields value without
    // // doing a from_bytes call.
    // assert_eq!(false, SimpleExample::read_one(&bytes));
    // assert_eq!(-4.25, SimpleExample::read_two(&bytes));
    // assert_eq!(-1034, SimpleExample::read_three(&bytes));
    // assert_eq!(63, SimpleExample::read_flags(&bytes));
    // // overwrite the values with new ones in the byte array.
    // SimpleExample::write_one(&mut bytes, true);
    // SimpleExample::write_two(&mut bytes, 5.5);
    // SimpleExample::write_three(&mut bytes, 511);
    // SimpleExample::write_flags(&mut bytes, 0);
    // // from bytes uses the read function so there is no need to
    // // assert the read functions again.
    // let reconstructed = SimpleExample::from_bytes(bytes);
    // // check the values read by from bytes and check if they are
    // // what we wrote to the bytes NOT the origanal values.
    // assert_eq!(true, reconstructed.one);
    // assert_eq!(5.5, reconstructed.two);
    // assert_eq!(511, reconstructed.three);
    // assert_eq!(0, reconstructed.flags);
}
