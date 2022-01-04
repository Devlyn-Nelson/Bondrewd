# Bondrewd

A proc-macro crate to safely and efficiently implement `from_bytes/to_bytes` for structures composed of primitive types and arrays.

The major features of the crate are:

* Decode/encode big or little endian data structures
* Associated functions to `read/write` a single field instead of decoding/encoding a whole structure, saving many instructions
* Ability to decode/encode C-like enums from integer types
* Pure-rust typing with attributes to assign endianness/bit-length/bit-positioning/...
* All generated code is safe, with only one failable type used for encoding slices
* Compress structures into small amounts of bits, or use to expand large data structures across many bytes
* All generated code is `no_std` capable and 100% safe code.

## Quickstart

Add the following to the `dependencies` in `Cargo.toml`:

```toml
[dependencies]
bondrewd = { version = "^0.1", features = ["derive"] }
```

`bondrewd` is easily implemented on structures to implement bit-field like structures like:
```rust
use bondrewd::{BitfieldEnum, Bitfields};

///! Implement a basic CCSDS 133.0-B-2 Primary Header using rust Enums to specify fields

/// Packet Sequence Flags as per 4.1.3.4.2.2
#[derive(BitfieldEnum, Clone, PartialEq, Eq, Debug)]
pub enum CcsdsPacketSequenceFlags {
  Continuation,
  Start,
  End,
  Unsegmented,
  Invalid(u8),
}

/// CCSDS Packet version as per 4.1.3.2
#[derive(BitfieldEnum, Clone, PartialEq, Eq, Debug)]
#[bondrewd_enum(u8)]
pub enum CcsdsPacketVersion {
  One,
  Two,
  Invalid,
}

/// Primary header object as per 4.1.3
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be", enforce_bytes = 6)]
pub struct CcsdsPacketHeader {
  #[bondrewd(enum_primitive = "u8", bit_length = 3)]
  pub(crate) packet_version_number: CcsdsPacketVersion,
  pub(crate) packet_type: bool,
  pub(crate) sec_hdr_flag: bool,
  #[bondrewd(bit_length = 11)]
  pub(crate) app_process_id: u16,
  #[bondrewd(enum_primitive = "u8", bit_length = 2)]
  pub(crate) sequence_flags: CcsdsPacketSequenceFlags,
  #[bondrewd(bit_length = 14)]
  pub(crate) packet_seq_count: u16,
  pub(crate) packet_data_length: u16,
}

// Now you're on your way to space :)
// Lets see what this can generate
fn main() {
  let packet = CcsdsPacketHeader {
    packet_version_number: CcsdsPacketVersion::Invalid,
    packet_type: true,
    sec_hdr_flag: true,
    app_process_id: 55255 & 0b0000011111111111,
    sequence_flags: CcsdsPacketSequenceFlags::Unsegmented,
    packet_seq_count: 65535 & 0b0011111111111111,
    packet_data_length: 65535,
  };
  
  // Turn into some bytes (clone used to assert_eq later)
  let bytes = packet.clone().into_bytes();
  
  // Play with some of the fields
  match CcsdsPacketHeader::read_sequence_flags(&bytes) {
    CcsdsPacketSequenceFlags::Unsegmented => println!("Unsegmented!"),
    CcsdsPacketSequenceFlags::End => println!("End!"),
    _ => println!("Something else")
  }
  
  // Set the secondary header flag
  CcsdsPacketHeader::write_sec_hdr_flag(&mut bytes, false);
  
  // Get back from bytes, check them
  let new_packet = CcsdsPacketHeader::from_bytes(bytes);
  assert_eq!(new_packet.sec_hdr_flag, false);
  assert_eq!(new_packet.app_process_id, packet.app_process_id);
}
```

# Usage Derive Details

`bondrewd` implements several derive attributes for:

* Structures
* Fields
* Enums

## `struct` Derive features:

