//! Randomised round-trip and isolation checks for every layout mode.
use bondrewd::{Bitfields, BitfieldsSlice};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn bits(&mut self, n: u32) -> u128 {
        let v = (u128::from(self.next()) << 64) | u128::from(self.next());
        if n == 128 { v } else { v & ((1u128 << n) - 1) }
    }
    fn signed(&mut self, n: u32) -> i128 {
        let v = self.bits(n) as i128;
        (v << (128 - n)) >> (128 - n)
    }
}

macro_rules! fuzz_type {
    ($test:ident, $name:ident, $inner:ident, $($attr:tt)*) => {
        #[derive(Bitfields, Clone, Copy, PartialEq, Eq, Debug)]
        #[bondrewd($($attr)*)]
        struct $inner {
            #[bondrewd(bit_length = 3)]
            a: u8,
            #[bondrewd(bit_length = 6)]
            b: i8,
        }

        #[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
        #[bondrewd($($attr)*)]
        struct $name {
            flag: bool,
            #[bondrewd(bit_length = 13)]
            u: u16,
            #[bondrewd(bit_length = 27)]
            s: i32,
            #[bondrewd(bit_length = 9)]
            nested: $inner,
            #[bondrewd(bit_length = 61)]
            wide: u64,
            #[bondrewd(element_bit_length = 7)]
            elems: [u8; 3],
            #[bondrewd(block_bit_length = 19)]
            block: [u8; 3],
            #[bondrewd(bit_length = 100)]
            huge: i128,
            #[bondrewd(element_bit_length = 9)]
            inners: [$inner; 2],
            #[bondrewd(bit_length = 5)]
            tail: u8,
        }

        impl $name {
            fn random(r: &mut Rng) -> Self {
                let inner = |r: &mut Rng| $inner { a: r.bits(3) as u8, b: r.signed(6) as i8 };
                let block_top = r.bits(3) as u8; // 19 bits = 3 + 8 + 8
                // Element 0 loses its top bits, whatever the endianness.
                let block = [block_top, r.bits(8) as u8, r.bits(8) as u8];
                Self {
                    flag: r.bits(1) == 1,
                    u: r.bits(13) as u16,
                    s: r.signed(27) as i32,
                    nested: inner(r),
                    wide: r.bits(61) as u64,
                    elems: [r.bits(7) as u8, r.bits(7) as u8, r.bits(7) as u8],
                    block,
                    huge: r.signed(100),
                    inners: [inner(r), inner(r)],
                    tail: r.bits(5) as u8,
                }
            }
        }

        #[test]
        fn $test() {
            let mut r = Rng(0x9E37_79B9_7F4A_7C15 ^ stringify!($name).len() as u64);
            for _ in 0..2000 {
                let v = $name::random(&mut r);
                let bytes = v.clone().into_bytes();
                assert_eq!($name::from_bytes(bytes), v);
                assert_eq!($name::from_slice(&bytes).unwrap(), v);

                // Writing a field into noise must leave every other field untouched.
                let mut noise = [0u8; $name::BYTE_SIZE];
                noise.iter_mut().for_each(|b| *b = r.next() as u8);
                let before = $name::from_bytes(noise);
                $name::write_s(&mut noise, v.s);
                $name::write_nested(&mut noise, v.nested);
                $name::write_block(&mut noise, v.block);
                let after = $name::from_bytes(noise);
                assert_eq!(after, $name { s: v.s, nested: v.nested, block: v.block, ..before });

                // Field-wise writes rebuild the same bytes as `into_bytes`.
                let mut built = [0u8; $name::BYTE_SIZE];
                {
                    let mut c = $name::check_slice_mut(&mut built).unwrap();
                    c.write_flag(v.flag);
                    c.write_u(v.u);
                    c.write_s(v.s);
                    c.write_nested(v.nested);
                    c.write_wide(v.wide);
                    c.write_elems(v.elems);
                    c.write_block(v.block);
                    c.write_huge(v.huge);
                    c.write_inners(v.inners);
                    c.write_tail(v.tail);
                }
                assert_eq!(built, bytes);
                assert_eq!($name::read_huge(&bytes), v.huge);
                assert_eq!($name::read_slice_inners(&bytes).unwrap(), v.inners);
            }
        }
    };
}

fuzz_type!(fuzz_be, FuzzBe, InnerBe, endianness = "be");
fuzz_type!(fuzz_le, FuzzLe, InnerLe, endianness = "le");
fuzz_type!(fuzz_ale, FuzzAle, InnerAle, endianness = "ale");
fuzz_type!(fuzz_be_rev, FuzzBeRev, InnerBeRev, endianness = "be", reverse);
fuzz_type!(fuzz_le_back, FuzzLeBack, InnerLeBack, endianness = "le", bit_traversal = "back");
fuzz_type!(fuzz_ale_rev_back, FuzzAleRevBack, InnerAleRevBack, endianness = "ale", reverse, bit_traversal = "back");
