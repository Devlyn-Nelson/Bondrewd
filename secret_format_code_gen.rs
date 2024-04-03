impl SecretFormat
{
    #[inline]
    #[doc =
    "Reads bits 0 through 3 within `input_byte_buffer`, getting the `variant_id` field of a `SecretFormat` in bitfield form."]
    pub fn read_variant_id(input_byte_buffer : & [u8; 1usize]) -> u8
    { ((input_byte_buffer [0usize] & 240u8) >> 4usize) as u8 } #[inline]
    #[doc =
    "Writes to bits 0 through 3 within `output_byte_buffer`, setting the `variant_id` field of a `SecretFormat` in bitfield form."]
    pub fn
    write_variant_id(output_byte_buffer : & mut [u8; 1usize], mut variant_id :
    u8)
    {
        output_byte_buffer [0usize] &= 15u8; output_byte_buffer [0usize] |=
        ((variant_id as u8) << 4usize) & 240u8;
    } #[inline]
    #[doc =
    "Returns the value for the `variant_id` field of a `SecretFormat` in bitfield form by reading  bits 0 through 3 in `input_byte_buffer`. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned if not enough bytes are present."]
    pub fn read_slice_variant_id(input_byte_buffer : & [u8]) -> Result < u8,
    bondrewd :: BitfieldLengthError >
    {
        let slice_length = input_byte_buffer.len(); if slice_length < 1usize
        { Err(bondrewd :: BitfieldLengthError(slice_length, 1usize)) } else
        { Ok(((input_byte_buffer [0usize] & 240u8) >> 4usize) as u8) }
    } #[inline]
    #[doc =
    "Writes to bits 0 through 3 in `input_byte_buffer` if enough bytes are present in slice, setting the `variant_id` field of a `SecretFormat` in bitfield form. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned"]
    pub fn
    write_slice_variant_id(output_byte_buffer : & mut [u8], variant_id : u8)
    -> Result < (), bondrewd :: BitfieldLengthError >
    {
        let slice_length = output_byte_buffer.len(); if slice_length < 1usize
        { Err(bondrewd :: BitfieldLengthError(slice_length, 1usize)) } else
        {
            output_byte_buffer [0usize] &= 15u8; output_byte_buffer [0usize]
            |= ((variant_id as u8) << 4usize) & 240u8; Ok(())
        }
    } #[inline]
    #[doc =
    "Reads bits 4 through 7 within `input_byte_buffer`, getting the `zero_bondrewd_fill_bits` field of a `Zero` in bitfield form."]
    pub fn read_zero_bondrewd_fill_bits(input_byte_buffer : & [u8; 1usize]) ->
    [u8; 1usize]
    { [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },] } #[inline]
    #[doc =
    "Returns the value for the `zero_bondrewd_fill_bits` field of a `Zero` in bitfield form by reading  bits 4 through 7 in `input_byte_buffer`. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned if not enough bytes are present."]
    pub fn read_slice_zero_bondrewd_fill_bits(input_byte_buffer : & [u8]) ->
    Result < [u8; 1usize], bondrewd :: BitfieldLengthError >
    {
        let slice_length = input_byte_buffer.len(); if slice_length < 1usize
        { Err(bondrewd :: BitfieldLengthError(slice_length, 1usize)) } else
        { Ok([{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]) }
    }
    #[doc =
    "Returns a [SecretFormatZeroChecked] which allows you to read any field for a `SecretFormatZero::Zero` from provided slice."]
    pub fn check_slice_zero(buffer : & [u8]) -> Result <
    SecretFormatZeroChecked, bondrewd :: BitfieldLengthError >
    {
        let buf_len = buffer.len(); if buf_len >= 1usize
        { Ok(SecretFormatZeroChecked { buffer }) } else
        { Err(bondrewd :: BitfieldLengthError(buf_len, 1usize)) }
    }
    #[doc =
    "Returns a [SecretFormatZeroCheckedMut] which allows you to read/write any field for a `SecretFormatZero::Zero` from/to provided mutable slice."]
    pub fn check_slice_mut_zero(buffer : & mut [u8]) -> Result <
    SecretFormatZeroCheckedMut, bondrewd :: BitfieldLengthError >
    {
        let buf_len = buffer.len(); if buf_len >= 1usize
        { Ok(SecretFormatZeroCheckedMut { buffer }) } else
        { Err(bondrewd :: BitfieldLengthError(buf_len, 1usize)) }
    } pub const ZERO_BYTE_SIZE : usize = 1usize; pub const ZERO_BIT_SIZE :
    usize = 4usize; #[inline]
    #[doc =
    "Reads bits 4 through 7 within `input_byte_buffer`, getting the `one_bondrewd_fill_bits` field of a `One` in bitfield form."]
    pub fn read_one_bondrewd_fill_bits(input_byte_buffer : & [u8; 1usize]) ->
    [u8; 1usize]
    { [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },] } #[inline]
    #[doc =
    "Returns the value for the `one_bondrewd_fill_bits` field of a `One` in bitfield form by reading  bits 4 through 7 in `input_byte_buffer`. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned if not enough bytes are present."]
    pub fn read_slice_one_bondrewd_fill_bits(input_byte_buffer : & [u8]) ->
    Result < [u8; 1usize], bondrewd :: BitfieldLengthError >
    {
        let slice_length = input_byte_buffer.len(); if slice_length < 1usize
        { Err(bondrewd :: BitfieldLengthError(slice_length, 1usize)) } else
        { Ok([{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]) }
    }
    #[doc =
    "Returns a [SecretFormatOneChecked] which allows you to read any field for a `SecretFormatOne::One` from provided slice."]
    pub fn check_slice_one(buffer : & [u8]) -> Result <
    SecretFormatOneChecked, bondrewd :: BitfieldLengthError >
    {
        let buf_len = buffer.len(); if buf_len >= 1usize
        { Ok(SecretFormatOneChecked { buffer }) } else
        { Err(bondrewd :: BitfieldLengthError(buf_len, 1usize)) }
    }
    #[doc =
    "Returns a [SecretFormatOneCheckedMut] which allows you to read/write any field for a `SecretFormatOne::One` from/to provided mutable slice."]
    pub fn check_slice_mut_one(buffer : & mut [u8]) -> Result <
    SecretFormatOneCheckedMut, bondrewd :: BitfieldLengthError >
    {
        let buf_len = buffer.len(); if buf_len >= 1usize
        { Ok(SecretFormatOneCheckedMut { buffer }) } else
        { Err(bondrewd :: BitfieldLengthError(buf_len, 1usize)) }
    } pub const ONE_BYTE_SIZE : usize = 1usize; pub const ONE_BIT_SIZE : usize
    = 4usize; #[inline]
    #[doc =
    "Reads bits 4 through 7 within `input_byte_buffer`, getting the `two_bondrewd_fill_bits` field of a `Two` in bitfield form."]
    pub fn read_two_bondrewd_fill_bits(input_byte_buffer : & [u8; 1usize]) ->
    [u8; 1usize]
    { [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },] } #[inline]
    #[doc =
    "Returns the value for the `two_bondrewd_fill_bits` field of a `Two` in bitfield form by reading  bits 4 through 7 in `input_byte_buffer`. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned if not enough bytes are present."]
    pub fn read_slice_two_bondrewd_fill_bits(input_byte_buffer : & [u8]) ->
    Result < [u8; 1usize], bondrewd :: BitfieldLengthError >
    {
        let slice_length = input_byte_buffer.len(); if slice_length < 1usize
        { Err(bondrewd :: BitfieldLengthError(slice_length, 1usize)) } else
        { Ok([{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]) }
    }
    #[doc =
    "Returns a [SecretFormatTwoChecked] which allows you to read any field for a `SecretFormatTwo::Two` from provided slice."]
    pub fn check_slice_two(buffer : & [u8]) -> Result <
    SecretFormatTwoChecked, bondrewd :: BitfieldLengthError >
    {
        let buf_len = buffer.len(); if buf_len >= 1usize
        { Ok(SecretFormatTwoChecked { buffer }) } else
        { Err(bondrewd :: BitfieldLengthError(buf_len, 1usize)) }
    }
    #[doc =
    "Returns a [SecretFormatTwoCheckedMut] which allows you to read/write any field for a `SecretFormatTwo::Two` from/to provided mutable slice."]
    pub fn check_slice_mut_two(buffer : & mut [u8]) -> Result <
    SecretFormatTwoCheckedMut, bondrewd :: BitfieldLengthError >
    {
        let buf_len = buffer.len(); if buf_len >= 1usize
        { Ok(SecretFormatTwoCheckedMut { buffer }) } else
        { Err(bondrewd :: BitfieldLengthError(buf_len, 1usize)) }
    } pub const TWO_BYTE_SIZE : usize = 1usize; pub const TWO_BIT_SIZE : usize
    = 4usize; #[inline]
    #[doc =
    "Reads bits 4 through 7 within `input_byte_buffer`, getting the `three_bondrewd_fill_bits` field of a `Three` in bitfield form."]
    pub fn read_three_bondrewd_fill_bits(input_byte_buffer : & [u8; 1usize])
    -> [u8; 1usize]
    { [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },] } #[inline]
    #[doc =
    "Returns the value for the `three_bondrewd_fill_bits` field of a `Three` in bitfield form by reading  bits 4 through 7 in `input_byte_buffer`. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned if not enough bytes are present."]
    pub fn read_slice_three_bondrewd_fill_bits(input_byte_buffer : & [u8]) ->
    Result < [u8; 1usize], bondrewd :: BitfieldLengthError >
    {
        let slice_length = input_byte_buffer.len(); if slice_length < 1usize
        { Err(bondrewd :: BitfieldLengthError(slice_length, 1usize)) } else
        { Ok([{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]) }
    }
    #[doc =
    "Returns a [SecretFormatThreeChecked] which allows you to read any field for a `SecretFormatThree::Three` from provided slice."]
    pub fn check_slice_three(buffer : & [u8]) -> Result <
    SecretFormatThreeChecked, bondrewd :: BitfieldLengthError >
    {
        let buf_len = buffer.len(); if buf_len >= 1usize
        { Ok(SecretFormatThreeChecked { buffer }) } else
        { Err(bondrewd :: BitfieldLengthError(buf_len, 1usize)) }
    }
    #[doc =
    "Returns a [SecretFormatThreeCheckedMut] which allows you to read/write any field for a `SecretFormatThree::Three` from/to provided mutable slice."]
    pub fn check_slice_mut_three(buffer : & mut [u8]) -> Result <
    SecretFormatThreeCheckedMut, bondrewd :: BitfieldLengthError >
    {
        let buf_len = buffer.len(); if buf_len >= 1usize
        { Ok(SecretFormatThreeCheckedMut { buffer }) } else
        { Err(bondrewd :: BitfieldLengthError(buf_len, 1usize)) }
    } pub const THREE_BYTE_SIZE : usize = 1usize; pub const THREE_BIT_SIZE :
    usize = 4usize; #[inline]
    #[doc =
    "Reads bits 4 through 7 within `input_byte_buffer`, getting the `invalid_bondrewd_fill_bits` field of a `Invalid` in bitfield form."]
    pub fn read_invalid_bondrewd_fill_bits(input_byte_buffer : & [u8; 1usize])
    -> [u8; 1usize]
    { [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },] } #[inline]
    #[doc =
    "Returns the value for the `invalid_bondrewd_fill_bits` field of a `Invalid` in bitfield form by reading  bits 4 through 7 in `input_byte_buffer`. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned if not enough bytes are present."]
    pub fn read_slice_invalid_bondrewd_fill_bits(input_byte_buffer : & [u8])
    -> Result < [u8; 1usize], bondrewd :: BitfieldLengthError >
    {
        let slice_length = input_byte_buffer.len(); if slice_length < 1usize
        { Err(bondrewd :: BitfieldLengthError(slice_length, 1usize)) } else
        { Ok([{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]) }
    }
    #[doc =
    "Returns a [SecretFormatInvalidChecked] which allows you to read any field for a `SecretFormatInvalid::Invalid` from provided slice."]
    pub fn check_slice_invalid(buffer : & [u8]) -> Result <
    SecretFormatInvalidChecked, bondrewd :: BitfieldLengthError >
    {
        let buf_len = buffer.len(); if buf_len >= 1usize
        { Ok(SecretFormatInvalidChecked { buffer }) } else
        { Err(bondrewd :: BitfieldLengthError(buf_len, 1usize)) }
    }
    #[doc =
    "Returns a [SecretFormatInvalidCheckedMut] which allows you to read/write any field for a `SecretFormatInvalid::Invalid` from/to provided mutable slice."]
    pub fn check_slice_mut_invalid(buffer : & mut [u8]) -> Result <
    SecretFormatInvalidCheckedMut, bondrewd :: BitfieldLengthError >
    {
        let buf_len = buffer.len(); if buf_len >= 1usize
        { Ok(SecretFormatInvalidCheckedMut { buffer }) } else
        { Err(bondrewd :: BitfieldLengthError(buf_len, 1usize)) }
    } pub const INVALID_BYTE_SIZE : usize = 1usize; pub const INVALID_BIT_SIZE
    : usize = 4usize;
    #[doc =
    "Returns a checked structure which allows you to read any field for a `SecretFormat` from provided slice."]
    pub fn check_slice(buffer : & [u8]) -> Result < SecretFormatChecked,
    bondrewd :: BitfieldLengthError >
    {
        let variant_id = Self :: read_slice_variant_id(& buffer) ? ; match
        variant_id
        {
            0 =>
            {
                Ok(SecretFormatChecked ::
                Zero(Self :: check_slice_zero(buffer) ?))
            } 1 =>
            {
                Ok(SecretFormatChecked ::
                One(Self :: check_slice_one(buffer) ?))
            } 2 =>
            {
                Ok(SecretFormatChecked ::
                Two(Self :: check_slice_two(buffer) ?))
            } 3 =>
            {
                Ok(SecretFormatChecked ::
                Three(Self :: check_slice_three(buffer) ?))
            } _ =>
            {
                Ok(SecretFormatChecked ::
                Invalid(Self :: check_slice_invalid(buffer) ?))
            }
        }
    }
    #[doc =
    "Returns a checked mutable structure which allows you to read/write any field for a `SecretFormat` from provided mut slice."]
    pub fn check_slice_mut(buffer : & mut [u8]) -> Result <
    SecretFormatCheckedMut, bondrewd :: BitfieldLengthError >
    {
        let variant_id = Self :: read_slice_variant_id(& buffer) ? ; match
        variant_id
        {
            0 =>
            {
                Ok(SecretFormatCheckedMut ::
                Zero(Self :: check_slice_mut_zero(buffer) ?))
            } 1 =>
            {
                Ok(SecretFormatCheckedMut ::
                One(Self :: check_slice_mut_one(buffer) ?))
            } 2 =>
            {
                Ok(SecretFormatCheckedMut ::
                Two(Self :: check_slice_mut_two(buffer) ?))
            } 3 =>
            {
                Ok(SecretFormatCheckedMut ::
                Three(Self :: check_slice_mut_three(buffer) ?))
            } _ =>
            {
                Ok(SecretFormatCheckedMut ::
                Invalid(Self :: check_slice_mut_invalid(buffer) ?))
            }
        }
    } pub fn id(& self) -> u8
    {
        match self
        {
            Self :: Zero { .. } => 0, Self :: One { .. } => 1, Self :: Two
            { .. } => 2, Self :: Three { .. } => 3, Self :: Invalid { .. } =>
            4,
        }
    }
} impl bondrewd :: Bitfields < 1usize > for SecretFormat
{
    const BIT_SIZE : usize = 4usize; fn
    from_bytes(mut input_byte_buffer : [u8; 1usize]) -> Self
    {
        let variant_id = Self :: read_variant_id(& input_byte_buffer); match
        variant_id
        {
            0 => { Self :: Zero } 1 => { Self :: One } 2 => { Self :: Two } 3
            => { Self :: Three } _ => { Self :: Invalid }
        }
    } fn into_bytes(self) -> [u8; 1usize]
    {
        let mut output_byte_buffer = [0u8; 1usize]; match self
        {
            Self :: Zero =>
            { Self :: write_variant_id(& mut output_byte_buffer, 0); } Self ::
            One => { Self :: write_variant_id(& mut output_byte_buffer, 1); }
            Self :: Two =>
            { Self :: write_variant_id(& mut output_byte_buffer, 2); } Self ::
            Three =>
            { Self :: write_variant_id(& mut output_byte_buffer, 3); } Self ::
            Invalid =>
            { Self :: write_variant_id(& mut output_byte_buffer, 4); }
        } output_byte_buffer
    }
} impl bondrewd :: BitfieldHex < 2usize, 1usize > for SecretFormat {} impl
bondrewd :: BitfieldHexDyn < 2usize, 1usize > for SecretFormat {}
#[doc =
"A Structure which provides functions for getting the fields of a [SecretFormatZero] in its bitfield form."]
pub struct SecretFormatZeroChecked < 'a > { buffer : & 'a [u8], } impl < 'a >
SecretFormatZeroChecked < 'a >
{
    #[inline]
    #[doc =
    "Reads bits 4 through 7 in pre-checked slice, getting the `bondrewd_fill_bits` field of a [Zero] in bitfield form."]
    pub fn read_bondrewd_fill_bits(& self) -> [u8; 1usize]
    {
        let input_byte_buffer : & [u8] = self.buffer;
        [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]
    }
    #[doc =
    "Panics if resulting `SecretFormatZeroChecked` does not contain enough bytes to read a field that is attempted to be read."]
    pub fn from_unchecked_slice(data : & 'a [u8]) -> Self
    { Self { buffer : data } }
}
#[doc =
"A Structure which provides functions for getting and setting the fields of a [SecretFormatZero] in its bitfield form."]
pub struct SecretFormatZeroCheckedMut < 'a > { buffer : & 'a mut [u8], } impl
< 'a > SecretFormatZeroCheckedMut < 'a >
{
    #[inline]
    #[doc =
    "Reads bits 4 through 7 in pre-checked slice, getting the `bondrewd_fill_bits` field of a [Zero] in bitfield form."]
    pub fn read_bondrewd_fill_bits(& self) -> [u8; 1usize]
    {
        let input_byte_buffer : & [u8] = self.buffer;
        [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]
    }
    #[doc =
    "Panics if resulting `SecretFormatZeroCheckedMut` does not contain enough bytes to read a field that is attempted to be read or written."]
    pub fn from_unchecked_slice(data : & 'a mut [u8]) -> Self
    { Self { buffer : data } }
}
#[doc =
"A Structure which provides functions for getting the fields of a [SecretFormatOne] in its bitfield form."]
pub struct SecretFormatOneChecked < 'a > { buffer : & 'a [u8], } impl < 'a >
SecretFormatOneChecked < 'a >
{
    #[inline]
    #[doc =
    "Reads bits 4 through 7 in pre-checked slice, getting the `bondrewd_fill_bits` field of a [One] in bitfield form."]
    pub fn read_bondrewd_fill_bits(& self) -> [u8; 1usize]
    {
        let input_byte_buffer : & [u8] = self.buffer;
        [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]
    }
    #[doc =
    "Panics if resulting `SecretFormatOneChecked` does not contain enough bytes to read a field that is attempted to be read."]
    pub fn from_unchecked_slice(data : & 'a [u8]) -> Self
    { Self { buffer : data } }
}
#[doc =
"A Structure which provides functions for getting and setting the fields of a [SecretFormatOne] in its bitfield form."]
pub struct SecretFormatOneCheckedMut < 'a > { buffer : & 'a mut [u8], } impl <
'a > SecretFormatOneCheckedMut < 'a >
{
    #[inline]
    #[doc =
    "Reads bits 4 through 7 in pre-checked slice, getting the `bondrewd_fill_bits` field of a [One] in bitfield form."]
    pub fn read_bondrewd_fill_bits(& self) -> [u8; 1usize]
    {
        let input_byte_buffer : & [u8] = self.buffer;
        [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]
    }
    #[doc =
    "Panics if resulting `SecretFormatOneCheckedMut` does not contain enough bytes to read a field that is attempted to be read or written."]
    pub fn from_unchecked_slice(data : & 'a mut [u8]) -> Self
    { Self { buffer : data } }
}
#[doc =
"A Structure which provides functions for getting the fields of a [SecretFormatTwo] in its bitfield form."]
pub struct SecretFormatTwoChecked < 'a > { buffer : & 'a [u8], } impl < 'a >
SecretFormatTwoChecked < 'a >
{
    #[inline]
    #[doc =
    "Reads bits 4 through 7 in pre-checked slice, getting the `bondrewd_fill_bits` field of a [Two] in bitfield form."]
    pub fn read_bondrewd_fill_bits(& self) -> [u8; 1usize]
    {
        let input_byte_buffer : & [u8] = self.buffer;
        [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]
    }
    #[doc =
    "Panics if resulting `SecretFormatTwoChecked` does not contain enough bytes to read a field that is attempted to be read."]
    pub fn from_unchecked_slice(data : & 'a [u8]) -> Self
    { Self { buffer : data } }
}
#[doc =
"A Structure which provides functions for getting and setting the fields of a [SecretFormatTwo] in its bitfield form."]
pub struct SecretFormatTwoCheckedMut < 'a > { buffer : & 'a mut [u8], } impl <
'a > SecretFormatTwoCheckedMut < 'a >
{
    #[inline]
    #[doc =
    "Reads bits 4 through 7 in pre-checked slice, getting the `bondrewd_fill_bits` field of a [Two] in bitfield form."]
    pub fn read_bondrewd_fill_bits(& self) -> [u8; 1usize]
    {
        let input_byte_buffer : & [u8] = self.buffer;
        [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]
    }
    #[doc =
    "Panics if resulting `SecretFormatTwoCheckedMut` does not contain enough bytes to read a field that is attempted to be read or written."]
    pub fn from_unchecked_slice(data : & 'a mut [u8]) -> Self
    { Self { buffer : data } }
}
#[doc =
"A Structure which provides functions for getting the fields of a [SecretFormatThree] in its bitfield form."]
pub struct SecretFormatThreeChecked < 'a > { buffer : & 'a [u8], } impl < 'a >
SecretFormatThreeChecked < 'a >
{
    #[inline]
    #[doc =
    "Reads bits 4 through 7 in pre-checked slice, getting the `bondrewd_fill_bits` field of a [Three] in bitfield form."]
    pub fn read_bondrewd_fill_bits(& self) -> [u8; 1usize]
    {
        let input_byte_buffer : & [u8] = self.buffer;
        [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]
    }
    #[doc =
    "Panics if resulting `SecretFormatThreeChecked` does not contain enough bytes to read a field that is attempted to be read."]
    pub fn from_unchecked_slice(data : & 'a [u8]) -> Self
    { Self { buffer : data } }
}
#[doc =
"A Structure which provides functions for getting and setting the fields of a [SecretFormatThree] in its bitfield form."]
pub struct SecretFormatThreeCheckedMut < 'a > { buffer : & 'a mut [u8], } impl
< 'a > SecretFormatThreeCheckedMut < 'a >
{
    #[inline]
    #[doc =
    "Reads bits 4 through 7 in pre-checked slice, getting the `bondrewd_fill_bits` field of a [Three] in bitfield form."]
    pub fn read_bondrewd_fill_bits(& self) -> [u8; 1usize]
    {
        let input_byte_buffer : & [u8] = self.buffer;
        [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]
    }
    #[doc =
    "Panics if resulting `SecretFormatThreeCheckedMut` does not contain enough bytes to read a field that is attempted to be read or written."]
    pub fn from_unchecked_slice(data : & 'a mut [u8]) -> Self
    { Self { buffer : data } }
}
#[doc =
"A Structure which provides functions for getting the fields of a [SecretFormatInvalid] in its bitfield form."]
pub struct SecretFormatInvalidChecked < 'a > { buffer : & 'a [u8], } impl < 'a
> SecretFormatInvalidChecked < 'a >
{
    #[inline]
    #[doc =
    "Reads bits 4 through 7 in pre-checked slice, getting the `bondrewd_fill_bits` field of a [Invalid] in bitfield form."]
    pub fn read_bondrewd_fill_bits(& self) -> [u8; 1usize]
    {
        let input_byte_buffer : & [u8] = self.buffer;
        [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]
    }
    #[doc =
    "Panics if resulting `SecretFormatInvalidChecked` does not contain enough bytes to read a field that is attempted to be read."]
    pub fn from_unchecked_slice(data : & 'a [u8]) -> Self
    { Self { buffer : data } }
}
#[doc =
"A Structure which provides functions for getting and setting the fields of a [SecretFormatInvalid] in its bitfield form."]
pub struct SecretFormatInvalidCheckedMut < 'a > { buffer : & 'a mut [u8], }
impl < 'a > SecretFormatInvalidCheckedMut < 'a >
{
    #[inline]
    #[doc =
    "Reads bits 4 through 7 in pre-checked slice, getting the `bondrewd_fill_bits` field of a [Invalid] in bitfield form."]
    pub fn read_bondrewd_fill_bits(& self) -> [u8; 1usize]
    {
        let input_byte_buffer : & [u8] = self.buffer;
        [{ ((input_byte_buffer [0usize] & 15u8) >> 0usize) as u8 },]
    }
    #[doc =
    "Panics if resulting `SecretFormatInvalidCheckedMut` does not contain enough bytes to read a field that is attempted to be read or written."]
    pub fn from_unchecked_slice(data : & 'a mut [u8]) -> Self
    { Self { buffer : data } }
} pub enum SecretFormatChecked < 'a >
{
    Zero(SecretFormatZeroChecked < 'a >), One(SecretFormatOneChecked < 'a >),
    Two(SecretFormatTwoChecked < 'a >),
    Three(SecretFormatThreeChecked < 'a >),
    Invalid(SecretFormatInvalidChecked < 'a >),
} pub enum SecretFormatCheckedMut < 'a >
{
    Zero(SecretFormatZeroCheckedMut < 'a >),
    One(SecretFormatOneCheckedMut < 'a >),
    Two(SecretFormatTwoCheckedMut < 'a >),
    Three(SecretFormatThreeCheckedMut < 'a >),
    Invalid(SecretFormatInvalidCheckedMut < 'a >),
} impl bondrewd :: BitfieldsDyn < 1usize > for SecretFormat
{
    #[doc =
    "Creates a new instance of `Self` by copying field from the bitfields. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned."]
    fn from_slice(input_byte_buffer : & [u8]) -> Result < Self, bondrewd ::
    BitfieldLengthError >
    {
        if input_byte_buffer.len() < Self :: BYTE_SIZE
        {
            return
            Err(bondrewd ::
            BitfieldLengthError(input_byte_buffer.len(), Self :: BYTE_SIZE));
        } let variant_id = Self :: read_slice_variant_id(& input_byte_buffer)
        ? ; let out = match variant_id
        {
            0 => { Self :: Zero } 1 => { Self :: One } 2 => { Self :: Two } 3
            => { Self :: Three } _ => { Self :: Invalid }
        }; Ok(out)
    }
    #[doc =
    "Creates a new instance of `Self` by copying field from the bitfields, removing bytes that where used. \n # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned."]
    fn from_vec(input_byte_buffer : & mut Vec < u8 >) -> Result < Self,
    bondrewd :: BitfieldLengthError >
    {
        if input_byte_buffer.len() < Self :: BYTE_SIZE
        {
            return
            Err(bondrewd ::
            BitfieldLengthError(input_byte_buffer.len(), Self :: BYTE_SIZE));
        } let variant_id = Self :: read_slice_variant_id(& input_byte_buffer)
        ? ; let out = match variant_id
        {
            0 => { Self :: Zero } 1 => { Self :: One } 2 => { Self :: Two } 3
            => { Self :: Three } _ => { Self :: Invalid }
        }; let _ = input_byte_buffer.drain(.. Self :: BYTE_SIZE); Ok(out)
    }
}