use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    hash::Hash,
    ops::Range,
};

#[cfg(feature = "derive")]
use quote::format_ident;
use thiserror::Error;

use crate::{
    build::{
        field::{ArrayInfo, DataBuilder, DataType},
        field_set::{EnumBuilder, FieldSetBuilder, GenericBuilder},
        Endianness, OverlapOptions, ReserveFieldOption,
    },
    solved::field::Resolver,
};

use super::field::SolvedData;

pub struct Solved<FieldSetId, DataId>
where
    FieldSetId: Display + Clone + Copy,
{
    /// DataSet's name.
    ///
    /// for derive this would be the Enum or Struct ident.
    #[cfg(feature = "derive")]
    name: FieldSetId,
    ty: SolvedType<FieldSetId, DataId>,
}
enum SolvedType<FieldSetId, DataId> {
    Enum {
        /// The id field. or the field that determines the variant.
        id: SolvedData,
        /// The default variant. in the case not other variant matches, this will be used.
        invalid: SolvedFieldSet<DataId>,
        /// The default variant's name/ident
        invalid_name: VariantInfo<FieldSetId>,
        /// Sets of fields, each representing a variant of an enum. the String
        /// being the name of the variant
        variants: BTreeMap<VariantInfo<FieldSetId>, SolvedFieldSet<DataId>>,
    },
    Struct(SolvedFieldSet<DataId>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VariantInfo<FieldSetId> {
    id: i64,
    name: FieldSetId,
}

struct SolvedFieldSet<DataId> {
    fields: HashMap<DataId, SolvedData>,
}

#[derive(Debug, Error)]
pub enum SolvingError {
    /// Fields overlaps
    #[error("Fields overlap")]
    Overlap,
    /// Tried to solve a field without a type.
    ///
    /// # Field
    /// the `String` provided should be the id or name of the field.
    #[error("No data type was provided for field with id {0}")]
    NoTypeProvided(String),
    /// Tried to solve a number field without endianness.
    ///
    /// # Field
    /// the `String` provided should be the id or name of the field.
    #[error("No endianness was provided for field with id {0}")]
    NoEndianness(String),
    /// [`Resolver::new`] had a left shift underflow.
    ///
    /// # Field
    /// the `String` provided should be the id or name of the field.
    #[error("Failed solving the `left_shift` due to underflow.")]
    ResolverUnderflow(String),
}

impl<FieldSetId, DataId> TryFrom<GenericBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId>
where
    FieldSetId: Display + Clone + Copy,
    DataId: Hash + PartialEq + Eq + Display + Clone + Copy,
{
    type Error = SolvingError;

    fn try_from(value: GenericBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        match value.ty {
            crate::build::field_set::BuilderType::Enum(e) => e.try_into(),
            crate::build::field_set::BuilderType::Struct(s) => s.try_into(),
        }
    }
}

impl<FieldSetId, DataId> TryFrom<EnumBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId>
where
    FieldSetId: Display + Clone + Copy,
    DataId: Hash + PartialEq + Eq + Display + Clone + Copy,
{
    type Error = SolvingError;

    fn try_from(value: EnumBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        let id = value.id;
        let variants = value.variants;
        let invalid = value.invalid;
        #[cfg(feature = "derive")]
        let name = value.name;
        todo!("write conversion from EnumBuilder to Solved")
    }
}

impl<FieldSetId, DataId> TryFrom<FieldSetBuilder<FieldSetId, DataId>> for Solved<FieldSetId, DataId>
where
    FieldSetId: Display + Clone + Copy,
    DataId: Hash + PartialEq + Eq + Display + Clone + Copy,
{
    type Error = SolvingError;

    fn try_from(value: FieldSetBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        Self::try_from_field_set(&value, None)
    }
}

impl<FieldSetId, DataId> TryFrom<&FieldSetBuilder<FieldSetId, DataId>>
    for Solved<FieldSetId, DataId>
where
    FieldSetId: Display + Clone + Copy,
    DataId: Hash + PartialEq + Eq + Display + Clone + Copy,
{
    type Error = SolvingError;

    fn try_from(value: &FieldSetBuilder<FieldSetId, DataId>) -> Result<Self, Self::Error> {
        Self::try_from_field_set(value, None)
    }
}

impl<FieldSetId, DataId> Solved<FieldSetId, DataId>
where
    FieldSetId: Display + Clone + Copy,
    DataId: Hash + PartialEq + Eq + Display + Clone + Copy,
{
    fn try_from_field_set(
        value: &FieldSetBuilder<FieldSetId, DataId>,
        id_field: Option<&SolvedData>,
    ) -> Result<Self, SolvingError> {
        let bit_size = if let Some(id_field) = id_field {
            id_field.bit_length()
        } else {
            0
        };
        let mut pre_fields: Vec<BuiltData<DataId>> = Vec::default();
        let mut last_end_bit_index: Option<usize> = None;
        let total_fields = value.fields.len();
        // START_HERE i think i should make an in-between structure that holds "Solved" fields that have not
        // undergone any checks or math to reduce it to an actual Solved struct.
        //
        // First stage checks for validity
        for value_field in &value.fields {
            // get resolved range for the field.
            let bit_range = get_range(value_field, last_end_bit_index);
            // update internal last_end_bit_index to allow automatic bit-range feature to work.
            if !value_field.overlap.is_redundant() {
                last_end_bit_index = Some(bit_range.end);
            }
            let field = BuiltData {
                id: value_field.id,
                ty: value_field.ty.clone(),
                endianness: if let Some(e) = &value_field.endianness {
                    e.clone()
                } else {
                    return Err(SolvingError::NoEndianness(format!("{}", value_field.id)));
                },
                rust_size: value_field.rust_size,
                array: value_field.array.clone(),
                bit_range,
                reserve: value_field.reserve.clone(),
                overlap: value_field.overlap.clone(),
            };
            for other in &pre_fields {
                if !field.overlap.enabled() && !other.overlap.enabled() {
                    // check that self's start is not within other's range
                    if field.bit_range.start >= other.bit_range.start
                        && (field.bit_range.start == other.bit_range.start
                            || field.bit_range.start < other.bit_range.end)
                    {
                        return Err(SolvingError::Overlap);
                    }
                    // check that other's start is not within self's range
                    if other.bit_range.start >= field.bit_range.start
                        && (other.bit_range.start == field.bit_range.start
                            || other.bit_range.start < field.bit_range.end)
                    {
                        return Err(SolvingError::Overlap);
                    }
                    if field.bit_range.end > other.bit_range.start
                        && field.bit_range.end <= other.bit_range.end
                    {
                        return Err(SolvingError::Overlap);
                    }
                    if other.bit_range.end > field.bit_range.start
                        && other.bit_range.end <= field.bit_range.end
                    {
                        return Err(SolvingError::Overlap);
                    }
                }
            }
            // let name = format!("{}", field.id);
            pre_fields.push(field);
        }
        let mut fields: HashMap<DataId, SolvedData> = HashMap::default();
        for pre_field in pre_fields {
            // get the total number of bits the field uses.
            let amount_of_bits = pre_field.bit_range.end - pre_field.bit_range.start;
            // amount of zeros to have for the right mask. (right mask meaning a mask to keep data on the
            // left)
            let zeros_on_left = pre_field.bit_range.start % 8;
            // TODO if don't think this error is possible, and im wondering why it is being checked for
            // in the first place.
            if 7 < zeros_on_left {
                return Err(SolvingError::ResolverUnderflow(format!(
                    "field \"{}\" would have had left shift underflow, report this at \
                        https://github.com/Devlyn-Nelson/Bondrewd",
                    pre_field.id,
                )));
            }
            let available_bits_in_first_byte = 8 - zeros_on_left;
            // calculate the starting byte index in the outgoing buffer
            let mut starting_inject_byte: usize = pre_field.bit_range.start / 8;
            // NOTE endianness is only for determining how to get the bytes we will apply to the output.
            // calculate how many of the bits will be inside the most significant byte we are adding to.
            if pre_field.endianness.is_byte_order_reversed() {
                let struct_byte_length = bit_size / 8;
                starting_inject_byte = struct_byte_length - starting_inject_byte;
            }

            // make a name for the buffer that we will store the number in byte form
            #[cfg(feature = "derive")]
            let field_buffer_name = format_ident!("{}_bytes", pre_field.i);

            let ty = if pre_field.endianness.is_alternative() {
                // Alt endian logic (default is little packed).

                todo!("refer to else branch.");
            } else {
                // Standard endian logic (default is big).
                todo!(
                    "Figure out how to create the resolver type, need to check if the field spans \
                across multiple fields or not and what endianness mode it is."
                )
            };
            let resolver = Resolver {
                amount_of_bits,
                zeros_on_left,
                available_bits_in_first_byte,
                starting_inject_byte,
                #[cfg(feature = "derive")]
                field_buffer_name,
                ty,
            };
            let new_field = SolvedData { resolver };
            fields.insert(pre_field.id, new_field);
        }
        todo!("solve for field order reversal, might do it in loop after `last_end_bit_index` is set.");
        let keys: Vec<DataId> = fields.keys().copied().collect();
        for key in keys {
            let field = fields.get(&key);
        }
        todo!("handle array solving");
        todo!("enforcements.");
        Ok(Self {
            #[cfg(feature = "derive")]
            name: value.name,
            ty: SolvedType::Struct(SolvedFieldSet { fields }),
        })
    }
}
/// This is going to house all of the information for a Field. This acts as the stage between Builder and
/// Solved, the point being that this can not be created unless a valid `BuilderData` that can be solved is
/// provided. Then we can do all of the calculation because everything has been determined as solvable.
pub(crate) struct BuiltData<Id: Display + PartialEq> {
    /// The name or ident of the field.
    pub(crate) id: Id,
    /// The approximate data type of the field. when solving, this must be
    /// filled.
    pub(crate) ty: DataType,
    pub(crate) endianness: Endianness,
    /// Size of the rust native type in bytes (should never be zero)
    pub(crate) rust_size: u8,
    /// Defines if this field is an array or not.
    /// If `None` this data is not in an array and should just be treated as a single value.
    ///
    /// If `Some` than this is an array, NOT a single value. Also Note that the `ty` and `rust_size` only
    /// describe a true data type, which would be the innermost part of an array. The array info
    /// is marly keeping track of the order and magnitude of the array and its dimensions.
    pub(crate) array: Option<ArrayInfo>,
    /// The range of bits that this field will use.
    pub(crate) bit_range: Range<usize>,
    /// Describes when the field should be considered.
    pub(crate) reserve: ReserveFieldOption,
    /// How much you care about the field overlapping other fields.
    pub(crate) overlap: OverlapOptions,
}

/// `field` should be the field we want to get a `bit_range` for.
/// `last_field_end` should be the ending bit of the previous field processed.
fn get_range<Id>(field: &DataBuilder<Id>, last_field_end: Option<usize>) -> Range<usize>
where
    Id: Clone + Copy,
{
    match &field.bit_range {
        crate::build::BuilderRange::Range(range) => range.clone(),
        crate::build::BuilderRange::Size(bit_length) => {
            let start = if let Some(prev) = &last_field_end {
                *prev
            } else {
                0
            };
            start..(start + *bit_length as usize)
        }
        crate::build::BuilderRange::None => {
            let start = if let Some(prev) = &last_field_end {
                *prev
            } else {
                0
            };
            start..(start + (field.rust_size as usize * 8))
        }
    }
}
