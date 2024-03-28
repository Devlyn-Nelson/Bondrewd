use bondrewd::{BitfieldHex, Bitfields};

#[derive(Bitfields, Debug, Clone, PartialEq, Eq)]
#[bondrewd(default_endianness = "msb", bit_traversal = "back", enforce_bytes = 1)]
pub struct StatusMagnetometer {
    mtm1: bool,
    mtm2: bool,
    mtm3: bool,
    #[bondrewd(bit_length = 5, reserve)]
    #[allow(dead_code)]
    reserved: u8,
}

#[derive(Bitfields, Debug, Clone, PartialEq)]
#[bondrewd(default_endianness = "big")]
pub struct Magnetometers {
    pub timestamp: u64,
    #[bondrewd(struct_size = 1)]
    pub status: StatusMagnetometer,
    pub mtm1_xyz: [i16; 3],
    pub mtm2_xyz: [f32; 3],
    #[bondrewd(struct_size = 1)]
    pub mtm3_xyz: [StatusMagnetometer; 3],
}

fn main() {
    println!(
        "overall bits/bytes used: {}/{}",
        Magnetometers::BIT_SIZE,
        Magnetometers::BYTE_SIZE
    );
    let og = Magnetometers {
        timestamp: 168_324,
        status: StatusMagnetometer {
            mtm1: true,
            mtm2: true,
            mtm3: true,
            reserved: 0,
        },
        mtm1_xyz: [-413, -605, 342],
        mtm2_xyz: [52.6, -14.85, -1.2],
        mtm3_xyz: [
            StatusMagnetometer {
                mtm1: true,
                mtm2: true,
                mtm3: true,
                reserved: 0,
            },
            StatusMagnetometer {
                mtm1: true,
                mtm2: true,
                mtm3: true,
                reserved: 0,
            },
            StatusMagnetometer {
                mtm1: true,
                mtm2: true,
                mtm3: true,
                reserved: 0,
            },
        ],
    };
    let bytes = og.clone().into_hex_upper();
    let mut hex_from_bytes = String::new();
    for hex_char in bytes {
        hex_from_bytes.push(hex_char as char);
    }
    match Magnetometers::from_hex(bytes) {
        Ok(mag) => {
            assert_eq!(mag, og);
        }
        Err(err) => {
            panic!("failed paring hex [{err}]");
        }
    }
}
