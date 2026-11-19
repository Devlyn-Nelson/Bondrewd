# Bondrewd (claude-rewrite)

Bit-level field packing via derive macros. This branch is a ground-up rewrite of
`bondrewd 0.3.0-beta` / `bondrewd-derive 0.5.0-beta`, released as `bondrewd 0.4.0-alpha.0`
and `bondrewd-derive 0.6.0-alpha.0`. It must stay a drop-in replacement for the old crates
except where noted under "Decisions". User-facing docs (attributes, generated API, layout
rules, wire changes) live in `README.md`; keep it in sync with any behaviour change.

## Layout

- `src/lib.rs`: runtime crate (`no_std`): traits `Bitfields<const SIZE>` and `BitfieldsSlice`,
  `BitfieldLengthError`. Features: `derive` (default), `std` -> `alloc` (default), `full` (compat alias).
- `derive/src/attrs.rs`: parses `#[bondrewd(...)]` by hand with `syn::parse_nested_meta` (no darling).
- `derive/src/layout.rs`: field model (`Prim`, `Kind`, `Field`), `classify`, `place` (positions and
  overlap checks), and the bit mapping (`Phys::locate`, `ValueMap`, `chunks`).
- `derive/src/codegen.rs`: `read_expr` / `write_stmts` / `nested_asserts`.
- `derive/src/lib.rs`: entry points, `Model::build`, `gen_bitfields`, `gen_slice`, enum handling.
- `derive/src/dump.rs`: `#[bondrewd(dump)]` writes generated code to
  `<target>/bondrewd_dump/<crate>/<Type>.<derive>.rs` (rustfmt if available, built-in formatter otherwise).
- `compat/`: differential test against bondrewd 0.3 (see below).

## How the codegen works (one code path for everything)

Every field becomes leaves. A leaf converts its value to small "value bytes" (`to_le_bytes`
for ints, or the nested type's logical bytes) and maps each logical bit of the field to a value
bit (`ValueMap`). The type's `Phys` maps each logical bit to a buffer bit. `chunks` groups bits
sharing (buffer byte, value byte, shift) into one masked shift. Generated code is only
`buf[i] = (buf[i] & keep) | ((v[j] >> s) & mask)` style statements; all positions are
computed in the macro. Do not add special-case paths per endianness or kind: the old crate's
bugs came from divergent paths.

- Logical layout: fields in declaration order from bit 0 (enum fields start after the id).
- `be`/`le` (`Phys.aligned = false`): logical bit i is bit `7 - i%8` of byte `i/8`.
- `ale`: buffer is one LE integer, logical bit i is integer bit `BYTE_SIZE*8 - BIT_SIZE + i`.
- `bit_traversal = "back"` mirrors each leaf inside `BIT_SIZE` (`start' = BIT_SIZE - start - len`)
  before the mapping, so padding stays put. `reverse` flips byte index last.
- Nested types are embedded through hidden `Bitfields::__into_logical` / `__from_logical`
  (canonical "logical bit i at bit 7-i%8 of byte i/8" form), so a child's internal layout is
  independent of the parent's endianness. Parents need `bit_length` == child's `BIT_SIZE`
  (enforced with a `const` assert).
- Block arrays are expanded per element (`block_elements` in codegen.rs): elements in array
  order, each in the field's endianness, truncation removes the top bits of element 0 first.

## Decisions made with the maintainer (don't undo without asking)

- Slice fns are named `read_slice_<field>` / `write_slice_<field>` (old bondrewd naming).
  Checked views use `read_<field>` / `write_<field>`. Enum accessors: `read_variant_id`,
  `read_<variant_snake>_<field>`; tuple fields are `field_<n>`.
- `BitfieldsSlice` is standalone (no const generic, no `Bitfields` supertrait, no
  `into_bytes`) and provides `from_slice`, `take_from_vec` (alloc), `check_slice`, `check_slice_mut`.
- `ale` enums: a variant shorter than the largest one is laid out right after the id. This is
  an intentional wire change from 0.3 (0.3 pushed it to the top of the payload), pinned by
  `ale_enum_short_variants_follow_the_id` in `tests/compat_layouts.rs`.
- `bit_traversal = "back"` and `le` block arrays were changed to match 0.3 byte for byte.
- `tests/ccsds.rs` is the maintainer's critical use case: only update API usage there, never
  values or expectations.
- README carries an AI disclosure (rewritten by Claude; human verified and tested externally
  without AI). Keep it.

## Compatibility with bondrewd 0.3 (`compat/`)

`compat/run.sh [--offline]` builds `compat/old` (bondrewd `=0.3.0-beta.2` from the
`pumpkin-patch` registry) and `compat/new` (this checkout) from the same `compat/cases.rs` and
`compat/probe.rs`, decodes random buffers, re-encodes them and diffs. Expected result:
everything `identical` except:

- `LeRev`, `OutBe`, `OutLe`: 0.3 fails its own round-trip (0.3 bugs: `le` + `reverse`, and
  nested types inside element arrays).
- `EnAle`, `EnUseAle`: the intentional `ale` enum change above.
- Not in the cases because 0.3 panics: nested `ale` types in element arrays (runtime index
  out of bounds) and `ale` block arrays (derive panic).

Any other `DIFFERS` line, or `NEW FAILS ROUND-TRIP`, is a regression. Add a case to
`cases.rs` when touching layout logic. The maintainer recalls older releases (0.1.x / derive
0.3.x) reading element arrays backwards; 0.3 does not, and neither does this crate.

## Not supported yet

- Generic types (rejected with an error in `Model::build`).
- Old API: `BitfieldsDyn` (`from_vec` is now `take_from_vec`), `BitfieldsHex*`, hex error types,
  per-variant `<VARIANT>_BIT_SIZE` consts, generated `check_slice_<name>` helpers.
- Old attributes: `overlapping_bits = N` (only the bare flag), `endianness_fn`,
  `fill`, `fill_bits_to`, `fill_bytes_to`, `payload_bit_length`/`payload_byte_length`,
  per-variant endianness/reverse/fill, `endianness = "ne"`, unquoted `bits` ranges.

## Release notes

- Not yet configured: `registry = "pumpkin-patch"` on the `bondrewd-derive` dependency and
  `publish = [...]` in both manifests. As written, `cargo publish` targets crates.io.

## Checks

```bash
cargo test --offline
cargo clippy --offline --workspace --all-targets
cargo build --offline --no-default-features --features derive   # no_std
compat/run.sh --offline
```