* `from_bytes` and `into_bytes` functions are created via [Bitfields](https://docs.rs/bondrewd/0.1.3/bondrewd/trait.Bitfields.html) trait in bondrewd.
* Reverse Byte Order with no runtime cost.
  * `#[bondrewd(reverse)]`
* Bit 0 positioning with `Msb0` or `Lsb0` with only small compile time cost.
  * `#[bondrewd(read_from = "ZERO_BIT_LOCATION")]`. `ZERO_BIT_LOCATION` can be `mbs0` or `lsb0`.
* Peek functions to unpack on a per fields basis. Useful if you only need a couple fields but would rather not unpack the entire structure.
  * `read_{field_name}()` and `read_slice_{field_name}()`.
* Bit Size Enforcement. Specify how many used bits/bytes you expect the output to have.
  * `#[bondrewd(enforce_bits = {AMOUNT_OF_BITS})]`
  * `#[bondrewd(enforce_bytes = {AMOUNT_OF_BYTES})]`
  * `#[bondrewd(enforce_full_bytes)]`

## `field` Derive features:

* Natural typing of primitives. No Custom Type Wrapping.
  * `#[bondrewd(bit_length = {TOTAL_BITS_TO_USE})]`
  * `#[bondrewd(byte_length = {TOTAL_BYTES_TO_USE})]`
  * `#[bondrewd(bits = "FIRST_BIT_INDEX..LAST_BIT_INDEX_PLUS_ONE")]` (To be tested).
* Enum Fields that can catch Invalid variants.
  * `#[bondrewd(enum_primitive = "u8")]`. Currently, u8 is the only supported type, with support for more in the future.
* Inner Structures.
  * `#[bondrewd(struct_size = {TOTAL_BYTES})]`
* Per field Endianness control.
  * `#[bondrewd(endianness = "{ENDIANNESS}")]`, ENDIANNESS can be: `le`, `be`, `msb`, `lsb`, `big`, `little`. use your favorite.
* Arrays.
  * Element Arrays. Define the bit-length of each element in the array.
    * `#[bondrewd(element_bit_length = {TOTAL_BITS_PER_ELEMENT})]`
    * `#[bondrewd(element_byte_length = {TOTAL_BYTES_PER_ELEMENT})]`
  * Block Array. Define a overall bit length. example `[u8;4]` defined with a bit-length of 28 would remove the 4 Most Significant bits.
    * `#[bondrewd(block_bit_length = {TOTAL_AMOUNT_OF_BITS})]`
    * `#[bondrewd(block_byte_length = {TOTAL_AMOUNT_OF_BYTES})]`
* Auto reserve fields. If the structures total bit amount is not a multiple of 8, the unused bits at the end will be ignored.
* Ignore reserve fields. read_ and read_slice_ functions are still generated but into_bytes and from_bytes will just use zeros
  * `#[bondrewd(reserve)]`

# `enum` Derive features:

* Derive from_primitive and into_primitive.
* Specify an `Invalid` variant for catching values that don't make sense, otherwise the last value will be used as a catch-all.
  * `#[bondrewd_enum(invalid)]`.
* Specify custom `u8` literal for discriminants on enum variants 
* Invalid with primitive. like the Invalid catch all above but it stores the value as a variant field.

# Why Bondrewd

Historically, the main reason for the crate was to share complex data structures for Space communication protocols (e.g. CCSDS/AOS/TC/TM...) between different software services and dependencies, without performance penalties for decoding/encoding whole `struct`s from bytes.
Originally, we wrote code for these formats using crates like [modular_bitfield](https://docs.rs/modular-bitfield/latest/modular_bitfield/) and [packed_struct](https://docs.rs/packed_struct/latest/packed_struct/) and shared a common crate across all software services.
For our software, we were severely constrained by compute while transferring large amounts of data, spending lots of time decoding/encoding data structures.
We found that for certain parts of the communications services that we didn't need to decode the whole structure from bytes in order to process the communications packets.
In addition, we found many times we'd just set a single field in the structure and pass the packet to the next stage.
So, to remedy these issues, we needed a bitfields implementation that allowed us the performance and safety, with the ability to only decode small parts of data to determine which node/process to pass data on to.

Both `modular_bitfields/packed_struct` are great/stable libraries, and an existing, performant and correct implementation of either of these libraries would be sufficient for almost all use cases.
However, this crate looks to fill the following from these two crates:

* Peek functions via associated functions. Unpack on a per field basis.
* Natural typing, using primitives directly with proc-macro attributes.
  * Neither ModularBitfields or PackedStruct do this for fields that have bit lengths that are not powers of 2.
* Enum Field support that can catch invalid numbers without panics. 
  * N.B. PackedStruct offers this feature, but you are required to use a built-in EnumType Wrapper. ModularBitfields exhibits panic behavior.
* Reverse Byte Order with no runtime cost.
* Bit 0 positioning. Msb0 or Lsb0
  * PackedStruct offers this.
* Bit Size Enforcement with non-power of 2 bit lengths.

