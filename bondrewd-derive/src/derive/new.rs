use std::range::Range;

use proc_macro2::TokenStream;

use crate::build::Endianness;

pub struct StructAttributes {
    endianess: Endianness,
}

pub struct FieldAttributes {
    name: NameOrIndex,
    bits: FieldBits,
}

pub enum NameOrIndex {
    Name(String),
    Index(usize),
}

impl NameOrIndex {
    pub fn name(&self) -> String {
        match self {
            NameOrIndex::Name(name) => name.clone(),
            NameOrIndex::Index(index) => format!("field_{index}"),
        }
    }
}

pub struct FieldBits {
    ranges: Vec<Range<usize>>,
}

/// `access` is a [`TokenStream`] for accessing the field from the field.
pub fn make_write_code(
    s: &StructAttributes,
    f: &FieldAttributes,
    access: &TokenStream,
) -> TokenStream {
    todo!("output tokenstream for writing field to bytes");
}

/// `access` is a [`TokenStream`] for accessing the field from the field.
pub fn make_read_code(
    s: &StructAttributes,
    f: &FieldAttributes,
    access: &TokenStream,
) -> TokenStream {
    todo!("output tokenstream for reading field from bytes");
}
