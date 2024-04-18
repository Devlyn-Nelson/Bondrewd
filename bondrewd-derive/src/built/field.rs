use crate::{common::field::DynamicIdent, gen::field::QuoteInfo};

pub struct Solved {
    name: DynamicIdent,
    resolver: Resolver,
    // TODO the le_single byte uses the starting bit in attrs, might want to
    // store starting_bit instead of `starting_inject_byte` and do the math
    // at the time needed.
    qi: QuoteInfo,
}

pub enum Resolver {
    // TODO START_HERE make a solved bondrewd field that is used for generation and future bondrewd-builder
    // Basically we need to removed all usages of `FieldInfo` in `gen` and allow `Info` to be an
    // active builder we can use for bondrewd builder, then solve. bondrewd-derive would then
    // use `Solved` for its information and `bondrewd-builder` would use a `Solved` runtime api to
    // access bondrewd's bit-engine at runtime.
    //
    // Also the `fill_bits` that make enums variants expand to the largest variant size currently get added
    // after the byte-order-reversal. This would make it so the `Object` could: parse all of the variants
    // one at a at, until a solve function is called, which then grabs the largest variant, does a
    // auto-fill-bits operation on variants that need it, THEN solve the byte-order for all of them,
    // Each quote maker (multi-byte-le, single-byte-ne, there are 6 total) will become a FieldHandler
    // that can be used at runtime or be used by bondrewd-derive to construct its quotes.
    StandardSingle(StandardSingle),
    StandardMultiple(StandardMultiple),
    AlternateSingle(AlternateSingle),
    AlternateMultiple(AlternateMultiple),
    NestedSingle(NestedSingle),
    NestedMultiple(NestedMultiple),
}

impl Solved {
    pub(crate) fn generate_fn_quotes(&self) {
        todo!("Solved should get all of the generation code, without needing the Info structures.");
    }
    pub fn read(&self) {
        todo!(
            "Solved should use generation information to perform runtime getting/setting of bits"
        );
    }
    pub fn write(&self) {
        todo!(
            "Solved should use generation information to perform runtime getting/setting of bits"
        );
    }
}

pub struct StandardSingle {}
pub struct StandardMultiple {}

pub struct AlternateSingle {}
pub struct AlternateMultiple {}

pub struct NestedSingle {}
pub struct NestedMultiple {}
