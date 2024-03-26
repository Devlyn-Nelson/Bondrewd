use crate::common::r#enum::IdPosition;

#[derive(Clone)]
pub struct EnumAttrInfoBuilder {
    pub id_bits: Option<usize>,
    pub id_position: IdPosition,
    pub total_bit_size: Option<usize>,
    pub payload_bit_size: Option<usize>,
}

impl Default for EnumAttrInfoBuilder {
    fn default() -> Self {
        Self {
            id_bits: None,
            id_position: IdPosition::Leading,
            total_bit_size: None,
            payload_bit_size: None,
        }
    }
}
