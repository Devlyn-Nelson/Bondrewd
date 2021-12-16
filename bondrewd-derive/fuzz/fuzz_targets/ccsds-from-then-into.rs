#![no_main]
use libfuzzer_sys::fuzz_target;
use bondrewd::{Bitfields, BitfieldEnum};

#[derive(BitfieldEnum, Clone, PartialEq, Eq, Debug)]
pub enum CcsdsPacketSequenceFlags {
    Continuation,
    Start,
    End,
    Unsegmented,
    Invalid(u8)
}

/// 3 bitt field describing the version number of Ccsds standard to use.
#[derive(BitfieldEnum, Clone, PartialEq, Eq, Debug)]
#[bondrewd_enum("u8")]
pub enum CcsdsPacketVersion {
    One,
    Two,
    Invalid(u8),
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
pub struct CcsdsPacketHeader {
    #[bondrewd(enum_primitive = "u8",bit_length=3)]
    pub(crate) packet_version_number: CcsdsPacketVersion,
    pub(crate) packet_type: bool,
    pub(crate) sec_hdr_flag: bool,
    #[bondrewd(bit_length = 11)]
    pub(crate) app_process_id: u16,
    #[bondrewd(enum_primitive = "u8", bit_length = 2)]
    pub(crate) sequence_flags: CcsdsPacketSequenceFlags,
    #[bondrewd(bit_length = 14)]
    pub(crate) packet_seq_count: u16,
    pub(crate) packet_data_length: u16,
}

fuzz_target!(|data: [u8;6]| {
    assert_eq!(CcsdsPacketHeader::BIT_SIZE, 6 * 8);
    assert_eq!(CcsdsPacketHeader::from_bytes(data.clone()).into_bytes(), data);
    
});
