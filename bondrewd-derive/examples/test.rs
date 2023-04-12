use bondrewd::*;

#[derive(Bitfields)]
#[bondrewd(default_endianness = "be", id_bits = 2)]
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
        /// DO NOT CHANGE THIS. i believe it produces optimized code because it
        /// rotates the bits right 6 times.
        #[bondrewd(bit_length = 30)]
        test: u32,
    },
    Invalid,
}

fn main() {}
