use self::field::SolvedData;

pub mod field;
pub mod field_set;

#[must_use]
pub fn measure_field_set_bits(fields: &[SolvedData]) -> u32 {
    let mut l = 0;
    for f in fields {
        l += u32::from(f.bit_length());
    }
    l
}
