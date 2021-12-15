#![no_main]
use libfuzzer_sys::fuzz_target;
use bitfields::{Bitfields, BitfieldEnum};
use bitfields_derive::{Bitfields as BitfieldsDerive, BitfieldEnum as BitfieldEnumDerive};

#[derive(BitfieldEnumDerive, Clone, PartialEq, Eq, Debug)]
pub enum CcsdsPacketSequenceFlags {
    Continuation,
    Start,
    End,
    Unsegmented,
    Invalid(u8)
}

/// 3 bitt field describing the version number of Ccsds standard to use.
#[derive(BitfieldEnumDerive, Clone, PartialEq, Eq, Debug)]
#[bitfield_enum("u8")]
pub enum CcsdsPacketVersion {
    One,
    Two,
    Invalid(u8),
}

#[derive(BitfieldsDerive, Clone, PartialEq, Eq, Debug)]
#[bitfields(default_endianness = "be")]
pub struct CcsdsPacketHeader {
    #[enum_primitive = "u8"]
    #[bit_length=3]
    pub(crate) packet_version_number: CcsdsPacketVersion,
    pub(crate) packet_type: bool,
    pub(crate) sec_hdr_flag: bool,
    #[bit_length = 11]
    pub(crate) app_process_id: u16,
    #[enum_primitive = "u8"]
    #[bit_length = 2]
    pub(crate) sequence_flags: CcsdsPacketSequenceFlags,
    #[bit_length = 14]
    pub(crate) packet_seq_count: u16,
    pub(crate) packet_data_length: u16,
}

fuzz_target!(|data: [u8;6]| {
    assert_eq!(CcsdsPacketHeader::BIT_SIZE, 6 * 8);
    assert_eq!(CcsdsPacketHeader::from_bytes(data.clone()).into_bytes(), data);
    
});
