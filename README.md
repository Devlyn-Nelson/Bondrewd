# Bondrewd
I use this for work, so if you use this and find a problem i would be more than happy to fix it.
I currently do not have time to write good docs, so in the mean time looking at the files within the bondrews-derive/tests/ folder would be your best bet.
# Purpose
Short answer.
Derive Bit level fields packing/unpacking functions for a rust Structure with the ability to peek at a field without unpacking any other fields. 
outer struct features:
- Reverse Byte Order with no runtime cost. 
    #[bitfields(flip)]
- Bit 0 positioning. Msb0 or Lsb0. Small compile time cost. 
    #[bitfields(read_from = "msb0" or lsb0)].
- Peek functions. Unpack on a per fields basis. useful if you only need a couple fields but would rather not unpack the entire struct. 
    peek_{field_name}() and peek_slice_{field_name}().
- Bit Size Enforcement. Specify how many used bits you expect the output to have. 
    #[bitfields(enforce_bits = {AMOUNT_OF_BITS})] or #[bitfields(enforce_full_bytes)].

field feautres: 
- Natural typing of primitives. No Custom Type Wrapping. 
    #[bitfield(bit_length = {TOTAL_BITS_TO_USE})]
- Enum Fields that can catch Invalid variants. 
    #[bitfield(enum_primitive = "u8")]. u8 is the only supported type, but i am willing to support more if needed.
- Inner Structures. 
    #[bitfield(struct_size = {TOTAL_BYTES})]
- Per field Endiannes control. 
    #[bitfield(endianness = "{ENDIANNESS}")], ENDIANNESS can be: le, be, msb, lsb big, little. use your favorite.
- Arrays.
  - Element Arrays. Define the bit-length of each element in the array. 
      #[bitfield(element_bit_length = {TOTAL_BIT_PER_ELEMENT})]
  - Block Array. Define a overall bit length. example [u8;4] defined with a bit-length of 28 would remove the 4 Most Significant bits. 
      #[bitfield(array_bit_length = {TOTAL_AMOUNT_OF_BITS})]
- Auto reserve fields. If the structures total bit amount is not a multiple of 8, the unused bits at the end will be ignored.

Long answer.
I work in a Small satellite company and when sending data between ground and orbit we try to reduce byte size where ever possible because more bytes means more susceptibility to errors/interference. To fix this Protocols for communicating have been made, for example Ccsds Packets. Due to the limited selection of space grade processors our current processor is a single core 1gz proccessor meaning fast packing/unpacking is very important.
Reasons i needed to reinvent the wheel rather than use PackedStruct 0.6 or ModularBitfields 0.11: 
  - Natural typing. use primitives directly. (neither ModularBitfields nor PackedStruct do this for fields that have bit lengths that are not powers of 2).
  - Enum Field support that can catch invalid numbers without panics. (PackedStruct offers this, but you are required to use a built in EnumType Wrapper which i didn't like).
  - Reverse Byte Order with no runtime cost. this reason i needed this is very stupid so don't ask.
  - Bit 0 positioning. Msb0 or Lsb0 (PackedStruct offers this)
  - Peek functions. unpack on a per fields basis.
  - Bit Size Enforcement. (Both offer this but only in the Must be a multiple of 8 way).
