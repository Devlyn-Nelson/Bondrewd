use self::field::SolvedData;

pub mod array_iter;
pub mod field;
pub mod field_set;

#[must_use]
pub fn measure_field_set_bits(fields: &[SolvedData]) -> usize {
    let mut l = 0;
    for f in fields {
        l += f.bit_length();
    }
    l
}
