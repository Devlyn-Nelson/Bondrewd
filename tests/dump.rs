//! `#[bondrewd(dump)]` writes the generated code into the target directory.
use bondrewd::{Bitfields, BitfieldsSlice};

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "be", dump)]
struct Dumped {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 13)]
    two: u16,
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(endianness = "le", dump)]
enum DumpedEnum {
    A,
    B { #[bondrewd(bit_length = 4)] x: u8 },
}

#[test]
fn dump_files_exist_and_code_still_works() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/bondrewd_dump/dump");
    for file in ["Dumped.bitfields.rs", "Dumped.bitfields_slice.rs", "DumpedEnum.bitfields.rs"] {
        let text = std::fs::read_to_string(dir.join(file)).unwrap_or_else(|e| panic!("{file}: {e}"));
        assert!(text.contains("Bitfields"), "{file} looks empty");
    }
    let d = Dumped { one: 5, two: 300 };
    assert_eq!(Dumped::from_bytes(d.clone().into_bytes()), d);
    let e = DumpedEnum::B { x: 9 };
    assert_eq!(DumpedEnum::from_bytes(e.clone().into_bytes()), e);
}
