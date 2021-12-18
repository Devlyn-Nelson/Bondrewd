# Bondrewd
I use this for work, so if you use this and find a problem i would be more than happy to fix it.
Im also not good at making read me's.
# Purpose
Short answer.

Derive Bit level fields packing/unpacking functions for a rust Structure with the ability to peek at a field without unpacking any other fields.

All generated code is no_std capable and 100% safe code.

Struct Derive features:
- from_bytes and into_bytes functions are created. these are implemented for the Bitfields trait in bondrewd.
- Reverse Byte Order with no runtime cost. 
  - #[bondrewd(reverse)]
- Bit 0 positioning. Msb0 or Lsb0. Small compile time cost. 
  - #[bondrewd(read_from = "ZERO_BIT_LOCATION")]. ZERO_BIT_LOCATION can be mbs0 or lsb0.
- Peek functions. Unpack on a per fields basis. useful if you only need a couple fields but would rather not unpack the entire structure. 
  - peek_{field_name}() and peek_slice_{field_name}().
- Bit Size Enforcement. Specify how many used bits you expect the output to have. 
  - #[bondrewd(enforce_bits = {AMOUNT_OF_BITS})]
  - #[bondrewd(enforce_bytes = {AMOUNT_OF_BYTES})]
  - #[bondrewd(enforce_full_bytes)]

Field Derive features: 
- Natural typing of primitives. No Custom Type Wrapping. 
  - #[bondrewd(bit_length = {TOTAL_BITS_TO_USE})]
  - #[bondrewd(byte_length = {TOTAL_BYTES_TO_USE})]
  - #[bondrewd(bits = "FIRST_BIT_INDEX..LAST_BIT_INDEX_PLUS_ONE")] - not tested.
- Enum Fields that can catch Invalid variants. 
  - #[bondrewd(enum_primitive = "u8")]. u8 is the only supported type, but i am willing to support more if needed.
- Inner Structures. 
  - #[bondrewd(struct_size = {TOTAL_BYTES})]
- Per field Endianness control. 
  - #[bondrewd(endianness = "{ENDIANNESS}")], ENDIANNESS can be: le, be, msb, lsb big, little. use your favorite.
- Arrays.
  - Element Arrays. Define the bit-length of each element in the array. 
    - #[bondrewd(element_bit_length = {TOTAL_BITS_PER_ELEMENT})]
    - #[bondrewd(element_byte_length = {TOTAL_BYTES_PER_ELEMENT})]
  - Block Array. Define a overall bit length. example [u8;4] defined with a bit-length of 28 would remove the 4 Most Significant bits. 
    - #[bondrewd(block_bit_length = {TOTAL_AMOUNT_OF_BITS})]
    - #[bondrewd(block_byte_length = {TOTAL_AMOUNT_OF_BYTES})]
- Auto reserve fields. If the structures total bit amount is not a multiple of 8, the unused bits at the end will be ignored.
- Ignore reserve fields. peek_ and peek_slice_ functions are still generated but into_bytes and from_bytes will just use zeros
  - #[bondrewd(reserve)]

Enum Derive features: 
- Derive from_primitive and into_primitive.
- specify a Invalid variant for catching values that don't make sense. otherwise the last value will be used as a catch all.
  - #[bondrewd_enum(invalid)].
- Invalid with primitive. like the Invalid catch all above but it stores the value as a variant field.

Long answer.

I work in a Small satellite company and when sending data between ground and orbit we try to reduce byte size where ever possible because more bytes means more susceptibility to errors/interference. To fix this Protocols for communicating have been made, for example Ccsds Packets. Due to the limited selection of space grade processors our current processor is a single core 1gz processor meaning fast packing/unpacking is very important.
Reasons i needed to reinvent the wheel rather than use PackedStruct 0.6 or ModularBitfields 0.11: 
  - Natural typing. use primitives directly. (neither ModularBitfields nor PackedStruct do this for fields that have bit lengths that are not powers of 2).
  - Enum Field support that can catch invalid numbers without panics. (PackedStruct offers this, but you are required to use a built in EnumType Wrapper which i didn't like, ModularBitfields panics).
  - Reverse Byte Order with no runtime cost. this reason i needed this is very stupid so don't ask.
  - Bit 0 positioning. Msb0 or Lsb0 (PackedStruct offers this)
  - Peek functions. unpack on a per fields basis.
  - Bit Size Enforcement. (Both offer this but only in the Must be a multiple of 8 way).
