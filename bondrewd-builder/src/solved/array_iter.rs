use std::ops::Range;

use crate::build::field::DataType;

use super::field::DynamicIdent;

pub struct ElementArrayIter {
    outer_ident: DynamicIdent,
    /// An iter that returns the index of the element we are returning information for.
    element_range: Range<usize>,
    // The starting bit index of the first element
    starting_bit_index: usize,
    // type the array is holding.
    ty: BuiltDataType,
    // The amount of bits an single element consumes.
    element_bit_size: usize,
}

impl ElementArrayIter {
    // creates a new ElementArrayIter with `elements` array length.
    pub fn new(
        outer_ident: DynamicIdent,
        ty: BuiltDataType,
        starting_bit_index: usize,
        elements: usize,
        element_bit_size: usize,
    ) -> Self {
        Self {
            outer_ident,
            element_range: 0..elements,
            starting_bit_index,
            ty,
            element_bit_size,
        }
    }
}

impl Iterator for ElementArrayIter {
    type Item = BuiltDataTypeInfo;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.element_range.next() {
            let start = self.starting_bit_index + (index * self.element_bit_size);
            let bit_range = start..start + self.element_bit_size;
            let outer_ident = self.outer_ident.ident().clone();
            let name = format!("{outer_ident}_{index}");
            let ident = DynamicIdent::new_ident(name, outer_ident);
            Some(BuiltDataTypeInfo {
                ty: self.ty.clone(),
                bit_range,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct BlockArrayIter {
    pub outer_ident: DynamicIdent,
    // Starts as array length, but is decremented each time next is called.
    pub remaining_elements: usize,
    // the starting bit index of the first element
    pub starting_bit_index: usize,
    // The amount of bytes the rust type is
    pub ty: BuiltDataType,
    // Amount of remaining bits to consume.
    pub remaining_bits: usize,
    // Total amount of bytes the iterator will consume when `None` is the return of `self.next()`.
    pub total_elements: usize,
}

impl BlockArrayIter {
    // creates a new ElementArrayIter with `elements` array length.
    pub fn new(
        outer_ident: DynamicIdent,
        ty: BuiltDataType,
        starting_bit_index: usize,
        elements: usize,
        range: Range<usize>,
    ) -> Self {
        Self {
            outer_ident,
            starting_bit_index,
            ty,
            total_elements: elements,
            remaining_elements: elements,
            remaining_bits: range.end - range.start,
        }
    }
}

impl Iterator for BlockArrayIter {
    type Item = BuiltDataTypeInfo;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_elements != 0 {
            let mut ty_size = self.ty.rust_bytes_size() * 8;
            if self.remaining_bits % ty_size != 0 {
                ty_size = self.remaining_bits % ty_size;
            }
            let start = self.starting_bit_index;
            self.starting_bit_index = start + ty_size;
            let bit_range = start..(start + ty_size);
            self.remaining_bits -= ty_size;
            let index = self.total_elements - self.remaining_elements;
            let outer_ident = self.outer_ident.ident().clone();
            let name = format!("{outer_ident}_{index}");
            let ident = DynamicIdent::new_ident(name, outer_ident);
            self.remaining_elements -= 1;
            Some(BuiltDataTypeInfo {
                bit_range,
                ty: self.ty.clone(),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct BuiltDataTypeInfo {
    pub(crate) ty: BuiltDataType,
    /// The range of bits that this field will use.
    pub(crate) bit_range: Range<usize>,
}

#[derive(Clone, Debug)]
pub struct BuiltDataSubType {
    sub: Box<BuiltDataTypeInfo>,
}

#[derive(Clone, Debug)]
pub enum BuiltDataType {
    Single(DataType),
    BlockArray {
        elements: usize,
        sub: BuiltDataSubType,
    },
    ElementArray {
        elements: usize,
        sub: BuiltDataSubType,
    },
}

impl BuiltDataType {
    pub fn rust_bytes_size(&self) -> usize {
        match self {
            BuiltDataType::Single(dt) => dt.rust_size(),
            BuiltDataType::BlockArray { elements, sub }
            | BuiltDataType::ElementArray { elements, sub } => {
                sub.sub.ty.rust_bytes_size() * elements
            }
        }
    }
}
