use bondrewd::{BitfieldEnum, Bitfields};
use bondrewd_derive::{BitfieldEnum as BitfieldEnumDerive, Bitfields as BitfieldsDerive};

#[derive(BitfieldsDerive)]
#[bondrewd(default_endianness = "be")]
pub struct DownlinkFileHeader {
    #[element_byte_length = 1]
    aes256: [u8; 16],
    tai_timestamp: u64,
    #[bit_length = 2]
    reserve_spare: u8,
    #[bit_length = 30]
    pub file_id: u32,
    #[element_byte_length = 1]
    hash: [u8; 16],
}

#[derive(BitfieldsDerive)]
#[bondrewd(default_endianness = "be")]
pub struct UplinkFileHeader {
    #[element_byte_length = 1]
    aes256: [u8; 16],
    tai_timestamp: u64,
    #[bit_length = 2]
    reserve_spare: u8,
    #[bit_length = 30]
    pub file_id: u32,
    chunk_index: u16,
    #[element_byte_length = 1]
    hash: [u8; 16],
}

#[derive(BitfieldsDerive)]
#[bondrewd(default_endianness = "be")]
pub struct UplinkCommandHeader {
    #[element_byte_length = 1]
    aes256: [u8; 16],
    pub tai_timestamp: u64,
    pub service_port: u16,
    pub command_counter: u16,
}

#[derive(BitfieldsDerive)]
#[bondrewd(default_endianness = "be")]
pub struct DownlinkCommandHeader {
    #[element_byte_length = 1]
    aes256: [u8; 16],
    pub(crate) tai_timestamp: u64,
    pub(crate) service_port: u16,
    pub command_counter: u16,
}

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
#[bondrewd_enum(u8)]
pub enum CcsdsPacketVersion {
    One,
    Two,
    #[invalid]
    Invalid,
}

#[derive(BitfieldsDerive, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be", enforce_bytes = 6)]
pub struct CcsdsPacketHeader {
    #[bondrewd(enum_primitive = "u8", bit_length = 3)]
    pub(crate) packet_version_number: CcsdsPacketVersion,
    pub(crate) packet_type: bool,
    pub(crate) sec_hdr_flag: bool,
    #[bit_length = 11]
    pub(crate) app_process_id: u16,
    #[bondrewd(enum_primitive = "u8", bit_length = 2)]
    pub(crate) sequence_flags: CcsdsPacketSequenceFlags,
    #[bit_length = 14]
    pub(crate) packet_seq_count: u16,
    pub(crate) packet_data_length: u16,
}

#[test]
fn max_packet() {
    let packet = CcsdsPacketHeader {
        packet_version_number: CcsdsPacketVersion::Invalid,
        packet_type: true,
        sec_hdr_flag: true,
        app_process_id: u16::MAX & 0b0000011111111111,
        sequence_flags: CcsdsPacketSequenceFlags::Unsegmented,
        packet_seq_count: u16::MAX & 0b0011111111111111,
        packet_data_length: u16::MAX,
    };
    assert_eq!(
        CcsdsPacketHeader::from_bytes(packet.clone().into_bytes()),
        packet
    );
}
