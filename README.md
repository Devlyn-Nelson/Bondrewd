# Bondrewd

A proc-macro crate to safely and efficiently implement `from_bytes/into_bytes` for structures composed of primitive types and arrays.

The major features of the crate are:

- Decode/encode big or little endian data structures
- Associated functions to `read/write` a single field instead of decoding/encoding a whole structure, saving many instructions
- Ability to decode/encode C-like enums from integer types
- Pure-rust typing with attributes to assign endianness/bit-length/bit-positioning/...
- by default no failable functions are generated. Two Error types exist but are within crate features `"dyn_fns"` and `"hex_fns"`.
- Compress structures into small amounts of bits, or use to expand large data structures across many bytes
- All generated code is `no_std` capable and 100% safe code.

## Quick start

Add the following to the `dependencies` in `Cargo.toml`:

```toml
[dependencies]
bondrewd = { version = "0.2", features = ["derive"] }
```

`bondrewd` is easily implemented on structures to implement bit-field like structures like:
```rust
use bondrewd::Bitfields;

#[derive(Bitfields, Clone, Eq, PartialEq, Debug)]
#[bondrewd(id_bit_length = 3, default_endianness = "be")]
enum EyeColor {
    Blue,
    Green,
    Brown,
    Other {
        #[bondrewd(capture_id)]
        test: u8,
    },
}

#[derive(Bitfields, Clone, Eq, PartialEq, Debug)]
#[bondrewd(default_endianness = "be")]
struct PersonParts {
    head: bool,
    #[bondrewd(bit_length = 2)]
    shoulders: u8,
    #[bondrewd(bit_length = 2)]
    knees: u8,
    #[bondrewd(bit_length = 4)]
    toes: u8,
}

#[derive(Bitfields, Clone, Eq, PartialEq, Debug)]
#[bondrewd(default_endianness = "be", reverse)]
struct Person {
    // Name is english only?
    #[bondrewd(element_byte_length = 2)]
    name: [char; 32],
    // Age of Person Nobody lives past 127
    #[bondrewd(bit_length = 7)]
    age: u8,
    // Eyes have color
    #[bondrewd(bit_length = 3)]
    eye_color: EyeColor,
    // Standard components
    #[bondrewd(bit_length = 9)]
    parts: PersonParts,
    // How many times they have blinked each of their eyes
    #[bondrewd(bit_length = 60)]
    blinks: u64,
}

fn main() {
    let person = Person {
        name: ['a'; 32],
        age: 27,
        eye_color: EyeColor::Blue,
        parts: PersonParts {
            head: true,
            shoulders: 2,
            knees: 2,
            toes: 10,
        },
        blinks: 10_000_000_000,
    };

    // Get bitfield form of `person`.
    let bytes = person.clone().into_bytes();

    // Test reconstructing the output works and verify it is correct.
    assert_eq!(Person::from_bytes(bytes), person);

    // Test changing the output works as expected. This will be our target.
    let target_changes = Person {
        name: ['b'; 32],
        age: 72,
        eye_color: EyeColor::Green,
        parts: PersonParts {
            head: false,
            shoulders: 1,
            knees: 3,
            toes: 7,
        },
        blinks: 5,
    };

    // Get `person` as bytes again, which has different values than out target. 
    let mut bytes = person.into_bytes();

    // Change output.
    Person::write_name(&mut bytes, target_changes.name);
    Person::write_age(&mut bytes, target_changes.age);
    Person::write_eye_color(&mut bytes, target_changes.eye_color.clone());
    Person::write_parts(&mut bytes, target_changes.parts.clone());
    Person::write_blinks(&mut bytes, target_changes.blinks);
    
    // Verify.
    assert_eq!(Person::read_name(&bytes), target_changes.name);
    assert_eq!(Person::read_age(&bytes), target_changes.age);
    assert_eq!(Person::read_eye_color(&bytes), target_changes.eye_color.clone());
    assert_eq!(Person::read_parts(&bytes), target_changes.parts.clone());
    assert_eq!(Person::read_blinks(&bytes), target_changes.blinks);

    // Verify Harder.
    let new_person = Person::from_bytes(bytes);
    assert_eq!(new_person, target_changes);
}

```

# Usage Derive Details

`bondrewd` implements several derive attributes for:

- Structures
- Fields
- Enums

## Structure and Enum derive features:

