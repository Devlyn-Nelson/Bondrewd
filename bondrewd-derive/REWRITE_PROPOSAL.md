# Proposal: Rewrite `bondrewd-derive` Without Losing Functionality

## Status and intent

This is an implementation proposal, not the rewrite itself. The goal is to replace the current proc-macro internals while preserving the public `bondrewd` API, generated method names, byte-level behavior, supported attributes, `no_std` behavior, and compile-time diagnostics wherever practical.

The rewrite should be treated as a compatibility project. The current implementation already has a useful separation between parsing/building, solving, and token generation, but those responsibilities are spread across `lib.rs`, `build`, `solved`, `derive`, and `masked`. The replacement should make the data model and bit-placement rules explicit before generating any Rust code.

## Parallel implementation requirement

The rewrite must be developed as a new derive crate rather than by replacing the current `bondrewd-derive` crate in place. Use a temporary working package name such as `bondrewd-derive-next` (with the corresponding crate name `bondrewd_derive_next`) until the implementation is ready for a release decision.

The existing `bondrewd-derive` crate is the legacy oracle and must remain buildable and unchanged while the new crate is developed. Add the new crate as a separate workspace member with its own source tree, tests, features, and dependencies. Do not make the old crate depend on the new crate during the comparison period.

The new crate's integration tests should depend on both derive crates and the existing runtime `bondrewd` crate. Each comparison fixture should define equivalent old and new types, for example:

```rust
use bondrewd_derive as legacy;
use bondrewd_derive_next as next;
use legacy::Bitfields as LegacyBitfields;
use next::Bitfields as NextBitfields;

#[derive(LegacyBitfields, Clone, Debug, PartialEq)]
#[bondrewd(endianness = "ale")]
struct LegacyPacket {
    // same fields and attributes as NextPacket
}

#[derive(NextBitfields, Clone, Debug, PartialEq)]
#[bondrewd(endianness = "ale")]
struct NextPacket {
    // same fields and attributes as LegacyPacket
}
```

The types must be separate because both derives generate implementations and associated functions with the same names. The comparison harness should construct equivalent values and compare:

- `BYTE_SIZE` and `BIT_SIZE`.
- `into_bytes` output byte-for-byte.
- Values reconstructed by `from_bytes`.
- Every generated `read_<field>` and `write_<field>` result.
- Dynamic, checked-slice, and hex behavior where the derive is enabled.
- Error results and consumed byte counts for fallible APIs.
- Generated behavior for nested structs, nested enums, arrays, overlap, reserve, fill, and reverse.

The comparison suite should use shared fixture descriptions or duplicated declarations generated from one test macro so the old and new types cannot drift accidentally. It should distinguish three outcomes:

1. **Equivalent**: old and new behavior match.
2. **Intentional correction**: the new behavior fixes a documented bug, with a regression test and migration note.
3. **Unexpected difference**: the rewrite is not ready for cutover.

Compile-fail tests should be run against both crates independently because diagnostics may differ while accepted/rejected input behavior remains equivalent. The legacy crate must remain available until the new crate has passed the complete comparison corpus and at least one release candidate cycle.

## Current contract to preserve

The primary source of truth should be the existing documentation in `bondrewd-derive/src/lib.rs`, the runtime traits in `bondrewd/src/lib.rs` and `bondrewd/src/hex.rs`, and the integration tests under `bondrewd-derive/tests`.

### Derive entry points

The replacement must continue to provide these derives:

- `Bitfields`
  - Implements `bondrewd::Bitfields<N>`.
  - Generates fixed-size `from_bytes` and `into_bytes`.
  - Generates fixed-buffer `read_<field>` and `write_<field>` associated functions.
- `BitfieldsDyn`
  - Implements `bondrewd::BitfieldsDyn<N>`.
  - Generates `from_slice` and, with the runtime `std` feature, `from_vec`.
  - `from_slice` copies the required prefix; `from_vec` consumes the required prefix only after the length check succeeds.
- `BitfieldsSlice`
  - Implements `bondrewd::BitfieldsSlice<N>`.
  - Generates checked immutable and mutable slice wrapper types.
  - Generates `read_slice_<field>` and `write_slice_<field>` functions that validate the bytes needed by the field.
  - For enums, generates checked wrapper variants for each enum variant.
- `BitfieldsHex`
  - Implements fixed-size uppercase/lowercase hex encoding and fixed-size hex decoding.
  - Preserves the `[u8; 2 * N]` convention and `BitfieldHexError` behavior.
- `BitfieldsHexDyn`
  - Preserves slice-based hex decoding and `std`-gated vector decoding with `BitfieldHexDynError`.

`Bitfields` remains the prerequisite derive conceptually: the other generated traits depend on fixed-size byte conversion and the same solved layout.

### Supported input types

Preserve support for:

- Structs with named fields, tuple structs, and unit structs.
- Enums with unit, named, and tuple variants.
- `bool`, signed and unsigned integer primitives through 128 bits, `f32`, `f64`, and `char`.
- Nested types that implement `Bitfields`, when their `bit_length` or `byte_length` is supplied.
- Element arrays and block arrays, including nested arrays with literal lengths.
- C-like enum discriminants and explicit Bondrewd IDs.

Continue rejecting `usize` and `isize` because their serialized width is target-dependent. Continue requiring full-width floating-point fields unless the supported contract is deliberately expanded in a separate version.

### Attributes and semantics

The parser must preserve the documented attributes and their validation rules:

