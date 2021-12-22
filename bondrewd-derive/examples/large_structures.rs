use bondrewd::*;

#[derive(Bitfields)]
#[bondrewd(default_endianess = "msb", read_from = "lsb0", enforce_bytes = "1")]
pub struct StatusMagnetometer {
    mtm1: bool,
    mtm2: bool,
    mtm3: bool,
    #[bondrewd(bit_length = 5, reserve)]
    #[allow(dead_code)]
    reserved: u8,
}

#[derive(Bitfields)]
#[bondrewd(default_endianness = "big")]
pub struct Magnetometers {
    pub timestamp: u64,
    #[bondrewd(struct_size = 1)]
    pub status: StatusMagnetometer,
    pub mtm1_xyz: [i16; 3],
    pub mtm2_xyz: [i16; 3],
    pub mtm3_xyz: [f32; 3],
}

fn main() {
    println!(
        "overall bits/bytes used: {}/{}",
        Magnetometers::BIT_SIZE,
        Magnetometers::BYTE_SIZE
    );

    println!("");
}
