use bitfields::{BitfieldEnum, Bitfields};
use bitfields_derive::{BitfieldEnum as BitfieldEnumDerive, Bitfields as BitfieldsDerive};

#[derive(BitfieldEnumDerive, Clone, PartialEq, Eq, Debug)]
pub enum CcsdsPacketSequenceFlags {
    Continuation,
    Start,
    End,
    Unsegmented,
    Invalid(u8),
}

/// 3 bitt field describing the version number of Ccsds standard to use.
#[derive(BitfieldEnumDerive, Clone, PartialEq, Eq, Debug)]
#[bitfield_enum(u8)]
pub enum CcsdsPacketVersion {
    One,
    Two,
    #[invalid]
    Invalid,
}

#[derive(BitfieldsDerive, Clone, PartialEq, Eq, Debug)]
#[bitfields(default_endianness = "be", enforce_bytes = 6)]
pub struct CcsdsPacketHeader {
    #[enum_primitive = "u8"]
    #[bit_length = 3]
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

fn main() {
    let packet = CcsdsPacketHeader {
        packet_version_number: CcsdsPacketVersion::Invalid,
        packet_type: true,
        sec_hdr_flag: true,
        app_process_id: 55255 & 0b0000011111111111,
        sequence_flags: CcsdsPacketSequenceFlags::Unsegmented,
        packet_seq_count: 65535 & 0b0011111111111111,
        packet_data_length: 65535,
    };
    assert_eq!(
        CcsdsPacketHeader::from_bytes(packet.clone().into_bytes()),
        packet
    );
}
