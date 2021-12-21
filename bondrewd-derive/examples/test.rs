use bondrewd::{BitfieldEnum, Bitfields};
#[cfg(feature = "slice_fns")]
use bondrewd::BitfieldSliceError;

#[derive(BitfieldEnum, Clone, PartialEq, Eq, Debug)]
pub enum CcsdsPacketSequenceFlags {
    Continuation,
    Start,
    End,
    Unsegmented,
    Invalid(u8),
}

/// 3 bitt field describing the version number of Ccsds standard to use.
#[derive(BitfieldEnum, Clone, PartialEq, Eq, Debug)]
#[bondrewd_enum(u8)]
pub enum CcsdsPacketVersion {
    One,
    Two,
    Invalid,
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be", enforce_bytes = 6)]
pub struct CcsdsPacketHeader {
    #[bondrewd(enum_primitive = "u8", bit_length = 3)]
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

#[derive(Bitfields)]
struct Simple {
    #[bondrewd(bit_length = 3)]
    one: u8,
    #[bondrewd(bit_length = 4)]
    four: u8,
    asdf: [i8;6],
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
