# Rewrite Design Concerns and Follow-Up Decisions

This document records concerns identified during review of [`REWRITE_PROPOSAL.md`](REWRITE_PROPOSAL.md). It is intentionally separate from the main proposal so the implementation plan can remain stable while these questions are resolved.

## Priority summary

The rewrite should not begin implementation until these three areas are formally defined:

1. The exact logical-bit-to-physical-bit mapping for all endianness and traversal modes.
2. The composition and projection rules for nested structures and nested enums.
3. The meaning and scope of `reverse`, especially on enum variants.

The remaining concerns should be resolved before compatibility testing and release preparation.

## 1. Endianness needs a formal mapping function

The proposal currently defines:

```rust
struct LayoutPolicy {
    bit_traversal: BitTraversal,
    reverse: bool,
}

enum ValueBitOrder {
    MostSignificantFirst,
    LeastSignificantFirst,
}
```

This is a useful abstraction, but it is not yet a complete wire-format definition. The implementation still needs one authoritative mapping function, conceptually:

```rust
fn map_bit(
    value_bit: usize,
    field_range: Range<usize>,
    policy: EncodingPolicy,
    output_size: usize,
) -> PhysicalBit;
```

The mapping must define:

- Which value bit is selected first.
- Whether `bit_traversal` changes field placement, value-bit order, or both.
- When `reverse` is applied.
- How partial-byte fields are handled.
- How fields crossing multiple bytes are handled.
- How `be`, `le`, and `ale` map exactly.
- How the mapping composes with explicit `bits = "START..END"` ranges.
- How the mapping composes with fill and non-byte-aligned total sizes.

### Required action

Create an independent reference bit mapper before implementing the new generator. It should produce physical byte output from a logical model without reusing the legacy resolver code.

Use it to generate golden vectors for:

- `be`, `le`, and `ale`.
- `front` and `back` traversal.
- `reverse` and non-reversed layouts.
- Single-byte and multi-byte fields.
- Fields beginning at every bit offset from 0 through 7.
- Fields ending at every bit offset from 0 through 7.
- Signed and unsigned partial-width values.
- Nested values and enums.

The legacy implementation should be compared against this model, not treated as the only correctness oracle.

## 2. Nested structure projection is not fully defined

The proposal correctly treats nested types as child serializers whose internal layout is authoritative. However, the behavior is still ambiguous when the parent's field width differs from the child's representation.

Example:

```text
child BIT_SIZE  = 13
child BYTE_SIZE = 2
parent field    = 13 bits
parent field    = 16 bits
parent field    = 9 bits
```

The proposal distinguishes:

```rust
enum NestedProjection {
    Exact,
    ZeroPad { field_bits: usize },
    Truncate { field_bits: usize },
}
```

But truncation still needs a precise definition. If a 13-bit child is placed into a 9-bit parent field, the implementation must specify which four child bits are discarded:

- Highest-order bits.
- Lowest-order bits.
- Bits selected according to the child's own layout.
- Bits selected according to the parent's layout.

This must not be inferred from runtime endianness logic.

### Recommended staged approach

1. Initially require exact-width nested fields.
2. Support wider parent fields through explicitly defined zero padding.
3. Add truncation only after its direction is documented and covered by golden vectors.
4. Reject ambiguous nested projections at compile time.

## 3. Variant-level `reverse` may be circular

If `reverse` applies to an entire enum representation, variant-level reversal creates a decoding problem:

1. The decoder must read the enum ID.
2. The ID selects the variant.
3. The selected variant determines whether bytes should be reversed.
4. The decoder would need the variant before it can reliably read the variant ID.

Possible policies include:

1. Allow `reverse` only on the enum object.
2. Apply variant-level `reverse` only to the payload, while keeping the ID in a fixed location.
3. Generate support for multiple possible ID layouts before selecting a variant.

The first option is the safest. If variant-level `reverse` is retained, its scope must be explicitly documented and tested.

## 4. The legacy crate is not a complete correctness oracle

The side-by-side comparison between `bondrewd-derive` and `bondrewd-derive-next` is necessary, but it is not sufficient because the legacy implementation already has known bugs, including nested aligned-little-endian decoding.

Matching legacy behavior may preserve an existing bug. The comparison system should therefore use three sources:

1. Legacy implementation output.
2. New implementation output.
3. Independent expected output from:
   - The reference bit mapper.
   - Hand-authored golden vectors.
   - Mathematical invariants.

Every old/new difference should be classified as:

- Equivalent behavior.
- Intentional correction of a documented or confirmed bug.
- Unexpected regression.

A difference must not automatically be treated as a new implementation failure.

## 5. Structured fuzzing needs expected-value normalization

The proposed fuzz sequence is valuable, but direct comparison with the raw inverted Rust value will fail for values that cannot be represented exactly by the field layout.

Examples include:

- A `u8` field with `bit_length = 3`.
- A signed field with fewer bits than its Rust type.
- Reserved fields.
- Read-only and redundant fields.
- Captured enum IDs.
- Floating-point `NaN` values.
- Invalid Unicode scalar values produced by bitwise inversion.

