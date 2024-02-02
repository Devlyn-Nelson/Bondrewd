pub trait BitfieldHex<const HEX_SIZE: usize, const BYTE_SIZE: usize>:
    crate::Bitfields<BYTE_SIZE>
where
    Self: Sized,
{
    const UPPERS: &'static [u8; 16] = b"0123456789ABCDEF";
    const LOWERS: &'static [u8; 16] = b"0123456789abcdef";
    const HEX_SIZE: usize = HEX_SIZE;
    /// Extracts the values of the Bitfields in this structure from a hex encoded fixed size byte array
    /// while consuming it.
    ///
    /// Returns Self with the fields containing the extracted values from provided hex encoded fixed size
    /// array of bytes.
    fn from_hex(hex: [u8; HEX_SIZE]) -> Result<Self, crate::BitfieldHexError> {
        let mut bytes: [u8; BYTE_SIZE] = [0; BYTE_SIZE];
        for i in 0usize..BYTE_SIZE {
            let index = i * 2;
            let index2 = index + 1;
            let decode_nibble = |c, c_i| match c {
                b'A'..=b'F' => Ok(c - b'A' + 10u8),
                b'a'..=b'f' => Ok(c - b'a' + 10u8),
                b'0'..=b'9' => Ok(c - b'0'),
                _ => Err(crate::BitfieldHexError(c as char, c_i)),
            };
            bytes[i] = ((decode_nibble(hex[index], index)? & 0b0000_1111) << 4)
                | decode_nibble(hex[index2], index2)?;
        }
        Ok(Self::from_bytes(bytes))
    }
    /// Inserts the values of the Bitfields in this structure into a fixed size array with upper case hex
    /// encoding, consuming the structure.
    ///
    /// Returns a hex encoded fixed sized byte array containing the Bitfields of the provided structure.
    fn into_hex_upper(self) -> [u8; HEX_SIZE] {
        let bytes = self.into_bytes();
        let mut output: [u8; HEX_SIZE] = [0; HEX_SIZE];
        for (i, byte) in (0..HEX_SIZE).step_by(2).zip(bytes) {
            output[i] = Self::UPPERS[((byte & 0b1111_0000) >> 4) as usize];
            output[i + 1] = Self::UPPERS[(byte & 0b0000_1111) as usize];
        }
        output
    }
    /// Inserts the values of the Bitfields in this structure into a fixed size array with lower case hex
    /// encoding, consuming the structure.
    ///
    /// Returns a hex encoded fixed sized byte array containing the Bitfields of the provided structure.
    fn into_hex_lower(self) -> [u8; HEX_SIZE] {
        let bytes = self.into_bytes();
        let mut output: [u8; HEX_SIZE] = [0; HEX_SIZE];
        for (i, byte) in (0..HEX_SIZE).step_by(2).zip(bytes) {
            output[i] = Self::LOWERS[((byte & 0b1111_0000) >> 4) as usize];
            output[i + 1] = Self::LOWERS[(byte & 0b0000_1111) as usize];
        }
        output
    }
}

#[cfg(feature = "dyn_fns")]
pub trait BitfieldHexDyn<const HEX_SIZE: usize, const BYTE_SIZE: usize>:
    crate::Bitfields<BYTE_SIZE>
where
    Self: Sized,
{
    const UPPERS: &'static [u8; 16] = b"0123456789ABCDEF";
    const LOWERS: &'static [u8; 16] = b"0123456789abcdef";
    fn from_hex_vec(hex: &mut Vec<u8>) -> Result<Self, crate::BitfieldHexDynError> {
        if hex.len() < HEX_SIZE {
            return Err(crate::BitfieldHexDynError::Length(
                crate::BitfieldLengthError(hex.len(), HEX_SIZE),
            ));
        }
        let mut bytes: [u8; BYTE_SIZE] = [0; BYTE_SIZE];
        for i in 0usize..BYTE_SIZE {
            let index = i * 2;
            let index2 = index + 1;
            let decode_nibble = |c, c_i| match c {
                b'A'..=b'F' => Ok(c - b'A' + 10u8),
                b'a'..=b'f' => Ok(c - b'a' + 10u8),
                b'0'..=b'9' => Ok(c - b'0'),
                _ => {
                    Err(crate::BitfieldHexDynError::Hex(crate::BitfieldHexError(
                        c as char, c_i,
                    )))
                }
            };
            bytes[i] = ((decode_nibble(hex[index], index)? & 0b0000_1111) << 4)
                | decode_nibble(hex[index2], index2)?;
        }
        Ok(Self::from_bytes(bytes))
    }
    fn from_hex_slice(hex: &[u8]) -> Result<Self, crate::BitfieldHexDynError> {
        if hex.len() < HEX_SIZE {
            return Err(crate::BitfieldHexDynError::Length(
                crate::BitfieldLengthError(hex.len(), HEX_SIZE),
            ));
        }
        let mut bytes: [u8; BYTE_SIZE] = [0; BYTE_SIZE];
        for i in 0usize..BYTE_SIZE {
            let index = i * 2;
            let index2 = index + 1;
            let decode_nibble = |c, c_i| match c {
                b'A'..=b'F' => Ok(c - b'A' + 10u8),
                b'a'..=b'f' => Ok(c - b'a' + 10u8),
                b'0'..=b'9' => Ok(c - b'0'),
                _ => {
                    Err(crate::BitfieldHexDynError::Hex(crate::BitfieldHexError(
                        c as char, c_i,
                    )))
                }
            };
            bytes[i] = ((decode_nibble(hex[index], index)? & 0b0000_1111) << 4)
                | decode_nibble(hex[index2], index2)?;
        }
        Ok(Self::from_bytes(bytes))
    }
}
