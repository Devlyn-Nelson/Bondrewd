# bondrewd

Bit-level field packing for structs and enums with derive macros. `no_std` (the `alloc` feature adds
`take_from_vec`).

> **AI disclosure:** this version of `bondrewd` (0.4) and `bondrewd-derive` (0.6) was rewritten
> by an AI model (Claude, by Anthropic), including the code, tests and this README. A human
> maintainer verified the results and is preforming additional external testing without AI assistance. When everything is fully tested a non alpha version will be published.

```rust
use bondrewd::{Bitfields, BitfieldsSlice};

#[derive(Bitfields, BitfieldsSlice, Clone, PartialEq, Debug)]
#[bondrewd(endianness = "be")]
struct Header {
    #[bondrewd(bit_length = 3)]
    version: u8,
    #[bondrewd(bit_length = 13)]
    id: u16,
    length: u16, // primitives default to their full size
}

let bytes: [u8; 4] = Header { version: 1, id: 7, length: 9 }.into_bytes();
assert_eq!(Header::read_id(&bytes), 7);
assert_eq!(Header::read_slice_length(&bytes[..4])?, 9);
let view = Header::check_slice(&bytes)?;  // HeaderChecked, infallible reads
assert_eq!(view.read_version(), 1);
```

## Generated API

| Derive | Generates |
|---|---|
| `Bitfields` | `impl Bitfields<N>` (`BIT_SIZE`, `BYTE_SIZE`, `into_bytes`, `from_bytes`), inherent `BIT_SIZE`/`BYTE_SIZE`, `read_<field>(&[u8; N])`, `write_<field>(&mut [u8; N], v)` |
| `BitfieldsSlice` | `impl BitfieldsSlice` (`from_slice`, `take_from_vec`, `check_slice`, `check_slice_mut`), `read_slice_<field>` / `write_slice_<field>` (only need the bytes that field touches), `<Name>Checked` / `<Name>CheckedMut` with `read_<field>` / `write_<field>` and `from_unchecked_slice` (skips the length check; a short slice panics on access), `From<Checked>` for the type |

Enums get `read_variant_id` plus `read_<variant_snake>_<field>`. Tuple fields are named `field_<index>`.

## Attributes

Type level: `endianness = "be" | "le" | "ale"` (default `be`), `reverse`, `bit_traversal = "front" | "back"`,
`fill_bits`, `fill_bits = N`, `fill_bytes = N`, `enforce_bits = N`, `enforce_bytes = N`, `enforce_full_bytes`,
`crate_path = "..."`, `dump`. Enums also take `id_bit_length` / `id_byte_length`.

Variant level: `id = N` (Rust discriminants also work, and ids continue from the previous one), `invalid`.

Field level: `bit_length` / `byte_length`, `element_bit_length` / `element_byte_length`,
`block_bit_length` / `block_byte_length`, `endianness`, `bits = "a..b"`, `reserve`, `read_only`, `redundant`, `capture_id`.

## Inspecting generated code

Add `dump` to the type attribute (`#[bondrewd(endianness = "be", dump)]`) and each derive
writes its output to `<target dir>/bondrewd_dump/<crate name>/<Type>.<derive>.rs`, e.g.
`target/bondrewd_dump/my_crate/Header.bitfields.rs` and `Header.bitfields_slice.rs`.
The target dir is `CARGO_TARGET_DIR` if set, otherwise `target/` in the workspace root.
The file is formatted with `rustfmt` when it's on `PATH` (or `RUSTFMT` is set), and with a
built-in formatter otherwise. Masks are printed in binary so you can see which bits each
operation touches. If the file can't be written, compilation fails with the reason.

## Layout rules

* **be** – logical bit 0 is the left-most bit of byte 0; values are MSB first.
* **le** – same bit order, but the value's little-endian bytes are written in order (each MSB first);
  a partial final byte keeps its low `bits % 8` bits.
* **ale** – the buffer is one little-endian integer; logical bit 0 is on the right
  (`BYTE_SIZE*8 - BIT_SIZE`), values are LSB first, padding sits at the least significant end.
* `reverse` reverses the output byte order. `bit_traversal = "back"` reverses the field
  order inside the `BIT_SIZE` bits (padding stays where it would be without it).
* **Enums** – variant id first, using the fewest bits that fit the largest id (min 1) unless set.
  Every variant is laid out as a struct after the id. Unknown ids decode to the `invalid` variant,
  or to the last variant if none is marked. A `capture_id` field stores the id itself.
* **Nested types** need `bit_length`/`byte_length` equal to their `BIT_SIZE` (checked at compile time).
  Their logical bits are copied in order, so a nested type's internal layout is the same whatever
  endianness the parent uses.
* **Element arrays** – each element is its own bitfield (`element_bit_length`); default for arrays.
* **Block arrays** – the array is one block of bits (`block_bit_length`): elements in array
  order, each in the field's endianness. When the block is shorter than the array, the top
  bits of element 0 are removed first. For `be` this is the same as one big number with
  element 0 most significant.
* `reserve` fields are not written by `into_bytes` and decode as `Default::default()`.
  `read_only` fields are decoded by `from_bytes` but not written by `into_bytes` (their bits stay
  zero); their `write_<field>` functions are still generated. Can't be combined with `reserve`.
  `redundant` fields may overlap others and aren't written by `into_bytes`.

## Changes from bondrewd 0.3 / bondrewd-derive 0.5

* `BitfieldsSlice` is standalone (no `Bitfields` supertrait, no `into_bytes`), and also provides `from_slice` and `take_from_vec`.
* One code path: each field becomes per-byte mask/shift transfers computed in the macro, so every
  endianness, nesting depth and array kind shares the same code. The layouts in bondrewd's
  own BE/LE/ALE, reverse, back, nested-ALE, block and element array tests are reproduced in
  `tests/compat_layouts.rs`.
* Not supported yet: generic types, `hex_fns`/`dyn_fns`, `BitfieldsDyn`.

### Wire format changes

Compared byte for byte against bondrewd 0.3.0-beta.2 / bondrewd-derive 0.5.0-beta.6, all
layouts produce identical bytes except these. Data written by 0.3 in these layouts decodes
differently:

* **`ale` enums with a variant shorter than the largest one** (intentional): the short
  variant's fields now follow the id directly, like any struct. 0.3 placed them at the top of
  the payload, leaving a gap after the id. Pinned by `ale_enum_short_variants_follow_the_id`.
* **`le` structs with `reverse`** containing multi-byte fields: 0.3 did not round-trip its
  own output (it corrupted values), so there is no stable 0.3 format to match.
* **Nested types inside element arrays** (`be` and `le`): 0.3 did not round-trip its own
  output. Nested `ale` types in element arrays panicked at runtime in 0.3.
* **`ale` block arrays**: the 0.3 derive panicked, so there was no 0.3 format.