The fuzz harness needs a field-aware normalization step:

```text
expected_serialized_value =
    normalize_for_field_policy(changed_value)
```

Expected-value comparisons should use the normalized value.

Recommended rules:

- Mask unsigned values to the declared field width.
- Sign-extend signed values from the declared field width.
- Compare floats by bit pattern using `to_bits()`.
- Mutate `char` through a valid deterministic mapping.
- Compare reserve fields according to their documented default behavior.
- Compare captured IDs separately from normal writable fields.
- Compare redundant fields based on their read/write policy rather than ordinary struct equality.

## 6. Overlapping field writes need explicit ordering

For overlapping fields, writing every field sequentially does not produce a unique result unless write ordering is specified.

For example:

```text
write_flags(...)
write_flag_one(...)
write_flag_two(...)
```

The final shared bits depend on which write occurs last.

The fuzz and regression suites should distinguish:

- Non-overlapping fields.
- Redundant/read-only fields.
- Intentionally overlapping fields.

For overlap cases, the test should either:

- Use the documented canonical write order.
- Check the result after each individual write.
- Compare only fields whose writes are expected to dominate.

## 7. Enum fuzzing needs separate mutation modes

Enum fields cannot all be inverted in the same way as struct fields.

Enum fuzz targets should include separate passes for:

1. Preserving the active variant while changing its payload.
2. Changing the variant ID with `write_variant_id`.
3. Decoding arbitrary IDs and verifying invalid-variant behavior.
4. Testing captured IDs independently.
5. Testing payload writers only for the active variant.

Writing fields belonging to inactive variants should not be treated as an ordinary object mutation unless that behavior is explicitly part of the public API contract.

## 8. Old/new fuzz comparison needs a shared semantic input

The legacy and new implementations use different Rust types, so they should not rely on separate `Arbitrary` implementations consuming fuzz bytes in exactly the same way.

Instead, generate one shared semantic model:

```rust
enum SemanticValue {
    Struct { /* normalized fields */ },
    Enum { /* variant and normalized payload */ },
}
```

Then convert it into both implementation-specific types:

```text
SemanticValue -> LegacyType
SemanticValue -> NewType
```

This guarantees that both implementations receive the same logical value. The same semantic input should be used for:

- Initial `into_bytes` comparison.
- Read-accessor comparison.
- Inverted-value writes.
- Restoration writes.
- Nested and enum cases.

## 9. Full code unrolling may create excessive generated code

The compile-time-first strategy is appropriate for performance, but fully unrolling every field and array can cause:

- Very large token streams.
- Slow proc-macro expansion.
- Slow downstream compilation.
- Large binaries.
- Large incremental compilation costs.
- Huge nested enum match expressions.

The implementation needs a code-generation size policy.

Recommended rule:

- Fully unroll ordinary fields and small fixed arrays.
- Precompute all layout metadata regardless of size.
- Use generated helper functions or bounded loops for very large arrays if measurements justify them.
- Never calculate layout metadata at runtime.
- Benchmark unrolled and helper-based strategies.

A runtime loop over a compile-time-defined range may be acceptable. A runtime calculation of field offsets, masks, segment widths, or endianness is not.

## 10. Runtime facade migration needs more detail

The parallel crate plan should define how the new derive crate eventually reaches users.

Questions to resolve:

- Will `bondrewd` eventually depend on `bondrewd-derive-next`?
- Will the new crate temporarily use a different published package name?
- Can users explicitly select the legacy generator during the beta period?
- How will versions of `bondrewd`, `bondrewd-derive`, and `bondrewd-derive-next` remain compatible?
- When will the old crate be removed from the default workspace?

A possible migration path is:

```text
bondrewd-derive       legacy implementation
bondrewd-derive-next  new beta implementation
bondrewd               continues using legacy initially
bondrewd               optional feature selects next implementation
future release         next becomes the default
later release          legacy implementation is removed
```

## 11. Additional API and implementation questions

These should also be explicitly checked before cutover:

- Whether field-level endianness is actually supported or only documented.
- Whether `id_byte_length` is fully implemented and must be preserved.
- How invalid `char` values are handled by `from_bytes`.
- Whether nested child endianness is always authoritative or can be overridden by the parent.
- Whether checked slice wrappers are generated with the same visibility and names.
- Whether `BitfieldsDyn` and `BitfieldsHexDyn` remain correctly gated under `std`.
- Whether generated code remains `no_std` when the proc-macro crate itself uses `std` internally.
- Whether `dump` output is excluded from correctness and compatibility guarantees.
- Whether diagnostics need exact text compatibility or only equivalent source spans and error meaning.

## Suggested next steps

Before implementing the new crate:

1. Write the reference bit mapper.
2. Define exact nested projection rules.
3. Decide the allowed scope of variant-level `reverse`.
4. Build golden vectors independent of the legacy generator.
5. Define fuzz-value normalization.
6. Define enum mutation passes.
7. Define the code-generation size budget.
8. Define the runtime facade migration path.

Once these decisions are recorded, update `REWRITE_PROPOSAL.md` to mark the resolved items and keep this document as the historical design review record.
