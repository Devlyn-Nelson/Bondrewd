use bondrewd::Bitfields;

#[derive(Clone, Bitfields, PartialEq, Eq, Copy, Debug, PartialOrd, Ord)]
#[bondrewd(id_bit_length = 6, default_endianness = "be")]
pub enum AosFrameVirtualChannelId {
    /// Denotes that the `AosFrame` belongs to the Orbital or Non-Realtime Virtual Channel. value of 0.
    Orbital,
    /// Denotes that the `AosFrame` belongs to Realtime Virtual Channel. value of 1.
    Realtime,
    /// Denotes that the `AosFrame` Virtual Channel is not within the Pumpkin Inc spec values.
    Invalid {
        #[bondrewd(capture_id)]
        id: u8,
    },
}

/// A Structure containing all of the information of a AOS Space Data Link Header (CCSDS 732.0-B-4 4.1.2)
/// for AOS Space Data Link Protocol in native rust typing. Bondrewd Bitfields are derived which means we
/// can easily convert this from/into a fixed size array of bytes.
#[derive(Clone, Bitfields, Debug)]
#[bondrewd(default_endianness = "msb", bit_traversal = "front", enforce_bytes = 8)]
pub struct AosFrameHeaderBe {
    /// AOS Space Data Link Protocol `Transfer Frame Version Number` (CCSDS 732.0-B-4 4.1.2.2.2).
    #[bondrewd(bit_length = 2)]
    pub transfer_frame_version: u8,
    /// AOS Space Data Link Protocol `Spacecraft Identifier` (CCSDS 732.0-B-4 4.1.2.2.3).
    pub space_craft_id: u8,
    /// AOS Space Data Link Protocol `Virtual Channel Identifier` (CCSDS 732.0-B-4 4.1.2.3).
    #[bondrewd(bit_length = 6)]
    pub vcid: AosFrameVirtualChannelId,
    /// AOS Space Data Link Protocol `Virtual Channel Frame Count` (CCSDS 732.0-B-4 4.1.2.4).
    #[bondrewd(bit_length = 24)]
    pub virtual_channel_frame_count: u32,
    /// AOS Space Data Link Protocol `Replay Flag` (CCSDS 732.0-B-4 4.1.2.5.2).
    pub replay_flag: bool,
    /// AOS Space Data Link Protocol `Virtual Channel Frame Count Cycle Use Flag` (CCSDS 732.0-B-4 4.1.2.5.3).
    pub vc_frame_count_usage: bool,
    /// AOS Space Data Link Protocol `Reserved Spare` (CCSDS 732.0-B-4 4.1.2.5.4). we make this read-only
    /// because we like to use it as a parsing check.
    #[bondrewd(bit_length = 2, read_only)]
    pub reserved: u8,
    /// AOS Space Data Link Protocol `Virtual Channel Frame Count Cycle` (CCSDS 732.0-B-4 4.1.2.5.5).
    #[bondrewd(bit_length = 4)]
    pub vc_frame_count_cycle: u8,
    /// AOS Space Data Link Protocol Multiplexing Protocol Data Unit `Reserved Spare`
    /// (CCSDS 732.0-B-4 4.1.4.2.2). we make this read-only because we like to use it as a parsing check.
    #[bondrewd(bit_length = 5, read_only)]
    pub reserved_spare: u8,
    /// AOS Space Data Link Protocol Multiplexing Protocol Data Unit `First Header Pointer` (CCSDS 732.0-B-4 4.1.4.2.3).
    #[bondrewd(bit_length = 11)]
    pub first_header_pointer: u16,
}

#[test]
fn cycle_header_be() {
    let mut header = AosFrameHeaderBe {
        transfer_frame_version: 0,
        space_craft_id: 55,
        vcid: AosFrameVirtualChannelId::Orbital,
        virtual_channel_frame_count: 0,
        replay_flag: false,
        vc_frame_count_usage: false,
        reserved: 0,
        vc_frame_count_cycle: 0,
        reserved_spare: 0,
        first_header_pointer: 0xff,
    };

    let mut bytes = header.clone().into_bytes();

    let end = 2_u32.pow(24) - 1;
    while header.virtual_channel_frame_count != end {
        header.virtual_channel_frame_count += 1;
        AosFrameHeaderBe::write_virtual_channel_frame_count(
            &mut bytes,
            header.virtual_channel_frame_count,
        );
        assert_eq!(
            header.virtual_channel_frame_count,
            AosFrameHeaderBe::read_virtual_channel_frame_count(&bytes)
        );
    }
}
#[test]
fn cycle_header_be_slice() {
    let mut header = AosFrameHeaderBe {
        transfer_frame_version: 0,
        space_craft_id: 55,
        vcid: AosFrameVirtualChannelId::Orbital,
        virtual_channel_frame_count: 0,
        replay_flag: false,
        vc_frame_count_usage: false,
        reserved: 0,
        vc_frame_count_cycle: 0,
        reserved_spare: 0,
        first_header_pointer: 0xff,
    };

    let mut bytes = header.clone().into_bytes();

    let end = 2_u32.pow(24) - 1;
    while header.virtual_channel_frame_count != end {
        header.virtual_channel_frame_count += 1;
        let _ = AosFrameHeaderBe::write_slice_virtual_channel_frame_count(
            &mut bytes,
            header.virtual_channel_frame_count,
        );
        assert_eq!(
            header.virtual_channel_frame_count,
            AosFrameHeaderBe::read_slice_virtual_channel_frame_count(&bytes).unwrap()
        );
    }
}

