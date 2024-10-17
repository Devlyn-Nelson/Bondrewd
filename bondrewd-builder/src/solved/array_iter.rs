use std::ops::Range;

use quote::format_ident;

use crate::build::{field::DataType, ArraySizings};

use super::field::{
    DynamicIdent, Resolver, ResolverArrayType, ResolverData, ResolverSubType, ResolverType,
};

pub struct ElementArrayIter {
    outer_ident: DynamicIdent,
    /// An iter that returns the index of the element we are returning information for.
    element_range: Range<usize>,
    // The starting bit index of the first element
    starting_bit_index: usize,
    // type the array is holding.
    ty: ResolverType,
    // The amount of bits an single element consumes.
    element_bit_size: usize,
    flip: Option<usize>,
}

impl ElementArrayIter {
    // creates a new ElementArrayIter with `elements` array length.
    pub fn new(
        outer_ident: DynamicIdent,
        ty: ResolverType,
        starting_bit_index: usize,
        elements: usize,
        element_bit_size: usize,
        flip: Option<usize>,
    ) -> Self {
        Self {
            outer_ident,
            element_range: 0..elements,
            starting_bit_index,
            ty,
            element_bit_size,
            flip,
        }
    }
    pub fn from_values(
        resolver_data: &ResolverData,
        sub_ty: &ResolverSubType,
        array_ty: &ResolverArrayType,
        sizings: &ArraySizings,
    ) -> Self {
        let mut sizings = sizings.clone();
        let elements = sizings.pop().expect("Array sizings had not data, meaning a non-array type was attempted to be used as an array type");
        let ty: ResolverType = if sizings.is_empty() {
            (*sub_ty).clone().into()
        } else {
            ResolverType::Array {
                sub_ty: sub_ty.clone(),
                array_ty: array_ty.clone(),
                sizings,
            }
        };
        let element_bit_size = resolver_data.bit_length() / elements;
        ElementArrayIter::new(
            resolver_data.field_name.clone(),
            ty,
            resolver_data.bit_range_start(),
            elements,
            element_bit_size,
            resolver_data.flip(),
        )
    }
}

impl Iterator for ElementArrayIter {
    type Item = Resolver;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.element_range.next() {
            let outer_ident = self.outer_ident.ident().clone();
            let name = format_ident!("{outer_ident}_{index}");
            let ident = (outer_ident, name).into();
            let start = self.starting_bit_index + (index * self.element_bit_size);
            let bit_range = start..start + self.element_bit_size;
            // TODO : START_HERE
            let zeros_on_left = bit_range.start % 8;
            Some(Resolver {
                data: Box::new(ResolverData {
                    bit_range,
                    zeros_on_left,
                    available_bits_in_first_byte: 8 - zeros_on_left,
                    starting_inject_byte: bit_range.start / 8,
                    // TODO the flip information may be incorrect. it might be rust size.
                    flip: if self.flip.is_some() {
                        Some(bit_range.end - bit_range.start)
                    } else {
                        None
                    },
                    field_name: ident,
                }),
                ty: Box::new(self.ty.clone().into()),
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
    pub ty: ResolverType,
    // Amount of remaining bits to consume.
    pub remaining_bits: usize,
    // Total amount of bytes the iterator will consume when `None` is the return of `self.next()`.
    pub total_elements: usize,
}

impl BlockArrayIter {
    // creates a new ElementArrayIter with `elements` array length.
    pub fn new(
        outer_ident: DynamicIdent,
        ty: ResolverType,
        starting_bit_index: usize,
        elements: usize,
        amount_of_bits: usize,
    ) -> Self {
        Self {
            outer_ident,
            starting_bit_index,
            ty,
            total_elements: elements,
            remaining_elements: elements,
            remaining_bits: amount_of_bits,
        }
    }

    pub fn from_values(
        resolver_data: &ResolverData,
        sub_ty: &ResolverSubType,
        array_ty: &ResolverArrayType,
        sizings: &ArraySizings,
    ) -> Self {
        let mut sizings = sizings.clone();
        let elements = sizings.pop();
        let ty = if sizings.is_empty() {
            sub_ty.clone()
        } else {
            ResolverType::Array {
                sub_ty: sub_ty.clone(),
                array_ty: array_ty.clone(),
                sizings,
            }
        };
        Ok(BlockArrayIter::new(
            resolver_data.field_name.clone(),
            ty,
            resolver_data.bit_range_start(),
            elements,
            resolver_data.amount_of_bits,
        ))
    }
}

impl Iterator for BlockArrayIter {
    type Item = Resolver;
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
            let name = format_ident!("{outer_ident}_{index}");
            let ident = (outer_ident, name).into();
            self.remaining_elements -= 1;
            let zeros_on_left = bit_range.start % 8;
            // Some(BuiltDataTypeInfo {
            //     name: ident,
            //     bit_range,
            //     ty: self.ty.clone(),
            // })
            Some(Resolver {
                data: Box::new(ResolverData {
                    bit_range,
                    zeros_on_left,
                    available_bits_in_first_byte: 8 - zeros_on_left,
                    starting_inject_byte: bit_range.start / 8,
                    // TODO the flip information may be incorrect. it might be rust size.
                    flip: if self.flip.is_some() {
                        Some(bit_range.end - bit_range.start)
                    } else {
                        None
                    },
                    field_name: ident,
                }),
                ty: Box::new(self.ty.clone().into()),
            });
            todo!("make iter logic for block array iter");
        } else {
            None
        }
    }
}