- Object/layout: `endianness = "le" | "be" | "ale"`, `bit_traversal = "front" | "back"`, `reverse`, `dump`.
- Size checks: `enforce_bits`, `enforce_bytes`, `enforce_full_bytes`.
- Padding: `fill_bits`, `fill_bytes`, including automatic fill behavior.
- Enum layout: `id_bit_length`, `id_byte_length`, variant `id`/discriminants, `invalid`, and field `capture_id`.
- Field sizing: `bit_length`, `byte_length`, `bits` ranges.
- Arrays: `element_bit_length`, `element_byte_length`, `block_bit_length`, `block_byte_length`.
- Field behavior: `reserve`, `read_only`, `overlapping_bits`, and `redundant`.

The rewrite should reject conflicting attributes during parsing, with errors attached to the most useful source span. It should not silently reinterpret conflicting size declarations.

## Target architecture

The central design should be a typed intermediate representation (IR) with three independent concerns:

1. **What the user wrote**: syntax, attributes, source spans, field/variant names, and Rust types.
2. **Where each value lives**: a fully resolved logical bit layout and encoding plan.
3. **How to expose it**: fixed arrays, dynamic slices, checked slices, and hex APIs.

The generator should consume only the resolved IR. It should not recalculate bit positions while producing individual read and write functions.

### Compile-time-first performance requirement

Compile-time work is a primary design goal, not merely an implementation convenience. Bondrewd should perform as much layout and specialization work as possible during proc-macro expansion so generated runtime code performs only operations that depend on the actual input value or byte buffer.

The following must be resolved before tokens are generated:

- Field start/end positions and all physical byte/bit segments.
- Byte size, bit size, padding, fill, and enforcement results.
- Endianness aliases, traversal direction, byte reversal, and value-bit order.
- Masks, shifts, rotations, source-bit offsets, destination-bit offsets, and sign-extension masks.
- Nested child projection, child-buffer size, and every parent-to-child segment mapping.
- Array element offsets and block-array chunk boundaries.
- Enum ID widths, discriminant values, variant dispatch arms, and maximum payload size.
- The minimum byte ranges needed by direct slice field access.
- Whether a field is readable, writable, reserved, redundant, overlapping, or captured.

The generated fixed-layout runtime code should contain no calculations for those facts. It should use literal constants and statically emitted operations. In particular, it should not calculate `% 8`, `/ 8`, `ceil`, field offsets, masks, or endian decisions at runtime.

Runtime work is still required for value-dependent operations, including loading bytes, converting a supplied primitive value, extracting an input enum ID, checking a dynamic slice length, and applying read-modify-write masks. Those operations should use constants produced by the solver.

For ordinary fixed-size fields and arrays, macro expansion should unroll the segment operations. Runtime loops are acceptable only when required to prevent pathological generated-code size, and that exception must be measured and documented. A runtime loop must never be used merely because the layout was not solved at compile time.

The generated-code performance contract should be verified with token/assembly inspection or benchmarks for representative layouts. Correctness tests alone are not sufficient to prove that layout work was removed from runtime execution.

```text
syn::DeriveInput
        |
        v
  ParsedModel + diagnostics
        |
        v
  LayoutModel / resolved bit plan
        |
        +--> validation and compatibility snapshots
        |
        v
  EncodingPlan per field/variant
        |
        +--> fixed-array backend
        +--> dynamic/slice backend
        +--> checked-slice backend
        +--> hex trait backend
        |
        v
  quote! generated Rust
```

### 1. Parsed model

Create a small parser layer, preferably in a new `src/model` module, using `syn` and either explicit parsing or the existing `darling` dependency. 

**Diagnostic-First Design**: The parser must use a `DiagnosticBuffer` to accumulate `syn::Error` instances. It should not return early on the first attribute typo or logic conflict. This ensures that a user can see and fix all attribute errors in a single compilation pass.

It should produce types similar to:

```rust
struct ParsedObject {
    name: Ident,
    visibility: Visibility,
    attributes: ObjectAttrs,
    kind: ObjectKind,
    span: Span,
}

enum ObjectKind {
    Struct { fields: Vec<ParsedField>, tuple: bool },
    Enum { variants: Vec<ParsedVariant> },
}

struct ParsedField {
    access: FieldAccess,
    rust_type: RustType,
    attrs: FieldAttrs,
    span: Span,
}
```

Important rules:

- Normalize `bit_length`, `byte_length`, and `bits` into one `SizeSpec` while retaining the original span.
- Normalize array attributes into an `ArraySpec` with `Element` or `Block` mode.
- Normalize enum IDs before layout solving, but retain whether the ID came from a discriminant or a Bondrewd attribute for diagnostics.
- Represent tuple fields by stable generated identities while retaining the tuple index for construction and pattern matching.
- Preserve user visibility for generated checked types and generated functions according to the current behavior.

### 2. Type capability model

Resolve Rust types into a closed internal type model:

```rust
enum ValueType {
    Bool,
    Unsigned { bits: u16 },
    Signed { bits: u16 },
    Float { bits: u16 },
    Char,
    Nested { ident: Ident, byte_size: usize },
}

enum FieldType {
    Scalar(ValueType),
    Array { element: Box<FieldType>, dimensions: Vec<usize>, mode: ArrayMode },
}
```

This model must carry enough information to choose safe conversion operations:

