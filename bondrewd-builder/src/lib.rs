#![allow(unreachable_code, dead_code, unused_variables)]

pub mod build;
pub mod derive;
pub mod masked;
pub mod solved;
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
//
