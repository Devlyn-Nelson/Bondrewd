// Shared by the old and new crates: same source, different `bondrewd`.
use bondrewd::Bitfields;

macro_rules! simple {
    ($name:ident, $($attr:tt)*) => {
        #[derive(Bitfields, Clone, PartialEq, Debug)]
        #[bondrewd($($attr)*)]
        struct $name {
            #[bondrewd(bit_length = 3)] one: u8,
            #[bondrewd(bit_length = 27)] two: u32,
            #[bondrewd(bit_length = 14)] three: u16,
            four: u8,
            #[bondrewd(bit_length = 5)] five: i8,
            #[bondrewd(bit_length = 45)] six: i64,
            flag: bool,
            #[bondrewd(bit_length = 70)] seven: u128,
        }
    };
}
simple!(BeS, endianness = "be");
simple!(LeS, endianness = "le");
simple!(AleS, endianness = "ale");
simple!(BeRev, endianness = "be", reverse);
simple!(LeRev, endianness = "le", reverse);
simple!(AleRev, endianness = "ale", reverse);
simple!(BeBack, endianness = "be", bit_traversal = "back");
simple!(LeBack, endianness = "le", bit_traversal = "back");
simple!(AleBack, endianness = "ale", bit_traversal = "back");
simple!(BeFill, endianness = "be", fill_bits);
simple!(AleFill, endianness = "ale", fill_bits);

macro_rules! nested {
    ($inner:ident, $outer:ident, $($attr:tt)*) => {
        #[derive(Bitfields, Clone, PartialEq, Debug)]
        #[bondrewd($($attr)*)]
        struct $inner {
            a: bool,
            #[bondrewd(bit_length = 3)] b: u8,
            #[bondrewd(bit_length = 7)] c: u16,
        }
        #[derive(Bitfields, Clone, PartialEq, Debug)]
        #[bondrewd($($attr)*)]
        struct $outer {
            #[bondrewd(bit_length = 5)] x: u8,
            #[bondrewd(bit_length = 11)] inner: $inner,
            #[bondrewd(bit_length = 13)] y: u16,
            #[bondrewd(element_bit_length = 11)] many: [$inner; 2],
        }
    };
}
nested!(InBe, OutBe, endianness = "be");
nested!(InLe, OutLe, endianness = "le");
nested!(InAle, OutAle, endianness = "ale");

macro_rules! arrays {
    ($name:ident, $($attr:tt)*) => {
        #[derive(Bitfields, Clone, PartialEq, Debug)]
        #[bondrewd($($attr)*)]
        struct $name {
            #[bondrewd(bit_length = 3)] pad: u8,
            #[bondrewd(element_bit_length = 5)] elems: [u8; 3],
            #[bondrewd(element_bit_length = 12)] wide: [u16; 2],
            #[bondrewd(block_bit_length = 21)] block: [u8; 3],
            #[bondrewd(block_bit_length = 28)] block16: [u16; 2],
            bools: [bool; 3],
        }
    };
}
arrays!(ArrBe, endianness = "be");
arrays!(ArrLe, endianness = "le");
// ArrAle: old bondrewd-derive panics on ale arrays, so it is compared separately.

macro_rules! enums {
    ($name:ident, $user:ident, $($attr:tt)*) => {
        #[derive(Bitfields, Clone, PartialEq, Debug)]
        #[bondrewd($($attr)*, id_bit_length = 3)]
        enum $name {
            Zero,
            One { #[bondrewd(bit_length = 9)] a: u16, b: bool },
            #[bondrewd(id = 5)]
            Five { #[bondrewd(bit_length = 4)] c: i8 },
            Other { #[bondrewd(capture_id)] id: u8, #[bondrewd(bit_length = 6)] d: u8 },
        }
        #[derive(Bitfields, Clone, PartialEq, Debug)]
        #[bondrewd($($attr)*)]
        struct $user {
            #[bondrewd(bit_length = 2)] pre: u8,
            #[bondrewd(bit_length = 13)] e: $name,
            #[bondrewd(bit_length = 3)] post: u8,
        }
    };
}
enums!(EnBe, EnUseBe, endianness = "be");
enums!(EnLe, EnUseLe, endianness = "le");
enums!(EnAle, EnUseAle, endianness = "ale");

#[derive(Bitfields, Clone, PartialEq, Debug)]
#[bondrewd(endianness = "be")]
enum Plain {
    A,
    B,
    C,
    D,
    E,
    Invalid,
}

#[derive(Bitfields, Clone, PartialEq, Debug)]
#[bondrewd(endianness = "be")]
struct UsesPlain {
    #[bondrewd(bit_length = 3)]
    p: Plain,
    #[bondrewd(bit_length = 5)]
    q: u8,
}

// CCSDS space packet header, as in tests/ccsds.rs.
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(id_bit_length = 2, endianness = "be")]
pub enum SeqFlags { Continuation, Start, End, Unsegmented }
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(id_bit_length = 3, endianness = "be")]
pub enum Version { One, Two, Invalid }
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be", enforce_bytes = 6)]
pub struct SpacePacketHeader {
    #[bondrewd(bit_length = 3)] version: Version,
    packet_type: bool,
    sec_hdr_flag: bool,
    #[bondrewd(bit_length = 11)] apid: u16,
    #[bondrewd(bit_length = 2)] seq_flags: SeqFlags,
    #[bondrewd(bit_length = 14)] seq_count: u16,
    data_length: u16,
}

struct Rng(u64);
impl Rng {
    fn byte(&mut self) -> u8 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 24) as u8
    }
}

macro_rules! run {
    ($($t:ident),* $(,)?) => {$({
        let mut r = Rng(0x1234_5678_9abc_def1);
        println!("{} BIT_SIZE={} BYTE_SIZE={}", stringify!($t), $t::BIT_SIZE, $t::BYTE_SIZE);
        for i in 0..200 {
            let mut b = [0u8; $t::BYTE_SIZE];
            for x in b.iter_mut() { *x = r.byte(); }
            // Low ids and all-zero/all-one buffers hit the interesting enum and edge cases.
            if i == 0 { b = [0; $t::BYTE_SIZE]; }
            if i == 1 { b = [0xFF; $t::BYTE_SIZE]; }
            let v = $t::from_bytes(b);
            let out = v.clone().into_bytes();
            let rt = $t::from_bytes(out) == v;
            println!("{} {} rt={} in={:02x?} value={:?} out={:02x?}", stringify!($t), i, rt, b, v, out);
        }
    })*};
}

fn main() {
    run!(
        BeS, LeS, AleS, BeRev, LeRev, AleRev, BeBack, LeBack, AleBack, BeFill, AleFill,
        OutBe, OutLe, ArrBe, ArrLe, EnBe, EnLe, EnAle, EnUseBe, EnUseLe, EnUseAle,
        UsesPlain, SpacePacketHeader,
    );
}
