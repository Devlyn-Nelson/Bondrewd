use bitfields::BitfieldEnum;
use bitfields_derive::BitfieldEnum as BitfieldEnumDerive;

// for situation where all bits are accounted for, like if this enum was used as a 2bit field than
// we can just let the last option be a valid catch all (in proc_macro code it is still marked as
// an invalid catch all but that doesn't really matter)
#[derive(Eq, PartialEq, Clone, Debug, BitfieldEnumDerive)]
#[bitfield_enum(u8)]
enum NoInvalidEnum {
    Zero,
    One,
    Two,
    /// because a field using only 2 bits has no more than 4 possible values this last field will be
    /// automatically marked as the Invalid catch all.
    Three,
}

#[test]
fn enum_auto_catch_all() {
    assert!(NoInvalidEnum::from_primitive(0u8).into_primitive() == 0);
    assert!(NoInvalidEnum::from_primitive(1u8).into_primitive() == 1);
    assert!(NoInvalidEnum::from_primitive(2u8).into_primitive() == 2);
    assert!(NoInvalidEnum::from_primitive(3u8).into_primitive() == 3);

    // test the catch all functionality
    assert!(NoInvalidEnum::from_primitive(4u8).into_primitive() == 3);
    assert!(NoInvalidEnum::from_primitive(5u8).into_primitive() == 3);
    assert!(NoInvalidEnum::from_primitive(154u8).into_primitive() == 3);
    assert!(NoInvalidEnum::from_primitive(255u8).into_primitive() == 3);
}