- Integer fields use shifts/masks and sign extension for partial signed values.
- Floats use `to_bits`/`from_bits` and only accept full-width layouts.
- `char` uses its `u32` representation and reconstructs with the same valid-value behavior as today; the rewrite must decide and test how invalid code points are handled before implementation.
- Nested values call their `Bitfields<N>` implementation and then use the nested serialized bits as a value source/sink.
- Arrays expand into element plans for `Element` mode and into contiguous chunks for `Block` mode.

### Nested structures and nested enums: a first-class composition system

Nested values are not just another primitive type. They are one of the most important use cases for Bondrewd and are currently a known correctness problem, especially when a nested value is non-byte-aligned or uses `ale`. The rewrite must give nested composition its own plan and test phase rather than leaving it as a branch inside primitive quote generation.

A nested type should be treated as a child serializer with an explicit serialized contract:

```rust
struct NestedPlan {
    child_ident: Ident,
    child_bit_size: usize,
    child_byte_size: usize,
    field_bit_range: Range<usize>,
    projection: NestedProjection,
}

enum NestedProjection {
    Exact,
    ZeroPad { field_bits: usize },
    Truncate { field_bits: usize },
}
```

The composition rules should be:

1. The child type owns the layout of its own fields, enum ID, variant payload, padding, and child-level `endianness` policy.
2. The parent does not re-solve the child's fields and does not call an unfinished endianness merge operation.
3. The child is serialized into its canonical `[u8; child_byte_size]` representation using `Child::into_bytes()`.
4. The parent treats that serialized representation as a sequence of child bits and maps those bits into the parent's field segments.
5. Reading performs the inverse operation: it extracts the parent field segments into a zero-initialized child byte array and calls `Child::from_bytes()`.
6. Writing a nested field modifies only the parent's field segments and never changes unrelated parent fields.
7. A nested field's parent `bit_length` or `byte_length` determines the size of the parent projection; it must not silently change the child's internal layout.

This gives nested types a clean boundary: the child decides what its bytes mean, while the parent decides where those bytes are placed. It also prevents the current class of bugs where nested values are rotated or byte-swapped as if they were primitive integers.

Projection rules must be explicit and tested:

- `Exact`: the parent field width equals the child serialized bit width.
- `ZeroPad`: the parent field is larger than the child's meaningful bits; extra bits are initialized to zero and ignored when reconstructing the child.
- `Truncate`: the parent field is smaller than the child; this should only be accepted when explicitly requested by `bit_length` and must use one documented projection direction. If that direction cannot be made compatible with current behavior, reject the layout rather than silently dropping a different set of bits.
- `byte_length` must account for the child's serialized byte representation and any parent padding. `BIT_SIZE` and physical byte size remain separate quantities.

Nested decoding must never depend on the child output being one byte wide. The implementation must copy arbitrary bit segments into the local child buffer, including fields that begin at bit offsets 1 through 7, cross multiple bytes, or end in a partial byte.

#### Nested enums

A nested enum uses the same composition boundary as a nested struct. The parent must not inspect or reconstruct the child's variant ID itself. The child enum remains responsible for:

- ID width and placement within the child layout.
- Explicit and automatic variant IDs.
- Invalid-variant fallback for unknown IDs.
- `capture_id` behavior.
- Maximum payload sizing across variants.

The parent only embeds or extracts the child's serialized bits. A nested enum therefore works whether it is a direct field, an element-array member, or a block-array member. Its parent field width must be checked against the complete child representation so that the parent cannot accidentally cut through the child's ID unless an explicitly supported truncation rule says so.

#### Nested API behavior

The same nested plan must be used for every derive flavor:

- Fixed `Bitfields`: use a local fixed child byte array for encode/decode.
- `BitfieldsDyn`: validate the parent input length, then use the same local child array; the child itself remains fixed-size.
- `BitfieldsSlice`: field reads and writes use the parent field's physical byte coverage, then decode or encode through the local child array.
- Hex derives: operate around the parent byte representation and do not add nested-specific hex logic.

This removes the current need for separate nested quote families such as `get_read_nested_*` and `get_write_nested_*`. The generator should produce one segment-copy plan for nested values and reuse it in fixed, slice, and checked accessors.

### 3. Layout solver

The solver should be the only component that assigns logical bit ranges. It should operate on a logical bitstream first and map that stream to physical byte indices second.

For every field, produce a `FieldLayout` containing at least:

```rust
struct FieldLayout {
    logical_range: Range<usize>,
    physical_segments: Vec<BitSegment>,
    value_type: FieldType,
    encoding: Encoding,
    write_policy: WritePolicy,
    source_span: Span,
}

struct BitSegment {
    byte_index: usize,
    bit_offset: u8,
    width: u8,
    value_bit_offset: u16,
}
```

`physical_segments` is the key simplification. A field that crosses bytes, uses aligned little endian, is reversed, or is an array can be described as a list of segments. Both reading and writing then use the same segment list, eliminating separate hand-maintained formulas for each direction.

The segment list is also the compile-time specialization boundary. `byte_index`, `bit_offset`, `width`, `value_bit_offset`, masks, and shifts should all be concrete values in the generated token stream. The runtime should iterate over no layout metadata because the metadata has already been consumed by the proc macro.

The solver must make these concepts explicit:

- **Logical field order**: the order in which fields consume logical bits.
- **Bit traversal**: whether the first logical field is placed at the front or back of the logical bitstream.
- **Byte reversal**: whether physical byte indices are mirrored after the logical layout is formed.
- **Primitive value encoding**: the significance/order used when a Rust primitive is converted to or from its value bits.
- **Padding**: bits that exist in the output buffer but are not part of `BIT_SIZE` when appropriate.
- **Overlap**: legal shared bits and which fields are write-enabled.

