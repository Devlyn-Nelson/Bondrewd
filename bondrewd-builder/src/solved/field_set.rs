use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    hash::Hash,
    ops::Range,
};

use thiserror::Error;

use crate::{
    build::{
        field::{DataType, RustByteSize},
        field_set::{EnumBuilder, FieldSetBuilder, GenericBuilder, StructEnforcement},
        BuilderRange, Endianness, OverlapOptions, ReserveFieldOption,
    },
    solved::field::Resolver,
};

use super::field::{DynamicIdent, SolvedData};

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
    #[error("Final bit count was not evenly divisible by 0.")]
    EnforceFullBytes,
    #[error("Final bit count does not match enforcement size.[user = {user}, actual = {actual}]")]
    EnforceBitCount { actual: usize, user: usize },
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
        let fields_ref = &value.fields;
        // First stage checks for validity
        for value_field in fields_ref {
            let rust_size = value_field.ty.rust_size();
            // get resolved range for the field.
            let bit_range = BuiltRange::from_builder(&value_field.bit_range, &rust_size, last_end_bit_index.clone());// get_range(&value_field.bit_range, &rust_size, last_end_bit_index);
            // update internal last_end_bit_index to allow automatic bit-range feature to work.
            if !value_field.overlap.is_redundant() {
                last_end_bit_index = Some(bit_range.end());
            }
            let field = BuiltData {
                endianness: if let Some(e) = &value_field.endianness {
                    e.clone()
                } else {
                    // TODO no endianess is actually valid in the case of nested structs/enums.
                    // We need to check if the value is a primitive number, then if it is a number and does
                    // not have endianess we can throw this error.
                    return Err(SolvingError::NoEndianness(format!("{}", value_field.id)));
                },
                id: value_field.id,
                ty: todo!("calculate new built type"),
                reserve: value_field.reserve.clone(),
                overlap: value_field.overlap.clone(),
            };
            for other in &pre_fields {
                if !field.overlap.enabled() && !other.overlap.enabled() {
                    // check that self's start is not within other's range
                    if field.ty.bit_range.start >= other.ty.bit_range.start
                        && (field.ty.bit_range.start == other.ty.bit_range.start
                            || field.ty.bit_range.start < other.ty.bit_range.end)
                    {
                        return Err(SolvingError::Overlap);
                    }
                    // check that other's start is not within self's range
                    if other.ty.bit_range.start >= field.ty.bit_range.start
                        && (other.ty.bit_range.start == field.ty.bit_range.start
                            || other.ty.bit_range.start < field.ty.bit_range.end)
                    {
                        return Err(SolvingError::Overlap);
                    }
                    if other.ty.bit_range.end > field.ty.bit_range.start
                        && other.ty.bit_range.end <= field.ty.bit_range.end
                    {
                        return Err(SolvingError::Overlap);
                    }
                    if field.ty.bit_range.end > other.ty.bit_range.start
                        && field.ty.bit_range.end <= other.ty.bit_range.end
                    {
                        return Err(SolvingError::Overlap);
                    }
                }
            }
            // let name = format!("{}", field.id);
            pre_fields.push(field);
        }
        let mut fields: HashMap<DataId, SolvedData> = HashMap::default();
        for mut pre_field in pre_fields {
            // Reverse field order
            if pre_field.endianness.is_field_order_reversed() {
                pre_field.ty.bit_range = (bit_size - pre_field.ty.bit_range.end)
                    ..(bit_size - pre_field.ty.bit_range.start);
            }
            // get the total number of bits the field uses.
            let amount_of_bits = pre_field.ty.bit_range.end - pre_field.ty.bit_range.start;
            // amount of zeros to have for the right mask. (right mask meaning a mask to keep data on the
            // left)
            let zeros_on_left = pre_field.ty.bit_range.start % 8;
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
            let mut starting_inject_byte: usize = pre_field.ty.bit_range.start / 8;
            // NOTE endianness is only for determining how to get the bytes we will apply to the output.
            // calculate how many of the bits will be inside the most significant byte we are adding to.
            if pre_field.endianness.is_byte_order_reversed() {
                let struct_byte_length = bit_size / 8;
                starting_inject_byte = struct_byte_length - starting_inject_byte;
            }

            // make a name for the buffer that we will store the number in byte form
            #[cfg(feature = "derive")]
            let field_buffer_name = format!("{}_bytes", pre_field.id);
            let ty = todo!("determine proper built type or error");
            let resolver = Resolver {
                amount_of_bits,
                zeros_on_left,
                available_bits_in_first_byte,
                starting_inject_byte,
                #[cfg(feature = "derive")]
                field_buffer_name,
                ty,
                reverse_byte_order: pre_field.endianness.is_byte_order_reversed(),
            };
            let new_field = SolvedData { resolver };
            fields.insert(pre_field.id, new_field);
        }
        let keys: Vec<DataId> = fields.keys().copied().collect();
        for key in keys {
            let field = fields.get(&key);
        }
        todo!("handle array solving");
        todo!("auto_fill");
        match value.enforcement {
            StructEnforcement::NoRules => {}
            StructEnforcement::EnforceFullBytes => {
                if bit_size % 8 != 0 {
                    return Err(SolvingError::EnforceFullBytes);
                }
            }
            StructEnforcement::EnforceBitAmount(expected_total_bits) => {
                if bit_size != expected_total_bits {
                    return Err(SolvingError::EnforceBitCount {
                        actual: bit_size,
                        user: expected_total_bits,
                    });
                }
            }
        }
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
#[derive(Clone, Debug)]
pub struct BuiltData<Id: Display + PartialEq> {
    /// The name or ident of the field.
    pub(crate) id: Id,
    pub(crate) ty: BuiltDataTypeInfo,
    pub(crate) endianness: Endianness,
    /// Describes when the field should be considered.
    pub(crate) reserve: ReserveFieldOption,
    /// How much you care about the field overlapping other fields.
    pub(crate) overlap: OverlapOptions,
}

