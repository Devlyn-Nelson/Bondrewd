#[cfg(feature = "hex_fns")]
mod hex_tests {
    use bondrewd::*;
    #[derive(Bitfields, Clone, Debug, PartialEq)]
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
    }
    #[test]
    fn hex_test(){
        let og = Magnetometer {
            timestamp: 063482412850,
            status: StatusMagnetometer {
                int_mtm1: true,
                int_mtm2: false,
                ext_mtm: true,
            },
            int_mtm1_xyz: [53, 6, 9246],
            int_mtm2_xyz: [876, 29, 678],
            ext_mtm_xyz: [485, 2534, 2],
        };
        let bytes = og.clone().into_bytes();
        let mut hex_from_bytes = String::new();
        for byte in bytes {
            let hex_byte = format!("{:02X}", byte);
            hex_from_bytes.push_str(hex_byte.as_str());
        }
        let hex = og.clone().into_hex_upper();
        let mut hex_string = String::new();
        for hex_char in hex {
            hex_string.push(hex_char as char);
        }
        assert_eq!(hex_string, hex_from_bytes);

        let from_bytes_obj = Magnetometer::from_bytes(bytes);
        assert_eq!(from_bytes_obj, og);

        let from_hex_obj = if let Ok(m) = Magnetometer::from_hex(hex) { m } else { panic!("Bad decode")};
        assert_eq!(from_hex_obj, og);

    }
}