### Endianness clarification: two layout axes plus value encoding

The `Endianness` documentation in `src/build/mod.rs` shows that the old implementation ultimately has two physical placement axes:

1. `field_order`, which should become the direct `bit_traversal = "front" | "back"` policy.
2. `byte_order`, which should become the direct `reverse` policy.

In the rewrite these should not be represented as mode-dependent XORs. The intended layout policy should be explicit:

```rust
struct LayoutPolicy {
    bit_traversal: BitTraversal, // Front or Back
    reverse: bool,                // mirror physical byte indices
}
```

`Front` means the first field consumes the front of the logical bitstream and `Back` means it consumes the back. `reverse` mirrors the completed buffer's byte indices; it does not reverse the bits inside each byte. This is the simple, user-facing placement model described by the attached `Endianness` documentation.

There is one compatibility qualification: the current `endianness` values are not only placement policies. They also select the primitive bit significance order used by the resolver:

| Existing mode | Current primitive encoding | Default effective placement |
|---|---|---|
| `be` | standard / big-endian resolver | front, not reversed |
| `le` | alternate / little-endian resolver | front, not reversed |
| `ale` | standard / big-endian resolver | back, byte-reversed |

The existing source makes this visible in `Endianness::big`, `little_packed`, and `little_aligned`, and in `SolvedData::from_built`, where resolver strategy and field placement are selected together. The `simple_be.rs` and `simple_le.rs` fixtures also prove that `be` and `le` produce different bytes for the same field declarations even when their default physical placement is similar.

### Decision: retain aliases, unify the codec

The public convenience attribute remains:

```rust
#[bondrewd(endianness = "be")]
#[bondrewd(endianness = "le")]
#[bondrewd(endianness = "ale")]
```

These values are parsed once and normalized into the canonical plan. They must not select separate code-generation backends. The rewrite should remove the current split between standard and alternate resolver functions and use one codec for all three modes.

The unified codec should:

1. Choose one canonical internal representation during macro expansion and generate the runtime conversion into that representation. The preferred representation is a big-endian byte sequence or an integer bit accumulator, because it gives every runtime value a stable numbered bit sequence.
2. Use the same emitted read/write segment operations for every mode.
3. Reverse the source-bit selection order when the normalized policy requires little-endian significance.
4. Place the selected bits into their physical segments using compile-time-generated masks and shifts.
5. Generate only the operations required by the selected layout. A policy is known while the macro expands, so the output should contain no runtime `if endian` branch.

In other words, `to_be_bytes`/`from_be_bytes` may be the sole primitive conversion implementation. Little-endian behavior is produced by selecting value bits in the opposite significance order before placing them into the output segments; it does not require `to_le_bytes`/`from_le_bytes` or a second resolver family.

The internal model should therefore retain only a semantic bit-order value where required, not a conversion-function strategy:

```rust
enum ValueBitOrder {
    MostSignificantFirst,
    LeastSignificantFirst,
}

struct EncodingPolicy {
    layout: LayoutPolicy,
    value_bit_order: ValueBitOrder,
}
```

`ValueBitOrder` is an implementation detail derived from the legacy alias. It is not an additional user-facing attribute. The important compatibility rule is that the normalized `be`, `le`, and `ale` policies must reproduce the byte snapshots before the old resolver code is removed.

The current parser also has an inconsistency worth fixing during the rewrite: struct-level `reverse` is mapped to byte reversal in `StructDarlingSimplified`, while variant-level `reverse` is currently mapped to field reversal in `VariantDarlingSimplified`. The new normalized policy should apply the same direct meaning at object and variant scope, with an explicit precedence rule.

Do not represent these policies as XORs over booleans in the solver state. Use named `LayoutPolicy` and `ValueEncoding` values, then use one function to map a logical bit position to a physical bit position. This makes the old aliases, new traversal/reversal behavior, and nested layouts reviewable and testable.

### 4. Size and padding rules

The solver should calculate and retain all of these values separately:

- `defined_bit_size`: bits contributed by ordinary fields.
- `serialized_bit_size`: the final logical size after enum ID and any required layout contribution.
- `bit_size`: the value exposed as `Bitfields::BIT_SIZE`, including the current documented fill behavior.
- `byte_size`: `ceil(serialized_bit_size / 8)` or the enforced/fill size.
- `padding_ranges`: output bits that are zero-initialized and not read into ordinary fields.

Apply validation in this order:

1. Parse and validate field-local declarations.
2. Resolve field sizes and array totals.
3. Resolve enum IDs and ID width.
4. Lay out non-padding fields.
5. Validate `enforce_bits`, `enforce_bytes`, and `enforce_full_bytes` against the documented pre-fill or final-fill quantity.
6. Apply `fill_bits`/`fill_bytes` and compute the final byte array size.
7. Validate that every field segment is in bounds and every overlap is intentional.

The existing documentation distinguishes enforcement from fill; that distinction must be captured by tests before changing the implementation.

### 5. Enum system

Model an enum as a tagged union with one shared layout:

```text
[variant id range][maximum payload range]
```

The solver should:

1. Collect explicit IDs and discriminants.
2. Reject duplicate IDs.
3. Assign unspecified IDs deterministically to the lowest unused values, preserving current declaration-order behavior.
4. Compute the minimum ID width unless explicitly supplied.
5. Reject IDs that do not fit in the configured width.
6. Select the explicit `invalid` variant, or preserve the current default catch-all rule.
7. Calculate the maximum payload width across all variants.
8. Place each variant payload into the same maximum-size output layout.
9. Decode unknown IDs into the invalid variant without panicking.
10. Handle `capture_id` as a read-only field backed by the ID bits; it must not allow `into_bytes` to emit an inconsistent variant ID.

