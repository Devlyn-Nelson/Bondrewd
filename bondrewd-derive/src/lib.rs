#![allow(unreachable_code, dead_code, unused_variables)]

use build::field_set::GenericBuilder;
use proc_macro::TokenStream;
use solved::field_set::Solved;
use syn::{parse_macro_input, DeriveInput};

mod build;
mod derive;
mod masked;
mod solved;
// TODO I think a further calculated model is possible beyond the current solved.
// currently Solved is a small data package describing the area of bits and the
// technique to to calculate the actual masks and byte indices. This new model
// could be called `SolvedMasks` and would store a lot of very specific information
// each bytes masks. currently `bondrewd-derive` has a ParsedModel into a Solved model
// design already. This would separate part of the "Resolver" functions which take
// the solved model and calculates the masks and the byte indexes to access data in byte
// streams and figure out how they relate to rust native types. So essentially
// `bondrewd-derive` currently does
//
// `Parse -> Solved -> Masks/MakeDeriveFunctions`
//
// this change would make it
//
// `Parse/Builder -> Solved -> Masks -> MakeDeriveFunctions or RuntimeAccess`
//
// it would be important to make the `Solved -> Masks` step very fast because `Masks` will
// use more data at runtime. so having a specialty runtime function that rather than make the
// entire `Masks` model at once, it would preform them as needed throwing each `Masks` per
// field away after it is used. this would offer 2 options for storing these runtime models.
// using `Solved` would be space efficient but would need to re-calculate the masks each time
// to access a field or fields. Where as storing a `Masks` model would have less runtime cpu
// cost but storing each mask or multiple masks per byte can take really inflate the stored
// model size. None of this should effect `bondrewd-derive` in a negative way but change the
// actual function writing code to use the `Masks` model to make the function rather than
// calculating the masks and writing the function at the same time.
#[proc_macro_derive(Bitfields, attributes(bondrewd,))]
pub fn derive_bitfields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // parse the input into a StructInfo which contains all the information we
    // along with some helpful structures to generate our Bitfield code.
    let struct_info = match GenericBuilder::parse(&input) {
        Ok(parsed_struct) => parsed_struct,
        Err(err) => {
            return TokenStream::from(err.to_compile_error());
        }
    };
    let solved: Solved = match struct_info.try_into() {
        Ok(s) => s,
        Err(err) => {
            let err: syn::Error = err.into();
            return TokenStream::from(err.to_compile_error());
        }
    };
    match solved.gen(true, true, false) {
        Ok(gen) => gen.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
