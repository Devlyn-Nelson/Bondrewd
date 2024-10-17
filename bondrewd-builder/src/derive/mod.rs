use crate::solved::field::ResolverData;

// mod into;

/// Returns a u8 mask with provided `num` amount of 1's on the left side (most significant bit)
pub fn get_left_and_mask(num: usize) -> u8 {
    match num {
        8 => 0b1111_1111,
        7 => 0b1111_1110,
        6 => 0b1111_1100,
        5 => 0b1111_1000,
        4 => 0b1111_0000,
        3 => 0b1110_0000,
        2 => 0b1100_0000,
        1 => 0b1000_0000,
        _ => 0b0000_0000,
    }
}

/// Returns a u8 mask with provided `num` amount of 1's on the right side (least significant bit)
pub fn get_right_and_mask(num: usize) -> u8 {
    match num {
        8 => 0b1111_1111,
        7 => 0b0111_1111,
        6 => 0b0011_1111,
        5 => 0b0001_1111,
        4 => 0b0000_1111,
        3 => 0b0000_0111,
        2 => 0b0000_0011,
        1 => 0b0000_0001,
        _ => 0b0000_0000,
    }
}

/// calculate the starting bit index for a field.
///
/// Returns the index of the byte the first bits of the field
///
/// # Arguments
/// * `amount_of_bits` - amount of bits the field will be after `into_bytes`.
/// * `right_rotation` - amount of bit Rotations to preform on the field. Note if rotation is not needed
///                         to retain all used bits then a shift could be used.
/// * `last_index` - total struct bytes size minus 1.
#[inline]
#[expect(
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
pub fn get_be_starting_index(
    amount_of_bits: usize,
    right_rotation: i8,
    last_index: usize,
) -> Result<usize, String> {
    let first = ((amount_of_bits as f64 - right_rotation as f64) / 8.0f64).ceil() as usize;
    if last_index < first {
        Err("Failed getting the starting index for big endianness, field's type doesn't fix the bit size".to_string())
    } else {
        Ok(last_index - first)
    }
}

impl ResolverData {
    /// Returns the next byte index in sequence based of the given `index` and whether or not the Structure in has a reverse bytes order.
    pub fn next_index(&self, index: usize) -> usize {
        if self.flip.is_some() {
            index - 1
        } else {
            index + 1
        }
    }
    /// Returns the `starting_inject_byte` plus or minus `offset` depending on if the bytes order is reversed.
    pub fn offset_starting_inject_byte(&self, offset: usize) -> usize {
        if self.flip.is_some() {
            self.starting_inject_byte - offset
        } else {
            self.starting_inject_byte + offset
        }
    }
    pub fn fields_last_bits_index(&self) -> usize {
        self.bit_range_end().div_ceil(8) - 1
    }
    pub fn flip(&self) -> Option<usize> {
        self.flip
    }
    pub fn bit_range_start(&self) -> usize {
        self.bit_range.start
    }
    pub fn bit_range_end(&self) -> usize {
        self.bit_range.end
    }
    #[must_use]
    pub fn bit_length(&self) -> usize {
        self.bit_range.end - self.bit_range.start
    }
}

pub struct ResolverDataLittleAdditive {
    pub right_shift: i8,
    pub first_bit_mask: u8,
    pub last_bit_mask: u8,
}
impl From<&ResolverData> for ResolverDataLittleAdditive {
    fn from(qi: &ResolverData) -> Self {
        let amount_of_bits = qi.bit_length();
        let bits_in_last_byte = (amount_of_bits - qi.available_bits_in_first_byte) % 8;
        // how many times to shift the number right.
        // NOTE if negative shift left.
        // NOTE if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        let mut bits_needed_in_msb = amount_of_bits % 8;
        if bits_needed_in_msb == 0 {
            bits_needed_in_msb = 8;
        }
        #[expect(clippy::cast_possible_truncation)]
        let mut right_shift: i8 =
            (bits_needed_in_msb as i8) - ((qi.available_bits_in_first_byte % 8) as i8);
        if right_shift == 8 {
            right_shift = 0;
        }
        // because we are applying bits in place we need masks in insure we don't effect other fields
        // data. we need one for the first byte and the last byte.
        let first_bit_mask = get_right_and_mask(qi.available_bits_in_first_byte);
        let last_bit_mask = if bits_in_last_byte == 0 {
            get_left_and_mask(8)
        } else {
            get_left_and_mask(bits_in_last_byte)
        };
        Self {
            right_shift,
            first_bit_mask,
            last_bit_mask,
        }
    }
}
pub struct ResolverDataNestedAdditive {
    pub right_shift: i8,
}
impl From<&ResolverData> for ResolverDataNestedAdditive {
    fn from(quote_info: &ResolverData) -> Self {
        #[expect(clippy::cast_possible_truncation)]
        let right_shift: i8 = 8_i8 - ((quote_info.available_bits_in_first_byte % 8) as i8);
        Self { right_shift }
    }
}

pub struct ResolverDataBigAdditive {
    pub right_shift: i8,
    pub first_bit_mask: u8,
    pub last_bit_mask: u8,
    pub bits_in_last_byte: usize,
}
impl From<&ResolverData> for ResolverDataBigAdditive {
    fn from(qi: &ResolverData) -> Self {
        let amount_of_bits = qi.bit_length();
        let bits_in_last_byte = (amount_of_bits - qi.available_bits_in_first_byte) % 8;
        // how many times to shift the number right.
        // NOTE if negative shift left.
        // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        #[allow(clippy::cast_possible_truncation)]
        let mut right_shift: i8 =
            ((amount_of_bits % 8) as i8) - ((qi.available_bits_in_first_byte % 8) as i8);
        if right_shift < 0 {
            right_shift += 8;
        }
        // because we are applying bits in place we need masks in insure we don't effect other fields
        // data. we need one for the first byte and the last byte.
        let first_bit_mask = get_right_and_mask(qi.available_bits_in_first_byte);
        let last_bit_mask = if bits_in_last_byte == 0 {
            get_left_and_mask(8)
        } else {
            get_left_and_mask(bits_in_last_byte)
        };
        Self {
            right_shift,
            first_bit_mask,
            last_bit_mask,
            bits_in_last_byte,
        }
    }
}