Generated enum code should use a single decode match and one encode match generated from the same enum plan. The `id()` and `write_variant_id` behavior must be covered as part of the compatibility suite.

### 6. Read/write backend

Generate a small set of internal operations from `FieldLayout`:

- `read_segments(bytes) -> unsigned accumulator`.
- `write_segments(bytes, value_bits)` with clear-then-set masking.
- `read_signed` with explicit sign extension from the field width.
- `read_nested`/`write_nested` through the nested `Bitfields` implementation.
- Array expansion/reassembly for element arrays and block arrays.

**Static Safety Invariants**: For every field, the generator should emit a `const _: () = assert!(...)` check. This invariant must verify at compile-time that the solver's calculated `physical_segments` and bit-shifts are within the bounds of the target Rust primitive and the output buffer. This protects against logical bugs in the solver that might otherwise result in runtime panics or silent bit-corruption.

Each generated public function should call these operations inline through quoted code or a private generated helper. The implementation should avoid runtime loops for fixed layouts when that is a documented performance goal; the IR can still be built using loops at macro-expansion time.

Writing must preserve unrelated bits. For every segment:

```text
byte = (byte & !segment_mask) | (source_fragment & segment_mask)
```

Reserve/read-only/redundant policies must be applied by the generator, not by the low-level bit operation:

- `reserve`: read functions remain available, but `from_bytes` ignores the serialized value and constructs `Default::default()`; `into_bytes` writes zero.
- `read_only`: read functions remain available, but ordinary object conversion does not write the field.
- `redundant`: the field contributes no new layout size and is read but not written, equivalent to the documented read-only/overlap combination.
- `overlapping_bits`: the field may share the specified amount of already assigned bits, but the solver must verify that the overlap is actually in bounds and intentional.

### 7. Derive flavor adapters

The five derive macros should share one generated `EncodingPlan` and differ only in the outer API adapter:

- **Fixed adapter**: fixed `[u8; N]`, infallible length, `Bitfields<N>` implementation.
- **Dynamic adapter**: length-check `&[u8]`; vector adapter checks first, decodes, then drains exactly `N` bytes.
- **Slice adapter**: checks `N` once when constructing a checked wrapper; field access uses the wrapper without repeated whole-structure checks. Direct `read_slice_`/`write_slice_` functions check only the bytes needed for that field, preserving current documentation.
- **Hex adapter**: fixed hex array conversion around the fixed byte adapter.
- **Dynamic hex adapter**: slice/vector length and character validation around the same byte adapter.

The adapters must use the runtime paths `bondrewd::...` and remain valid when the consumer crate is `no_std`. `std`-only APIs must remain cfg-gated exactly as the runtime traits are.

Each adapter must reuse the same precomputed plan. Dynamic and slice adapters may perform their required length checks at runtime, but they must not redo layout calculations. Checked wrappers should store only the validated slice reference; they should not store runtime field offsets or masks.

### 8. Diagnostics and generated names

Introduce a dedicated validation error type internally, but continue returning `syn::Error` from the proc-macro entry point. Aggregate errors where possible so a user sees all independent attribute conflicts in one compile.

Preserve or deliberately document compatibility for generated names:

- `read_<field>`, `write_<field>`.
- `read_slice_<field>`, `write_slice_<field>`.
- Variant-prefixed helper names for enum fields.
- Checked wrapper type names and enum variant names.
- `variant_id`, `id()`, and `write_variant_id` helpers.

Use `quote_spanned!` for field-level generated code so type and attribute errors point back to the relevant user field.

## Step-by-step implementation plan

### Phase 0: Freeze the external contract

1. Add a new workspace member for `bondrewd-derive-next`; do not replace or rename the legacy package yet.
2. Copy only the minimum public test fixtures and derive-facing documentation needed to exercise the new crate. Do not copy the legacy implementation as a starting point; the new crate should have an independent architecture.
3. Add a comparison integration-test harness that imports both derive crates under aliases and generates equivalent old/new fixture types.

**Shared Test DSL**: To prevent manual duplication errors, Phase 0 must include the creation of a `test_bitfield!` macro. This macro should take a single bitfield definition (fields and attributes) and expand it into both a Legacy-derived struct and a Next-derived struct, along with the standard comparison assertions.
4. Inventory all derive exports, runtime trait methods, generated helper names, attributes, feature gates, and supported input forms.
5. Turn every runnable example in `bondrewd-derive/src/lib.rs` into a maintained test where it is not already covered.
6. Correct documentation contradictions before using it as a specification, especially wording around `bit_traversal`, `BIT_SIZE`, fill, enum invalid variants, and `ale`.
7. Record current output bytes for representative layouts. These byte snapshots are the compatibility oracle, even when the current behavior is surprising.
8. Keep the legacy implementation available as a first-class comparison dependency throughout the rewrite; do not rely on a temporary feature flag inside the old generator.

### Phase 1: Characterization and regression coverage

