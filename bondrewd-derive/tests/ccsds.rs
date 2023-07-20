use bondrewd::Bitfields;

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(id_bit_length = 2, default_endianness = "be")]
pub enum CcsdsPacketSequenceFlags {
    Continuation,
    Start,
    End,
    Unsegmented,
}

/// 3 bitt field describing the version number of Ccsds standard to use.
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(id_bit_length = 3, default_endianness = "be")]
pub enum CcsdsPacketVersion {
    One,
    Two,
    Invalid,
}

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be", enforce_bytes = 6)]
pub struct CcsdsPacketHeader {
    #[bondrewd(bit_length = 3)]
    pub(crate) packet_version_number: CcsdsPacketVersion,
    pub(crate) packet_type: bool,
    pub(crate) sec_hdr_flag: bool,
    #[bondrewd(bit_length = 11)]
    pub(crate) app_process_id: u16,
    #[bondrewd(bit_length = 2)]
    pub(crate) sequence_flags: CcsdsPacketSequenceFlags,
    #[bondrewd(bit_length = 14)]
    pub(crate) packet_seq_count: u16,
    pub(crate) packet_data_length: u16,
}

#[test]
fn max_packet() {
    let packet = CcsdsPacketHeader {
        packet_version_number: CcsdsPacketVersion::Invalid,
        packet_type: true,
        sec_hdr_flag: true,
        app_process_id: u16::MAX & 0b0000_0111_1111_1111,
        sequence_flags: CcsdsPacketSequenceFlags::Unsegmented,
        packet_seq_count: u16::MAX & 0b0011_1111_1111_1111,
        packet_data_length: u16::MAX,
    };
    assert_eq!(
        CcsdsPacketHeader::from_bytes(packet.clone().into_bytes()),
        packet
    );
}

#[cfg(feature = "slice_fns")]
#[test]
fn slice_fn_check_slice() {
    let packet = CcsdsPacketHeader {
        packet_version_number: CcsdsPacketVersion::Invalid,
        packet_type: true,
        sec_hdr_flag: true,
        app_process_id: 0b0000_0111_1111_1111,
        sequence_flags: CcsdsPacketSequenceFlags::Unsegmented,
        packet_seq_count: 0b0011_1111_1111_1111,
        packet_data_length: u16::MAX,
    };
    let mut bytes = packet.clone().into_bytes();
    match CcsdsPacketHeader::check_slice(&bytes[..]) {
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
    match CcsdsPacketHeader::check_slice_mut(&mut bytes[..]) {
        Ok(mut checked) => {
            let packet = CcsdsPacketHeader {
                packet_version_number: CcsdsPacketVersion::Two,
                packet_type: false,
                sec_hdr_flag: false,
                app_process_id: 0,
                sequence_flags: CcsdsPacketSequenceFlags::Start,
                packet_seq_count: 0,
                packet_data_length: 0,
            };
            checked.write_packet_version_number(packet.packet_version_number.clone());
            checked.write_packet_type(packet.packet_type.clone());
            checked.write_sec_hdr_flag(packet.sec_hdr_flag.clone());
            checked.write_app_process_id(packet.app_process_id.clone());
            checked.write_sequence_flags(packet.sequence_flags.clone());
            checked.write_packet_seq_count(packet.packet_seq_count.clone());
            checked.write_packet_data_length(packet.packet_data_length.clone());
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

fn slice_fns_inner() -> Result<(), bondrewd::BitfieldSliceError> {
    let packet = CcsdsPacketHeader {
        packet_version_number: CcsdsPacketVersion::Invalid,
        packet_type: true,
        sec_hdr_flag: true,
        app_process_id: 0b0000_0111_1111_1111,
        sequence_flags: CcsdsPacketSequenceFlags::Unsegmented,
        packet_seq_count: 0b0011_1111_1111_1111,
        packet_data_length: u16::MAX,
    };
    let mut bytes = packet.clone().into_bytes();
    assert_eq!(
        CcsdsPacketHeader::read_slice_packet_version_number(&bytes)?,
        packet.packet_version_number
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_packet_type(&bytes)?,
        packet.packet_type
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_sec_hdr_flag(&bytes)?,
        packet.sec_hdr_flag
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_app_process_id(&bytes)?,
        packet.app_process_id
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_sequence_flags(&bytes)?,
        packet.sequence_flags
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_packet_seq_count(&bytes)?,
        packet.packet_seq_count
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_packet_data_length(&bytes)?,
        packet.packet_data_length
    );
    let packet = CcsdsPacketHeader {
        packet_version_number: CcsdsPacketVersion::Two,
        packet_type: false,
        sec_hdr_flag: false,
        app_process_id: 0,
        sequence_flags: CcsdsPacketSequenceFlags::Start,
        packet_seq_count: 0,
        packet_data_length: 0,
    };
    CcsdsPacketHeader::write_slice_packet_version_number(
        &mut bytes,
        packet.packet_version_number.clone(),
    )?;
    CcsdsPacketHeader::write_slice_packet_type(&mut bytes, packet.packet_type.clone())?;
    CcsdsPacketHeader::write_slice_sec_hdr_flag(&mut bytes, packet.sec_hdr_flag.clone())?;
    CcsdsPacketHeader::write_slice_app_process_id(&mut bytes, packet.app_process_id.clone())?;
    CcsdsPacketHeader::write_slice_sequence_flags(&mut bytes, packet.sequence_flags.clone())?;
    CcsdsPacketHeader::write_slice_packet_seq_count(&mut bytes, packet.packet_seq_count.clone())?;
    CcsdsPacketHeader::write_slice_packet_data_length(
        &mut bytes,
        packet.packet_data_length.clone(),
    )?;
    assert_eq!(
        CcsdsPacketHeader::read_slice_packet_version_number(&bytes)?,
        packet.packet_version_number
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_packet_type(&bytes)?,
        packet.packet_type
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_sec_hdr_flag(&bytes)?,
        packet.sec_hdr_flag
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_app_process_id(&bytes)?,
        packet.app_process_id
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_sequence_flags(&bytes)?,
        packet.sequence_flags
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_packet_seq_count(&bytes)?,
        packet.packet_seq_count
    );
    assert_eq!(
        CcsdsPacketHeader::read_slice_packet_data_length(&bytes)?,
        packet.packet_data_length
    );
    Ok(())
}

#[cfg(feature = "slice_fns")]
#[test]
fn slice_fns() {
    if let Err(err) = slice_fns_inner() {
        panic!("{err}");
    }
}