#[derive(Clone, Debug)]
pub struct BuiltDataTypeInfo {
    pub(crate) ty: BuiltDataType,
    /// The range of bits that this field will use.
    pub(crate) bit_range: Range<usize>,
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
            BuiltDataType::Single(dt) => dt.rust_size().bytes(),
            BuiltDataType::BlockArray { elements, sub }
            | BuiltDataType::ElementArray { elements, sub } => {
                sub.sub.ty.rust_bytes_size() * elements
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BuiltDataSubType {
    sub: Box<BuiltDataTypeInfo>,
}

pub struct ElementArrayIter {
    pub outer_ident: DynamicIdent,
    /// a iter that returns the index of the element we are returning information for.
    pub element_range: Range<usize>,
    // the starting bit index of the first element
    pub starting_bit_index: usize,
    pub ty: BuiltDataType,
    pub element_bit_size: usize,
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
    pub bit_length: usize,
    // Total amount of bytes the iterator will consume when `None` is the return of `self.next()`.
    pub total_bytes: usize,
}

impl Iterator for BlockArrayIter {
    type Item = BuiltDataTypeInfo;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_elements != 0 {
            let mut ty_size = self.ty.rust_bytes_size() * 8;
            if self.bit_length % ty_size != 0 {
                ty_size = self.bit_length % ty_size;
            }
            let start = self.starting_bit_index;
            self.starting_bit_index = start + ty_size;
            let bit_range = start..(start + ty_size);
            self.bit_length -= ty_size;
            let index = self.total_bytes - self.remaining_elements;
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

pub enum BuiltRange {
    SingleElement(Range<usize>),
    BlockArray,   /*(BlockArrayIter)*/
    ElementArray, /*(ElementArrayIter)*/
}

impl BuiltRange {
    fn end(&self) -> usize {
        match self {
            BuiltRange::SingleElement(range) => range.end,
            BuiltRange::BlockArray => todo!("figure out the ending index"),
            BuiltRange::ElementArray => todo!("figure out the ending index"),
        }
    }
    fn from_builder(
        builder: &BuilderRange,
        rust_size: &RustByteSize,
        last_field_end: Option<usize>,
    ) -> Self {
        // TODO START_HERE move `get_range` function below into here.
        match builder {
            BuilderRange::Range(range) => Self::SingleElement(range.clone()),
            BuilderRange::Size(bit_length) => {
                let start = if let Some(prev) = &last_field_end {
                    *prev
                } else {
                    0
                };
                Self::SingleElement(start..(start + *bit_length as usize))
            }
            BuilderRange::None => {
                let start = if let Some(prev) = &last_field_end {
                    *prev
                } else {
                    0
                };
                Self::SingleElement(start..(start + (*rust_size as usize * 8)))
            }
            BuilderRange::ElementArray {
                sizings,
                element_bit_length,
            } => todo!("make these calcs"),
            BuilderRange::BlockArray {
                sizings,
                total_bits,
            } => todo!("make these calcs"),
        }
    }
}

// `field` should be the field we want to get a `bit_range` for.
// `last_field_end` should be the ending bit of the previous field processed.
// fn get_range(
//     bit_range: &BuilderRange,
//     rust_size: &RustByteSize,
//     last_field_end: Option<usize>,
// ) -> Range<usize> {
//     match bit_range {
//         BuilderRange::Range(range) => range.clone(),
//         BuilderRange::Size(bit_length) => {
//             let start = if let Some(prev) = &last_field_end {
//                 *prev
//             } else {
//                 0
//             };
//             start..(start + *bit_length as usize)
//         }
//         BuilderRange::None => {
//             let start = if let Some(prev) = &last_field_end {
//                 *prev
//             } else {
//                 0
//             };
//             start..(start + (*rust_size as usize * 8))
//         }
//         BuilderRange::ElementArray {
//             sizings,
//             element_bit_length,
//         } => todo!("make these calcs"),
//         BuilderRange::BlockArray {
//             sizings,
//             total_bits,
//         } => todo!("make these calcs"),
//     }
// }