1. Add table-driven tests for each primitive width, partial width, signed value, float, `char`, nested type, and array mode.
2. Add a matrix covering `be`, `le`, `ale`, `reverse`, both traversal values, and combinations that are currently accepted.
3. Add a dedicated nested-structure matrix covering child and parent layouts, child and parent modes, bit offsets 0 through 7, byte-aligned and non-byte-aligned widths, `bit_length`, `byte_length`, fill, reverse, and multi-byte values.
4. Add nested round-trip tests that compare direct child serialization with the exact bits embedded in the parent.
5. Add nested enum tests for every supported child variant, invalid IDs, `capture_id`, payload padding, and parent fields that begin or end mid-byte.
6. Add round-trip tests for fixed, dynamic, checked-slice, and hex derives.
7. Add tests asserting unrelated bits survive each field write, including nested writes.
8. Add compile-fail tests for duplicate IDs, invalid widths, conflicting attributes, bad array declarations, unsupported types, and enforcement mismatches.
9. Add overlap tests for structs and enums before rewriting the overlap solver.
10. Add the structured fuzzing protocol described below, in addition to simple byte round-trip fuzzing.
11. Mark the known current failure, `ale_multi_byte_no_shift`, as a characterization test with the expected desired behavior rather than hiding it.

### Required structured fuzzing protocol

The fuzz suite must contain several targets built from a small set of deliberately different fixtures, not only one randomly generated structure. At minimum, include:

- A direct primitive structure with booleans, signed and unsigned partial-width integers, full-width values, and arrays.
- A nested-structure fixture with children positioned at byte offsets 0 through 7 and with both byte-aligned and non-byte-aligned child widths.
- A direct enum fixture with unit, tuple, and named variants, explicit IDs, an invalid variant, and `capture_id` where supported.
- A nested-enum fixture where an enum and a structure are both used as children, including a child payload that crosses multiple parent bytes.
- The important fixtures under `be`, `le`, and `ale`, with `reverse` and `bit_traversal` combinations where those combinations are accepted.

The fuzz input should produce valid values of these types using `arbitrary` or an equivalent generator. It should constrain values to the representable field widths and valid enum/`char` domains so a failure identifies a codec or generated-accessor problem rather than an invalid Rust value. A separate raw-byte fuzz target should still be used for invalid enum IDs, malformed padding, and arbitrary input-byte decoding.

For each generated value, the test must preserve an untouched original copy:

```text
original = fuzz_value
working = original.clone()
bit_stream = working.into_bytes()
```

The structured fuzz test then performs this exact sequence.

#### Pass 1: verify reads against the original value

1. Call every applicable generated `read_<field>` function with `bit_stream`.
2. Compare each result with the corresponding field in `original`.
3. For nested fields, compare the complete nested value and recursively verify the child's fields where the fixture exposes them.
4. For enums, inspect the active variant, read its fields with the generated variant-specific helpers, and verify the ID/`id()` result. Inactive variant payload helpers should be tested separately only when their API contract says they are valid.
5. Use bitwise comparison for floating-point fields (`to_bits`) and domain-aware comparison for `char`, signed partial-width values, reserved fields, and captured IDs.

#### Pass 2: write inverted values and decode them

1. Create a second value containing an inverted or otherwise deliberately changed value for every writable field, while retaining the original copy for later restoration.
2. The inversion must remain representable:
   - Toggle `bool` values.
   - Complement unsigned values within the declared field mask.
   - Complement signed values in the declared two's-complement width and sign-extend them back to the Rust type.
   - Recursively invert nested structures and arrays.
   - Preserve the active enum variant while inverting its payload; use separate cases to exercise variant-ID writes and variant changes.
   - Change `char` values through a valid deterministic mapping.
   - Use bit-pattern comparisons or a finite-value mapping for floats so NaN equality does not create false failures.
3. Starting with the original `bit_stream`, call every generated `write_<field>` function with the changed values. For enums, call the active variant field writers and the variant-ID writer where appropriate.
4. Call `from_bytes` on the modified bit stream.
5. Verify that every writable field in the decoded value equals the changed value and that unrelated fields were preserved.
6. Verify the documented behavior of reserve, read-only, redundant, and captured-ID fields instead of treating those fields as ordinary writable fields.

#### Pass 3: re-read the changed value

1. Clone the newly decoded value and call `into_bytes` again to create a new bit stream.
2. Call every applicable `read_<field>` function against this new bit stream.
3. Compare all read results with the changed value from Pass 2, including recursive nested values and active enum payloads.

#### Pass 4: restore the original value through write accessors

1. Starting with the new bit stream, call every generated `write_<field>` function again, this time using the values from the untouched `original` copy.
2. Use the same active-variant and ID rules as Pass 2.
3. Call `from_bytes` again to produce the newest structure or enum.
4. Compare the newest value with the original value field-by-field and recursively. For floats compare bit patterns; for reserve/read-only/redundant fields compare according to their documented serialization semantics.
5. Re-run the read assertions one final time against the restored bit stream so the test proves both directions of every generated accessor.

The required invariant is therefore:

```text
original
  -> into_bytes
  -> every read_* equals original
  -> every write_* with changed values
  -> from_bytes == changed
  -> every read_* equals changed
  -> every write_* with original values
  -> from_bytes == original
  -> every read_* equals original again
```

Run this protocol independently against the legacy and next derive crates. For equivalent fixtures, compare both implementations' bit streams after each pass. A legacy/new difference must be classified using the same compatibility rules as the ordinary comparison harness, while a new-implementation failure is a fuzz regression even if the legacy implementation produces the same incorrect result.

Fuzz targets should retain failing inputs as corpus files and promote every minimized failure into a deterministic regression test. The fuzz suite should run for all three endianness aliases and should include nested cases in every supported layout mode, not only `be`.

### Phase 2: Build the pure layout engine