- `from_bytes` and `into_bytes` functions are created via [Bitfields](https://docs.rs/bondrewd/0.1.3/bondrewd/trait.Bitfields.html) trait in bondrewd.
- Reverse Byte Order with no runtime cost.
  - `#[bondrewd(reverse)]`
- Traverse the bits in reverse order.
  - `#[bondrewd(bit_traversal = "front")]` means that the first bit is the left most bit in the first byte.
  - `#[bondrewd(bit_traversal = "back")]` means that the first bit is the left most bit in the first byte.
- Read functions to unpack on a per fields basis. Useful if you only need a couple fields but would rather not unpack the entire structure.
  - `read_{field_name}()` and `read_slice_{field_name}()`.
- Bit Size Enforcement. Specify how many used bits/bytes you expect the output to have.
  - `#[bondrewd(enforce_bits = {AMOUNT_OF_BITS})]`
  - `#[bondrewd(enforce_bytes = {AMOUNT_OF_BYTES})]`
  - `#[bondrewd(enforce_full_bytes)]`

## Field derive features:

- Natural typing of primitives. No Custom Type Wrapping.
  - `#[bondrewd(bit_length = {TOTAL_BITS_TO_USE})]`
  - `#[bondrewd(byte_length = {TOTAL_BYTES_TO_USE})]`
  - `#[bondrewd(bits = "FIRST_BIT_INDEX..LAST_BIT_INDEX_PLUS_ONE")]` (Not well tested).
- Nested Structures.
  - Structures that also implement `Bitfields` can be used as a field by simply providing the `bit_length` or `byte_length`. `#[bondrewd(bit_length = {TOTAL_BYTES})]`
- Per field Endianness control.
  - `#[bondrewd(endianness = "{ENDIANNESS}")]`, ENDIANNESS can be `big` or `little`. use your favorite.
- Arrays.
  - Element Arrays. Define the bit-length of each element in the array.
    - `#[bondrewd(element_bit_length = {TOTAL_BITS_PER_ELEMENT})]`
    - `#[bondrewd(element_byte_length = {TOTAL_BYTES_PER_ELEMENT})]`
  - Block Array. Define a overall bit length. example `[u8;4]` defined with a bit-length of 28 would remove the 4 Most Significant bits.
    - `#[bondrewd(block_bit_length = {TOTAL_AMOUNT_OF_BITS})]`
    - `#[bondrewd(block_byte_length = {TOTAL_AMOUNT_OF_BYTES})]`
- Auto reserve fields. If the structures total bit amount is not a multiple of 8, the unused bits at the end will be ignored.
- Ignore reserve fields. read_ and read_slice_ functions are still generated but into_bytes and from_bytes will just use zeros
  - `#[bondrewd(reserve)]`

# Why Bondrewd

Historically, the main reason for the crate was to share complex data structures for Space communication protocols (e.g. CCSDS/AOS/TC/TM...) between different software services and dependencies, without performance penalties for decoding/encoding whole `struct`s from bytes.
Originally, we wrote code for these formats using both [modular_bitfield](https://docs.rs/modular-bitfield/latest/modular_bitfield/) and [packed_struct](https://docs.rs/packed_struct/latest/packed_struct/) but eventually were unsatisfied with the results.

For our software, we were severely constrained by compute while transferring large amounts of data, spending lots of time decoding/encoding data structures.
We found that for certain parts of the communications services that we didn't need to decode the whole structure from bytes in order to process the communications packets.
In addition, we found many times we'd just set a single field in the structure and pass the packet to the next stage.
So, to remedy these issues, we needed a bitfields implementation that allowed us the performance and safety, with the ability to only decode small parts of data to determine which node/process to pass data on to.

Both `modular_bitfields/packed_struct` are great/stable libraries, and an existing, performant and correct implementation of either of these libraries would be sufficient for almost all use cases.
However, this crate looks to fill the following from these two crates:

- Read functions via associated functions. Unpack on a per field basis.
- Natural typing, using primitives directly with proc-macro attributes.
  - Neither ModularBitfields or PackedStruct do this for fields that have bit lengths that are not powers of 2.
- Enum Field support that can catch invalid numbers without panics. 
  - N.B. PackedStruct offers this feature, but you are required to use a built-in EnumType Wrapper. ModularBitfields exhibits panic behavior.
- Reverse Byte Order with no runtime cost.
- Bit 0 positioning. `front` or `back`. This doesn't truly change bit order, but simply defined which direction in which the indexs of bits should be read. (please read aligned example)
  - PackedStruct offers this.
- Bit Size Enforcement with non-power of 2 bit lengths.