/// A Structure containing all of the information of a AOS Space Data Link Header (CCSDS 732.0-B-4 4.1.2)
/// for AOS Space Data Link Protocol in native rust typing. Bondrewd Bitfields are derived which means we
/// can easily convert this from/into a fixed size array of bytes.
#[derive(Clone, Bitfields, Debug)]
#[bondrewd(default_endianness = "le", bit_traversal = "front", enforce_bytes = 8)]
pub struct AosFrameHeaderLe {
    /// AOS Space Data Link Protocol `Transfer Frame Version Number` (CCSDS 732.0-B-4 4.1.2.2.2).
    #[bondrewd(bit_length = 2)]
    pub transfer_frame_version: u8,
    /// AOS Space Data Link Protocol `Spacecraft Identifier` (CCSDS 732.0-B-4 4.1.2.2.3).
    pub space_craft_id: u8,
    /// AOS Space Data Link Protocol `Virtual Channel Identifier` (CCSDS 732.0-B-4 4.1.2.3).
    #[bondrewd(bit_length = 6)]
    pub vcid: AosFrameVirtualChannelId,
    /// AOS Space Data Link Protocol `Virtual Channel Frame Count` (CCSDS 732.0-B-4 4.1.2.4).
    #[bondrewd(bit_length = 24)]
    pub virtual_channel_frame_count: u32,
    /// AOS Space Data Link Protocol `Replay Flag` (CCSDS 732.0-B-4 4.1.2.5.2).
    pub replay_flag: bool,
    /// AOS Space Data Link Protocol `Virtual Channel Frame Count Cycle Use Flag` (CCSDS 732.0-B-4 4.1.2.5.3).
    pub vc_frame_count_usage: bool,
    /// AOS Space Data Link Protocol `Reserved Spare` (CCSDS 732.0-B-4 4.1.2.5.4). we make this read-only
    /// because we like to use it as a parsing check.
    #[bondrewd(bit_length = 2, read_only)]
    pub reserved: u8,
    /// AOS Space Data Link Protocol `Virtual Channel Frame Count Cycle` (CCSDS 732.0-B-4 4.1.2.5.5).
    #[bondrewd(bit_length = 4)]
    pub vc_frame_count_cycle: u8,
    /// AOS Space Data Link Protocol Multiplexing Protocol Data Unit `Reserved Spare`
    /// (CCSDS 732.0-B-4 4.1.4.2.2). we make this read-only because we like to use it as a parsing check.
    #[bondrewd(bit_length = 5, read_only)]
    pub reserved_spare: u8,
    /// AOS Space Data Link Protocol Multiplexing Protocol Data Unit `First Header Pointer` (CCSDS 732.0-B-4 4.1.4.2.3).
    #[bondrewd(bit_length = 11)]
    pub first_header_pointer: u16,
}

#[test]
fn cycle_header_le() {
    let mut header = AosFrameHeaderLe {
        transfer_frame_version: 0,
        space_craft_id: 55,
        vcid: AosFrameVirtualChannelId::Orbital,
        virtual_channel_frame_count: 0,
        replay_flag: false,
        vc_frame_count_usage: false,
        reserved: 0,
        vc_frame_count_cycle: 0,
        reserved_spare: 0,
        first_header_pointer: 0xff,
    };

    let mut bytes = header.clone().into_bytes();

    let end = 2_u32.pow(24) - 1;
    while header.virtual_channel_frame_count != end {
        header.virtual_channel_frame_count += 1;
        AosFrameHeaderLe::write_virtual_channel_frame_count(
            &mut bytes,
            header.virtual_channel_frame_count,
        );
        assert_eq!(
            header.virtual_channel_frame_count,
            AosFrameHeaderLe::read_virtual_channel_frame_count(&bytes)
        );
    }
}

#[test]
fn cycle_header_le_slice() {
    let mut header = AosFrameHeaderLe {
        transfer_frame_version: 0,
        space_craft_id: 55,
        vcid: AosFrameVirtualChannelId::Orbital,
        virtual_channel_frame_count: 0,
        replay_flag: false,
        vc_frame_count_usage: false,
        reserved: 0,
        vc_frame_count_cycle: 0,
        reserved_spare: 0,
        first_header_pointer: 0xff,
    };

    let mut bytes = header.clone().into_bytes();

    let end = 2_u32.pow(24) - 1;
    while header.virtual_channel_frame_count != end {
        header.virtual_channel_frame_count += 1;
        let _ = AosFrameHeaderLe::write_slice_virtual_channel_frame_count(
            &mut bytes,
            header.virtual_channel_frame_count,
        );
        assert_eq!(
            header.virtual_channel_frame_count,
            AosFrameHeaderLe::read_slice_virtual_channel_frame_count(&bytes).unwrap()
        );
    }
}
