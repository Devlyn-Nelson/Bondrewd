use self::field::SolvedData;

pub mod field;
pub mod field_set;

pub fn measure_field_set_bits(fields: &[SolvedData]) -> u32 {
    let mut l = 0;
    for f in fields {
        l += f.bit_length() as u32;
    }
    l
}
