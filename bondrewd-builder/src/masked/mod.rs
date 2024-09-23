//! TODO START_HERE
//! create the masked structures. this step is takes a resolver and gives
//! the final form that bondrewd-builder is made to create a `Resolved` type
//! which is meant to be a fully solved version of a struct or enum containing
//! all of the necessary information to build derive functions or access bits in
//! at runtime. it would also be assumed that the structure version shouldn't be
//! needed at runtime because it should only need to be come this far as fields
//! as they are needed.