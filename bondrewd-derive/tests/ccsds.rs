use bondrewd_test as bondrewd;
use bondrewd::Bitfields;

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(id_bit_length = 2, default_endianness = "be")]
pub enum SpacePacketSequenceFlags {
    Continuation,
    Start,
    End,
    Unsegmented,
}

/// 3 bitt field describing the version number of Ccsds standard to use.
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(id_bit_length = 3, default_endianness = "be")]
pub enum SpacePacketVersion {
    One,
    Two,
    Invalid,
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be", enforce_bytes = 6, dump)]
pub struct SpacePacketHeader {
    #[bondrewd(bit_length = 3)]
    pub(crate) packet_version_number: SpacePacketVersion,
    pub(crate) packet_type: bool,
    pub(crate) sec_hdr_flag: bool,
    #[bondrewd(bit_length = 11)]
    pub(crate) app_process_id: u16,
    #[bondrewd(bit_length = 2)]
    pub(crate) sequence_flags: SpacePacketSequenceFlags,
    #[bondrewd(bit_length = 14)]
    pub(crate) packet_seq_count: u16,
    pub(crate) packet_data_length: u16,
}

#[test]
fn max_packet() {
    let packet = SpacePacketHeader {
        packet_version_number: SpacePacketVersion::Invalid,
        packet_type: true,
        sec_hdr_flag: true,
        app_process_id: u16::MAX & 0b0000_0111_1111_1111,
        sequence_flags: SpacePacketSequenceFlags::Unsegmented,
        packet_seq_count: u16::MAX & 0b0011_1111_1111_1111,
        packet_data_length: u16::MAX,
    };
    assert_eq!(
        SpacePacketHeader::from_bytes(packet.clone().into_bytes()),
        packet
    );
}

#[test]
fn slice_fn_check_slice() {
    let packet = SpacePacketHeader {
        packet_version_number: SpacePacketVersion::Invalid,
        packet_type: true,
        sec_hdr_flag: true,
        app_process_id: 0b0000_0111_1111_1111,
        sequence_flags: SpacePacketSequenceFlags::Unsegmented,
        packet_seq_count: 0b0011_1111_1111_1111,
        packet_data_length: u16::MAX,
    };
    let mut bytes = packet.clone().into_bytes();
    match SpacePacketHeader::check_slice(&bytes[..]) {
        Ok(checked) => {
            assert_eq!(
                checked.read_packet_version_number(),
                packet.packet_version_number
            );
            assert_eq!(checked.read_packet_type(), packet.packet_type);
            assert_eq!(checked.read_sec_hdr_flag(), packet.sec_hdr_flag);
            assert_eq!(checked.read_app_process_id(), packet.app_process_id);
            assert_eq!(checked.read_sequence_flags(), packet.sequence_flags);
            assert_eq!(checked.read_packet_seq_count(), packet.packet_seq_count);
            assert_eq!(checked.read_packet_data_length(), packet.packet_data_length);
        }
        Err(err) => panic!("check_slice failed {err}"),
    }
    match SpacePacketHeader::check_slice_mut(&mut bytes[..]) {
        Ok(mut checked) => {
            let packet = SpacePacketHeader {
                packet_version_number: SpacePacketVersion::Two,
                packet_type: false,
                sec_hdr_flag: false,
                app_process_id: 0,
                sequence_flags: SpacePacketSequenceFlags::Start,
                packet_seq_count: 0,
                packet_data_length: 0,
            };
            checked.write_packet_version_number(packet.packet_version_number.clone());
            checked.write_packet_type(packet.packet_type);
            checked.write_sec_hdr_flag(packet.sec_hdr_flag);
            checked.write_app_process_id(packet.app_process_id);
            checked.write_sequence_flags(packet.sequence_flags.clone());
            checked.write_packet_seq_count(packet.packet_seq_count);
            checked.write_packet_data_length(packet.packet_data_length);
            assert_eq!(
                checked.read_packet_version_number(),
                packet.packet_version_number
            );
            assert_eq!(checked.read_packet_type(), packet.packet_type);
            assert_eq!(checked.read_sec_hdr_flag(), packet.sec_hdr_flag);
            assert_eq!(checked.read_app_process_id(), packet.app_process_id);
            assert_eq!(checked.read_sequence_flags(), packet.sequence_flags);
            assert_eq!(checked.read_packet_seq_count(), packet.packet_seq_count);
            assert_eq!(checked.read_packet_data_length(), packet.packet_data_length);
        }
        Err(err) => panic!("check_slice failed {err}"),
    }
}

fn slice_fns_inner() -> Result<(), bondrewd::BitfieldLengthError> {
    use bondrewd::BitfieldsDyn;
    let packet = SpacePacketHeader {
        packet_version_number: SpacePacketVersion::Invalid,
        packet_type: true,
        sec_hdr_flag: true,
        app_process_id: 0b0000_0111_1111_1111,
        sequence_flags: SpacePacketSequenceFlags::Unsegmented,
        packet_seq_count: 0b0011_1111_1111_1111,
        packet_data_length: u16::MAX,
    };
    let mut bytes = packet.clone().into_bytes();
    assert_eq!(
        SpacePacketHeader::read_slice_packet_version_number(&bytes)?,
        packet.packet_version_number
    );
    assert_eq!(
        SpacePacketHeader::read_slice_packet_type(&bytes)?,
        packet.packet_type
    );
    assert_eq!(
        SpacePacketHeader::read_slice_sec_hdr_flag(&bytes)?,
        packet.sec_hdr_flag
    );
    assert_eq!(
        SpacePacketHeader::read_slice_app_process_id(&bytes)?,
        packet.app_process_id
    );
    assert_eq!(
        SpacePacketHeader::read_slice_sequence_flags(&bytes)?,
        packet.sequence_flags
    );
    assert_eq!(
        SpacePacketHeader::read_slice_packet_seq_count(&bytes)?,
        packet.packet_seq_count
    );
    assert_eq!(
        SpacePacketHeader::read_slice_packet_data_length(&bytes)?,
        packet.packet_data_length
    );
    let packet = SpacePacketHeader {
        packet_version_number: SpacePacketVersion::Two,
        packet_type: false,
        sec_hdr_flag: false,
        app_process_id: 0,
        sequence_flags: SpacePacketSequenceFlags::Start,
        packet_seq_count: 0,
        packet_data_length: 0,
    };
    SpacePacketHeader::write_slice_packet_version_number(
        &mut bytes,
        packet.packet_version_number.clone(),
    )?;
    SpacePacketHeader::write_slice_packet_type(&mut bytes, packet.packet_type)?;
    SpacePacketHeader::write_slice_sec_hdr_flag(&mut bytes, packet.sec_hdr_flag)?;
    SpacePacketHeader::write_slice_app_process_id(&mut bytes, packet.app_process_id)?;
    SpacePacketHeader::write_slice_sequence_flags(&mut bytes, packet.sequence_flags.clone())?;
    SpacePacketHeader::write_slice_packet_seq_count(&mut bytes, packet.packet_seq_count)?;
    SpacePacketHeader::write_slice_packet_data_length(&mut bytes, packet.packet_data_length)?;
    assert_eq!(
        SpacePacketHeader::read_slice_packet_version_number(&bytes)?,
        packet.packet_version_number
    );
    assert_eq!(
        SpacePacketHeader::read_slice_packet_type(&bytes)?,
        packet.packet_type
    );
    assert_eq!(
        SpacePacketHeader::read_slice_sec_hdr_flag(&bytes)?,
        packet.sec_hdr_flag
    );
    assert_eq!(
        SpacePacketHeader::read_slice_app_process_id(&bytes)?,
        packet.app_process_id
    );
    assert_eq!(
        SpacePacketHeader::read_slice_sequence_flags(&bytes)?,
        packet.sequence_flags
    );
    assert_eq!(
        SpacePacketHeader::read_slice_packet_seq_count(&bytes)?,
        packet.packet_seq_count
    );
    assert_eq!(
        SpacePacketHeader::read_slice_packet_data_length(&bytes)?,
        packet.packet_data_length
    );
    let mut bytes = bytes.to_vec();
    let new_slice = SpacePacketHeader::from_slice(&bytes)?;
    let new_vec = SpacePacketHeader::from_vec(&mut bytes)?;
    assert_eq!(new_slice, new_vec);
    assert_eq!(new_slice, packet);
    Ok(())
}

#[test]
fn dyn_fns() {
    if let Err(err) = slice_fns_inner() {
        panic!("{err}");
    }
}
