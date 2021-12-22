use bondrewd::*;

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

/*#[derive(Bitfields, Clone, Debug, PartialEq)]
#[bondrewd(read_from = "lsb0", enforce_bits = 3)]
pub struct StatusMagnetometer {
    int_mtm1: bool,
    int_mtm2: bool,
    ext_mtm: bool,
}

/// Response to a Get Mtm Reading command - returns data in the Telemetry::Magnetometer format (separate as non-unit enums are not supported by GraphQL)
/// This includes status and readings for all magnetometers
#[derive(Bitfields, Clone, Debug, PartialEq)]
#[bondrewd(default_endianness = "msb")]
pub struct Magnetometer {
    pub timestamp: u64,
    #[bondrewd(struct_size = 1)]
    pub status: StatusMagnetometer,
    #[bondrewd(block_byte_length = 6)]
    pub int_mtm1_xyz: [i16; 3],
    #[bondrewd(block_byte_length = 6)]
    pub int_mtm2_xyz: [i16; 3],
    #[bondrewd(block_byte_length = 6)]
    pub ext_mtm_xyz: [i16; 3],
}*/

#[derive(Bitfields, Clone, PartialEq, Debug)]
#[bondrewd(default_endianness = "le")]
struct SimpleWithFloats {
    #[bondrewd(bit_length = 27)]
    one: f32,
    #[bondrewd(bit_length = 60)]
    two: f64,
    #[bondrewd(bit_length = 19)]
    three: f32,
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