1. Add the parsed model and normalize attributes without generating tokens.
2. Implement `ValueType`, `FieldType`, `SizeSpec`, `ArraySpec`, nested metadata, and enum metadata.
3. Implement the explicit `LayoutPolicy` and unit-test logical-to-physical bit mapping independently of Rust types.
4. Implement scalar field allocation, explicit ranges, reversal, padding, and byte-size calculation.
5. Implement segment generation for fields crossing one or more bytes.
6. Implement nested projections and validate exact, padded, and explicitly truncated child widths.
7. Implement validation for bounds, overlap policies, size enforcement, fill, and unsupported combinations.
8. Implement enum ID allocation and shared payload layout.
9. Serialize or pretty-print the IR in debug tests so layout regressions are easy to inspect.

**Bit Layout Visualization**: The layout engine should be capable of producing a visual Markdown or ASCII bit-table diagram of the resolved structure. This diagram must be included in the `dump` output or test logs to make manual verification of complex `ale` or `reverse` layouts trivial.

At the end of this phase, no proc-macro token generation should be necessary to prove that a layout is correct.

### Phase 3: Implement one canonical scalar codec

1. Implement read and write segment operations for unsigned values.
2. Add signed sign-extension based on the declared field width, not the host integer width.
3. Add bool, char, and float conversions using the same canonical bit representation.
4. Make writes clear only their own segments.
5. Compare generated bytes to the Phase 1 snapshots for big endian and simple little endian first.
6. Add aligned little endian and reverse only after the canonical path is correct.
7. Eliminate zero-shift and invalid rotate operations during plan construction; a zero shift should generate no shift expression.
8. Inspect generated tokens or assembly to verify that no layout arithmetic or endian branch remains in the scalar runtime path.

### Phase 4: Implement nested composition

1. Implement child-to-parent bit projection using arbitrary segment lists rather than integer rotations.
2. Implement parent-to-child extraction into a zero-initialized fixed child buffer.
3. Prove exact-width nested structs for every layout mode before supporting padding or truncation.
4. Add zero-padding and explicitly requested truncation with byte snapshots and compile-fail cases for ambiguous projections.
5. Apply the same nested plan to direct reads, writes, `from_bytes`, `into_bytes`, and checked slice access.
6. Add nested enum composition only after standalone enum encoding is available; the parent must treat the enum as an opaque child serializer.
7. Make `ale_multi_byte_no_shift` pass as a required regression gate.

### Phase 5: Add arrays and overlap policies

1. Generate element subplans from multidimensional array dimensions.
2. Generate block-array chunks that consume the declared total block width and preserve the documented dropped-bit behavior.
3. Reassemble arrays in source order for reads and destructure them in source order for writes.
4. Add reserve, read-only, redundant, and allowed-overlap write policies.
5. Validate that redundant fields do not increase total size, while overlapping fields do not silently create out-of-bounds segments.
6. Add nested element-array and nested block-array tests.

### Phase 6: Add enum encoding

1. Reuse the canonical codec for the generated ID field.
2. Generate the variant payload plans against the maximum payload width.
3. Generate fixed `from_bytes` and `into_bytes` matches.
4. Add invalid-variant construction and `capture_id` handling.
5. Verify tuple and named variant construction separately.
6. Verify enums as nested children, including invalid IDs and parent bit offsets.
7. Add `BitfieldsSlice` checked enum wrappers only after fixed enum conversion is stable.

### Phase 7: Add API adapters

1. Generate `Bitfields` and fixed read/write helpers from the canonical plan.
2. Generate `BitfieldsDyn` from the same plan, including vector consumption semantics.
3. Generate direct slice field functions and checked immutable/mutable wrappers.
4. Generate fixed and dynamic hex APIs, preserving exact error types and character indexes.
5. Run all feature combinations: no default features, `derive`, `std`, and `full`.
6. Compile a consumer crate with `#![no_std]` to verify generated code does not accidentally require `std`.

### Phase 8: Compatibility cutover

1. Run the side-by-side comparison harness with the legacy and next derive crates against the complete characterization corpus.
2. For every byte or API difference, classify it as an intentional bug fix, an undocumented behavior, or a compatibility regression.
3. Preserve undocumented behavior when it is relied upon and safe; otherwise document the migration before changing it.
4. Keep both crates as workspace members through the release-candidate cycle so regressions can still be bisected against the legacy oracle.
5. Publish the new implementation under an explicitly versioned package or migrate the package name only after the comparison suite is green.
6. Update the `bondrewd` facade to point at the new derive crate only as part of the planned release transition.
7. Remove the legacy crate from the default workspace only after the new package has passed the release process and a final compatibility review.
8. Remove dead generation code, commented-out alternatives, and `allow(dead_code)` suppressions only from the new implementation; do not rewrite history in the legacy oracle.

### Phase 9: Documentation and release

1. Update the new crate's `lib.rs` examples to be executable doctests and ensure terminology matches the implementation.
2. Document the exact meaning of `be`, `le`, and `ale` with byte diagrams generated from the comparison tests.
3. Document whether fill bits contribute to `BIT_SIZE`, byte size, or only physical padding; do not leave this implicit.
4. Publish a migration note listing any changed diagnostics or previously accepted invalid combinations.
5. Release the new implementation behind a beta version while retaining the legacy derive crate and comparison harness.

## Validation gates

Each phase should have a concrete gate:

- **Parser gate**: all documented valid examples parse; all invalid attribute combinations produce errors.
- **Layout gate**: every field has in-bounds segments; intentional overlaps are the only overlaps.
- **Codec gate**: fixed encode/decode round trips and byte snapshots pass for all layout policies.
- **API gate**: fixed, dynamic, checked-slice, and hex APIs agree on the same bytes.
- **Safety gate**: generated code remains safe Rust; no unchecked indexing is emitted unless compile-time layout guarantees make it provably in bounds.
- **`no_std` gate**: the base runtime and generated fixed/slice code compile without `std`; vector APIs remain gated.
- **Performance gate**: generated fixed layouts contain no unnecessary runtime layout calculations, zero shifts, or whole-structure decode when a field read is requested.
- **Compile-time specialization gate**: generated code uses literal precomputed offsets, masks, shifts, segment widths, nested projections, and enum IDs; fixed layouts do not contain runtime metadata walks, layout arithmetic, or endian branches.
- **Code-generation gate**: fixed-size fields and ordinary arrays are unrolled by the macro. Any runtime loop must be justified by generated-code-size measurements and must operate only on compile-time-defined ranges.
- **Assembly/benchmark gate**: representative fixed, nested, enum, and array layouts are inspected or benchmarked to verify that the rewrite moves layout work out of runtime execution.
- **Structured fuzz gate**: every structured fuzz target completes the original/read/inverted-write/decode/re-read/original-restore cycle for direct structs, nested structs, direct enums, and nested enums under `be`, `le`, and `ale`.
- **Fuzz comparison gate**: legacy and next derive crates are both exercised with the same fuzz corpus; minimized failures become deterministic regression tests and unexpected old/new differences block cutover.
- **Compatibility gate**: public names, trait implementations, constants, error types, and field-level behavior match the frozen contract.

Run at minimum:

```text
cargo fmt --all -- --check
cargo test --workspace --all-features
cargo test -p bondrewd-derive --test <focused-test>
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fuzz run <structured-target> -- -runs=100000
```

Run the fuzz command from the fuzz package directory and repeat it for each structured target and endianness configuration. The exact run count is a minimum smoke-test budget; CI should also run longer jobs and replay all checked-in corpus files.

The current workspace baseline is not completely green: `bondrewd-derive/tests/aligned.rs` currently fails at `ale_multi_byte_no_shift` during `cargo test --workspace --all-features`. The failure reconstructs a nested aligned-little-endian value into the wrong inner field. This should be fixed by the canonical segment plan and retained as a regression test; it should not be papered over by weakening the test.

## Risks and decisions to make before coding

1. **Exact legacy semantics vs corrected semantics**: `ale`, `reverse`, and non-byte-aligned nested values are the highest-risk areas. Freeze byte snapshots first.
2. **`BIT_SIZE` and fill**: current documentation contains nuanced and partially inconsistent statements. Decide the compatibility rule explicitly.
3. **Invalid `char` values**: choose whether decoding uses a checked constructor, replacement behavior, or a compile-time restriction. Test and document the choice.
4. **Nested composition**: the child layout must remain authoritative while the parent places serialized child bits into its own field range. Do not revive the unfinished `Endianness::merge` behavior or treat nested bytes as a primitive integer. Exact-width nesting should be completed before padding, truncation, and nested arrays.
5. **Nested projection**: parent fields may be wider or narrower than a child representation in existing documentation. Define which bits are padded or truncated and reject ambiguous projections rather than silently changing the selected bits.
6. **Generated code size**: segment expansion can increase token count for large arrays. Add a threshold or generated helper strategy if expansion becomes excessive, without introducing runtime layout calculations for ordinary fixed fields.
7. **Public internal APIs**: the current `build` and `solved` modules are private to the proc-macro crate, so their types may be redesigned freely. Only generated consumer-facing APIs and the runtime crate traits are compatibility constraints.
8. **Diagnostics**: exact text should not be treated as stable, but source spans and actionable explanations should improve. If downstream tests depend on messages, preserve them during the beta period.

## Resolved decisions

The following decisions are now part of this proposal:

1. Keep `endianness = "be" | "le" | "ale"` as convenient public compatibility aliases.
2. Make `bit_traversal` and `reverse` the canonical physical layout concepts.
3. Give `reverse` one meaning everywhere: mirror physical byte indices; it must not switch between byte reversal and field traversal depending on whether it appears on a struct, enum, or variant.
4. Replace the separate BE/LE resolver families with one canonical codec. Prefer `to_be_bytes`/`from_be_bytes` (or one equivalent canonical bit representation) and generate the selected bit order through static masks, shifts, and segment order.
5. Require the macro expansion to precompute every fixed-layout mask, shift, segment, nested projection, and enum dispatch arm; runtime code may only consume values and bytes.
6. Preserve the old byte output through characterization tests before deleting the old resolver implementation.

## Recommended implementation order

The safest order is:

1. Characterize and freeze outputs.
2. Implement logical-to-physical bit mapping.
3. Implement scalar segments and fixed `Bitfields`.
4. Implement exact-width nested structures.
5. Add nested padding/truncation rules and nested arrays.
6. Add overlap/reserve/read-only policies.
7. Add standalone enums and then nested enums.
8. Add dynamic and checked-slice adapters.
9. Add hex adapters.
10. Compare, cut over, document, and remove the legacy generator.

This order keeps the difficult bit-placement rules independent from the number of generated API flavors and makes nested serialization a core compatibility gate rather than a late feature. It also makes the existing nested `ale` failure diagnosable as a layout/segment problem instead of an incidental bug in one generated `from_bytes` path.
