use std::{collections::BTreeMap, env::current_dir, ops::Range};

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;
use thiserror::Error;

use crate::build::{
    field::{DataType, NumberType, RustByteSize},
    field_set::{
        EnumBuilder, FieldSetBuilder, FillBits, GenericBuilder, StructBuilder, StructEnforcement,
    },
    ArraySizings, BuilderRange, BuilderRangeArraySize, Endianness, OverlapOptions,
    ReserveFieldOption, Visibility,
};

use super::field::{DynamicIdent, SolvedData};

pub struct Solved {
    /// `DataSet`'s name.
    ///
    /// for derive this would be the Enum or Struct ident.
    pub(crate) name: Ident,
    pub(crate) ty: SolvedType,
}
pub enum SolvedType {
    Enum {
        /// The id field. or the field that determines the variant.
        id: SolvedData,
        /// The default variant. in the case not other variant matches, this will be used.
        invalid: SolvedFieldSet,
        /// The default variant's name/ident
        invalid_name: VariantInfo,
        /// Sets of fields, each representing a variant of an enum. the String
        /// being the name of the variant
        variants: BTreeMap<VariantInfo, SolvedFieldSet>,
    },
    Struct(SolvedFieldSet),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantInfo {
    pub(crate) id: i64,
    pub(crate) name: Ident,
}

pub struct SolvedFieldSet {
    pub(crate) attrs: SolvedFieldSetAttributes,
    pub(crate) fields: Vec<SolvedData>,
}

#[derive(Debug, Clone)]
pub struct SolvedFieldSetAttributes {
    pub dump: bool,
    pub vis: Visibility,
}

impl Default for SolvedFieldSetAttributes {
    fn default() -> Self {
        Self {
            vis: Visibility(syn::Visibility::Public(syn::token::Pub {
                span: proc_macro2::Span::call_site(),
            })),
            dump: false,
        }
    }
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
    #[error("A feature you are trying to use in not yet supported. complain to the Bondrewd maintainer about \"{0}\"")]
    Unfinished(String),
}

impl From<SolvingError> for syn::Error {
    fn from(value: SolvingError) -> Self {
        syn::Error::new(Span::call_site(), format!("{value}"))
    }
}

impl TryFrom<GenericBuilder> for Solved {
    type Error = SolvingError;

    fn try_from(value: GenericBuilder) -> Result<Self, Self::Error> {
        match value.ty {
            crate::build::field_set::BuilderType::Enum(e) => (*e).try_into(),
            crate::build::field_set::BuilderType::Struct(s) => (*s).try_into(),
        }
    }
}

impl TryFrom<EnumBuilder> for Solved {
    type Error = SolvingError;

    fn try_from(value: EnumBuilder) -> Result<Self, Self::Error> {
        let id = value.id;
        let variants = value.variants;
        let invalid = value.invalid;
        let name = value.name;
        // START_HERE enum solving need to be written.
        // // give all variants ids.
        // // TODO this code was in hte parsing code, but has been moved here.
        // let mut used_ids: Vec<usize> = (&invalid_variant.id).map(|f| vec![f]).unwrap_or_default();
        // let mut last = 0;
        // for (i, variant) in variants.iter_mut().enumerate() {
        //     if let Some(ref value) = variant.id {
        //         if used_ids.contains(value) {
        //             return Err(Error::new(
        //                 variant.field_set.name.span(),
        //                 "variant identifier used twice.",
        //             ));
        //         }
        //         last = *value;
        //         // push used index.
        //         used_ids.push(*value);
        //     } else {
        //         let mut guess = last + 1;
        //         while used_ids.contains(&guess) {
        //             guess += 1;
        //         }
        //         variant.id = Some(guess);
        //         used_ids.push(guess);
        //     }
        // }
        // // validity checks
        // if largest_variant_id > id_field_bit_length_max_value {
        //     return Err(Error::new(
        //         data.enum_token.span(),
        //         "the bit size being used is less than required to describe each variant"
        //             .to_string(),
        //     ));
        // }
        // if enum_attrs.payload_bit_size + enum_attrs.id_bits < largest {
        //     return Err(Error::new(
        //         data.enum_token.span(),
        //         "the payload size being used is less than largest variant".to_string(),
        //     ));
        // }
        // // verify the size doesn't go over set size.
        // for variant in &variants {
        //     let size = variant.total_bits();
        //     if largest < size {
        //         largest = size;
        //     }
        //     let variant_id_field = {
        //         if let Some(id) = variant.get_id_field()? {
        //             id
        //         } else {
        //             return Err(syn::Error::new(variant.name.span(), "failed to get variant field for variant. (this is a bondrewd issue, please report issue)"));
        //         }
        //     };

        //     if let Some(bit_size) = enum_attrs.payload_bit_size {
        //         if bit_size < size - variant_id_field.bit_size() {
        //             return Err(Error::new(
        //                         variant.name.span(),
        //                         format!("variant is larger than defined payload_size of enum. defined size: {bit_size}. variant size: {}", size- variant_id_field.bit_size()),
        //                     ));
        //         }
        //     } else if let (Some(bit_size), Some(id_size)) =
        //         (enum_attrs.total_bit_size, enum_attrs.id_bits)
        //     {
        //         if bit_size - id_size < size - variant_id_field.bit_size() {
        //             return Err(Error::new(
        //                         variant.name.span(),
        //                         format!("variant with id is larger than defined total_size of enum. defined size: {}. calculated size: {}", bit_size - id_size, size - variant_id_field.bit_size()),
        //                     ));
        //         }
        //     }
        // }
        // // add fill_bits if needed.
        // // TODO fix fill byte getting inserted of wrong side sometimes.
        // // the problem is, things get calculated before fill is added. also fill might be getting added when it shouldn't.
        // for v in &mut variants {
        //     let first_bit = v.total_bits();
        //     if first_bit < largest {
        //         let fill_bytes_size = (largest - first_bit).div_ceil(8);
        //         let ident = quote::format_ident!("enum_fill_bits");
        //         let fill = FieldInfo {
        //             ident: Box::new(ident.into()),
        //             attrs: Attributes {
        //                 bit_range: first_bit..largest,
        //                 endianness: Box::new(Endianness::big()),
        //                 reserve: ReserveFieldOption::FakeField,
        //                 overlap: OverlapOptions::None,
        //                 capture_id: false,
        //             },
        //             ty: DataType::BlockArray {
        //                 sub_type: Box::new(SubFieldInfo {
        //                     ty: DataType::Number {
        //                         size: 1,
        //                         sign: NumberSignage::Unsigned,
        //                         type_quote: quote! {u8},
        //                     },
        //                 }),
        //                 length: fill_bytes_size,
        //                 type_quote: quote! {[u8;#fill_bytes_size]},
        //             },
        //         };
        //         if v.attrs.default_endianess.is_byte_order_reversed() {
        //             v.fields.insert(0, fill);
        //         } else {
        //             v.fields.push(fill);
        //         }
        //     }
        // }
        // // TODO make the id_field of truth. or the one that bondrewd actually reads NOT captured id_fields.
        // // the truth Id_field should also match the capture_id fields that do exist. use below code for this.
        // let (id_field_type, id_bits) = {
        //     let id_bits = if let Some(id_bits) = enum_attrs.id_bit_length {
        //         id_bits
        //     } else if let (Some(payload_size), StructEnforcement::EnforceBitAmount(total_size)) =
        //         (&enum_attrs.payload_bit_length, &struct_attrs.enforcement)
        //     {
        //         total_size - payload_size
        //     } else {
        //         let mut x = data_enum.variants.len();
        //         // find minimal id size from largest id value
        //         let mut n = 0;
        //         while x != 0 {
        //             x >>= 1;
        //             n += 1;
        //         }
        //         n
        //     };
        //     let bytes = match id_bits.div_ceil(8) {
        //         1 => RustByteSize::One,
        //         2 => RustByteSize::Two,
        //         3..=4 => RustByteSize::Four,
        //         5..=8 => RustByteSize::Eight,
        //         9..=16 => RustByteSize::Sixteen,
        //         invalid => return Err(syn::Error::new(
        //             data_enum.enum_token.span(),
        //             format!("The variant is must have a type of: u8, u16, u32, u64, or u128, variant bit length is currently {invalid} and bondrewd doesn't know which type use."),
        //         )),
        //     };
        //     (DataType::Number(NumberType::Unsigned, bytes), id_bits)
        // };
        // let id_field = DataBuilder {
        //     ty: id_field_type,
        //     id: format_ident!("{}", EnumBuilder::VARIANT_ID_NAME).into(),
        //     endianness: Some(struct_attrs.default_endianness.clone()),
        //     bit_range: super::BuilderRange::Range(0..id_bits),
        //     reserve: ReserveFieldOption::FakeField,
        //     overlap: OverlapOptions::None,
        //     is_captured_id: false,
        // };
        todo!("write conversion from EnumBuilder to Solved")
    }
}

impl TryFrom<StructBuilder> for Solved {
    type Error = SolvingError;

    fn try_from(value: StructBuilder) -> Result<Self, Self::Error> {
        Self::try_from_field_set(&value.field_set, &value.attrs, None)
    }
}

impl TryFrom<&StructBuilder> for Solved {
    type Error = SolvingError;

    fn try_from(value: &StructBuilder) -> Result<Self, Self::Error> {
        Self::try_from_field_set(&value.field_set, &value.attrs, None)
    }
}

impl Solved {
    pub fn total_bits_no_fill(&self) -> usize {
        match &self.ty {
            SolvedType::Enum {
                id,
                invalid,
                invalid_name,
                variants,
            } => {
                let mut largest = 0;
                for var in variants {
                    let other = var.1.total_bits_no_fill();
                    if other > largest {
                        largest = other;
                    }
                }
                largest
            }
            SolvedType::Struct(solved_field_set) => solved_field_set.total_bits_no_fill(),
        }
    }
    pub fn total_bytes_used(&self) -> usize {
        match &self.ty {
            SolvedType::Enum {
                id,
                invalid,
                invalid_name,
                variants,
            } => {
                let mut largest = 0;
                for var in variants {
                    let other = var.1.total_bytes();
                    if other > largest {
                        largest = other;
                    }
                }
                largest
            }
            SolvedType::Struct(solved_field_set) => solved_field_set.total_bytes(),
        }
    }
    pub fn total_bits_used(&self) -> usize {
        match &self.ty {
            SolvedType::Enum {
                id,
                invalid,
                invalid_name,
                variants,
            } => {
                let mut largest = 0;
                for var in variants {
                    let other = var.1.total_bits();
                    if other > largest {
                        largest = other;
                    }
                }
                largest
            }
            SolvedType::Struct(solved_field_set) => solved_field_set.total_bits(),
        }
    }
    pub fn gen(&self, dyn_fns: bool, hex_fns: bool, setters: bool) -> syn::Result<TokenStream> {
        let struct_name = &self.name;
        let (gen, struct_size) = match &self.ty {
            SolvedType::Enum {
                id,
                invalid,
                invalid_name,
                variants,
            } => todo!("generate enum quotes"),
            SolvedType::Struct(solved_field_set) => {
                let struct_size = self.total_bytes_used();
                (
                    solved_field_set
                        .generate_quotes(struct_name, None, struct_size, true)?
                        .finish(),
                    struct_size,
                )
            }
        };
        // get the struct size and name so we can use them in a quote.
        let impl_fns = gen.non_trait;
        let mut output = match self.ty {
            SolvedType::Struct(..) => {
                if setters {
                    return Err(syn::Error::new(
                        self.name.span(),
                        "Setters are currently unsupported",
                    ));
                    // TODO get setter for arrays working.
                    // get the setters, functions that set a field disallowing numbers
                    // outside of the range the Bitfield.
                    // let setters_quote = match struct_fns::create_setters_quotes(struct_info) {
                    //     Ok(parsed_struct) => parsed_struct,
                    //     Err(err) => {
                    //         return Err(err);
                    //     }
                    // };
                    // quote! {
                    //     impl #struct_name {
                    //         #impl_fns
                    //         #setters_quote
                    //     }
                    // }
                } else {
                    quote! {
                        impl #struct_name {
                            #impl_fns
                        }
                    }
                }
            }
            SolvedType::Enum { .. } => {
                // TODO implement getters and setters for enums.
                quote! {
                    impl #struct_name {
                        #impl_fns
                    }
                }
            }
        };
        // get the bit size of the entire set of fields to fill in trait requirement.
        let bit_size = self.total_bits_no_fill();
        let trait_impl_fn = gen.bitfield_trait;
        output = quote! {
            #output
            impl bondrewd::Bitfields<#struct_size> for #struct_name {
                const BIT_SIZE: usize = #bit_size;
                #trait_impl_fn
            }
        };
        if hex_fns {
            let hex_size = struct_size * 2;
            output = quote! {
                #output
                impl bondrewd::BitfieldHex<#hex_size, #struct_size> for #struct_name {}
            };
            if dyn_fns {
                output = quote! {
                    #output
                    impl bondrewd::BitfieldHexDyn<#hex_size, #struct_size> for #struct_name {}
                };
            }
        }
        if let Some(dyn_fns) = gen.dyn_fns {
            let checked_structs = dyn_fns.checked_struct;
            let from_vec_quote = dyn_fns.bitfield_dyn_trait;
            output = quote! {
                #output
                #checked_structs
                impl bondrewd::BitfieldsDyn<#struct_size> for #struct_name {
                    #from_vec_quote
                }
            }
        }
        if self.dump() {
            let name = self.name.to_string().to_case(Case::Snake);
            match current_dir() {
                Ok(mut file_name) => {
                    file_name.push("target");
                    file_name.push(format!("{name}_code_gen.rs"));
                    let _ = std::fs::write(file_name, output.to_string());
                }
                Err(err) => {
                    return Err(syn::Error::new(self.name.span(), format!("Failed to dump code gen because target folder could not be located. remove `dump` from struct or enum bondrewd attributes. [{err}]")));
                }
            }
        }
        Ok(output)
    }
    fn dump(&self) -> bool {
        match &self.ty {
            SolvedType::Enum {
                id,
                invalid,
                invalid_name,
                variants,
            } => {
                // TODO impl dump for enums.
                false
            }
            SolvedType::Struct(solved_field_set) => solved_field_set.attrs.dump,
        }
    }
    fn try_from_field_set(
        value: &FieldSetBuilder,
        attrs: &SolvedFieldSetAttributes,
        id_field: Option<&BuiltData>,
    ) -> Result<Self, SolvingError> {
        // TODO solve arrays.
        //
        // let bit_size = if let Some(id_field) = id_field {
        //     id_field.bit_length()
        // } else {
        //     0
        // };
        //
        // TODO verify captured id fields match the generated id field using below commented code.
        // if let Some(bondrewd_field) = id_field {
        //     if i == 0 {
        //         match (&bondrewd_field.ty, &mut parsed_field.ty) {
        //             (
        //                 DataType::Number(number_ty_bon, rust_size_bon),
        //                 DataType::Number(number_ty, rust_size)
        //             ) => {
        //                 let ty_ident = get_number_type_ident(number_ty, rust_size.bits());
        //                 let ty_ident_bon = get_number_type_ident(number_ty_bon, rust_size_bon.bits());
        //                 // TODO this if statements actions could cause confusing behavior
        //                 if bondrewd_field.bit_range != parsed_field.bit_range {
        //                     parsed_field.bit_range = bondrewd_field.bit_range.clone();
        //                 }
        //                 if number_ty_bon != number_ty {
        //                     return Err(Error::new(field.span(), format!("`capture_id` field must be unsigned. bondrewd will enforce the type as {ty_ident_bon}")));
        //                 }else if ty_ident_bon != ty_ident {
        //                     return Err(Error::new(field.span(), format!("`capture_id` field currently must be {ty_ident_bon} in this instance, because bondrewd makes an assumption about the id type. changing this would be difficult")));
        //                 }
        //             }
        //             (DataType::Number(number_ty_bon, rust_size_bon), _) => {
        //                 let ty_ident_bon = get_number_type_ident(number_ty_bon, rust_size_bon.bits());
        //                 return Err(Error::new(field.span(), format!("capture_id field must be an unsigned number. detected type is {ty_ident_bon}.")))
        //             }
        //             _ => return Err(Error::new(field.span(), "an error with bondrewd has occurred, the id field should be a number but bondrewd did not use a number for the id.")),
        //         }
        //     } else {
        //         return Err(Error::new(
        //             field.span(),
        //             "`capture_id` attribute must be the first field.",
        //         ));
        //     }
        // } else {
        //     return Err(Error::new(
        //         field.span(),
        //         "`capture_id` attribute is intended for enum variants only.",
        //     ));
        // }
        let mut pre_fields: Vec<BuiltData> = Vec::default();
        let mut last_end_bit_index: Option<usize> = id_field.map(|f| f.bit_range.bit_length());
        let total_fields = value.fields.len();
        let fields_ref = &value.fields;
        // First stage checks for validity
        for value_field in fields_ref {
            let default_bit_size = value_field.ty.default_bit_size();
            // get resolved range for the field.
            let bit_range = BuiltRange::from_builder(
                &value_field.bit_range,
                default_bit_size,
                last_end_bit_index,
            );
            // get_range(&value_field.bit_range, &rust_size, last_end_bit_index);
            // update internal last_end_bit_index to allow automatic bit-range feature to work.
            if !value_field.overlap.is_redundant() {
                last_end_bit_index = Some(bit_range.end());
            }
            let ty = value_field.ty.clone();
            let field = BuiltData {
                ty,
                bit_range,
                endianness: if let Some(e) = &value_field.endianness {
                    e.clone()
                } else {
                    // TODO no endianess is actually valid in the case of nested structs/enums.
                    // We need to check if the value is a primitive number, then if it is a number and does
                    // not have endianess we can throw this error.
                    return Err(SolvingError::NoEndianness(format!(
                        "{}",
                        value_field.id.ident()
                    )));
                },
                id: value_field.id.clone(),
                reserve: value_field.reserve.clone(),
                overlap: value_field.overlap.clone(),
                is_captured_id: value_field.is_captured_id,
            };
            let field_range = field.bit_range.range();
            for other in &pre_fields {
                if field.conflict(other) {
                    return Err(SolvingError::Overlap);
                }
            }
            // let name = format!("{}", field.id);
            pre_fields.push(field);
        }
        let mut fields: Vec<SolvedData> = Vec::default();
        for pre_field in pre_fields {
            if let Some(field) = id_field {
                if field.conflict(&pre_field) {
                    return Err(SolvingError::Overlap);
                }
            }
            fields.push(SolvedData::from(pre_field));
        }
        let mut out = SolvedFieldSet {
            fields,
            attrs: attrs.clone(),
        };
        // let keys: Vec<DynamicIdent> = fields.keys().cloned().collect();
        // for key in keys {
        //     let field = fields.get(&key);
        // }
        let bit_size = out.total_bits();
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
        let first_bit = if let Some(last_range) = out.fields.iter().last() {
            last_range.bit_range().end
        } else {
            0_usize
        };
        let auto_fill = match value.fill_bits {
            FillBits::None => None,
            FillBits::Bits(bits) => Some(bits),
            FillBits::Auto => {
                if id_field.is_some() {
                    return Err(SolvingError::Unfinished(
                        "Auto fill_bits is currently not finished for enums.".to_string(),
                    ));
                }
                let unused_bits = bit_size % 8;
                if unused_bits == 0 {
                    None
                } else {
                    Some(8 - unused_bits)
                    // None
                }
            }
        };
        // add reserve for fill bytes. this happens after bit enforcement because bit_enforcement is for checking user code.
        if let Some(fill_bits) = auto_fill {
            let end_bit = first_bit + fill_bits;
            // bit_size += fill_bits;
            let fill_bytes_size = (end_bit - first_bit).div_ceil(8);
            let ident = quote::format_ident!("bondrewd_fill_bits");
            let endian = value.default_endianness.clone();
            // fields.push(FieldInfo {
            //     ident: Box::new(ident.into()),
            //     attrs: Attributes {
            //         bit_range: first_bit..end_bit,
            //         endianness: Box::new(endian),
            //         reserve: ReserveFieldOption::FakeField,
            //         overlap: OverlapOptions::None,
            //         capture_id: false,
            //     },
            //     ty: DataType::BlockArray {
            //         sub_type: Box::new(SubFieldInfo {
            //             ty: DataType::Number {
            //                 size: 1,
            //                 sign: NumberSignage::Unsigned,
            //                 type_quote: quote! {u8},
            //             },
            //         }),
            //         length: fill_bytes_size,
            //         type_quote: quote! {[u8;#fill_bytes_size]},
            //     },
            // });
            let fill_field = BuiltData {
                id: ident.into(),
                ty: DataType::Number(NumberType::Unsigned, RustByteSize::One),
                bit_range: BuiltRange {
                    bit_range: first_bit..end_bit,
                    ty: BuiltRangeType::BlockArray(vec![fill_bytes_size]),
                },
                endianness: endian,
                reserve: ReserveFieldOption::FakeField,
                overlap: OverlapOptions::None,
                is_captured_id: false,
            };
            out.fields.push(fill_field.into());
        }
        Ok(Self {
            name: value.name.clone(),
            ty: SolvedType::Struct(out),
        })
    }
}
/// This is going to house all of the information for a Field. This acts as the stage between Builder and
/// Solved, the point being that this can not be created unless a valid `BuilderData` that can be solved is
/// provided. Then we can do all of the calculation because everything has been determined as solvable.
#[derive(Clone, Debug)]
pub struct BuiltData {
    /// The name or ident of the field.
    pub(crate) id: DynamicIdent,
    pub(crate) ty: DataType,
    pub(crate) bit_range: BuiltRange,
    pub(crate) endianness: Endianness,
    /// Describes when the field should be considered.
    pub(crate) reserve: ReserveFieldOption,
    /// How much you care about the field overlapping other fields.
    pub(crate) overlap: OverlapOptions,
    pub(crate) is_captured_id: bool,
}

impl BuiltData {
    pub fn conflict(&self, other: &Self) -> bool {
        if self.reserve.is_fake_field() {
            return false;
        }
        if !self.overlap.enabled() && !other.overlap.enabled() {
            let field_range = self.bit_range.range();
            let other_range = other.bit_range.range();
            // check that self's start is not within other's range
            if field_range.start >= other_range.start
                && (field_range.start == other_range.start || field_range.start < other_range.end)
            {
                return true;
            }
            // check that other's start is not within self's range
            if other_range.start >= field_range.start
                && (other_range.start == field_range.start || other_range.start < field_range.end)
            {
                return true;
            }
            if other_range.end > field_range.start && other_range.end <= field_range.end {
                return true;
            }
            if field_range.end > other_range.start && field_range.end <= other_range.end {
                return true;
            }
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct BuiltRange {
    /// This is the full bit range of the field. The `ty` does not effect this values meaning, `bit_range`
    /// shall contain the entire bit range for a field, array elements should each calculate their own
    /// range within this range.
    pub(crate) bit_range: Range<usize>,
    /// The range type determines if the `bit_range` contains a single value or contains a array of values.
    pub(crate) ty: BuiltRangeType,
}

impl BuiltRange {
    #[must_use]
    pub fn range(&self) -> &Range<usize> {
        &self.bit_range
    }
    #[must_use]
    pub fn bit_length(&self) -> usize {
        self.bit_range.end - self.bit_range.start
    }
}

#[derive(Clone, Debug)]
pub enum BuiltRangeType {
    SingleElement,
    BlockArray(ArraySizings),
    ElementArray(ArraySizings),
}

impl BuiltRange {
    fn end(&self) -> usize {
        self.bit_range.end
    }
    fn from_builder(
        builder: &BuilderRange,
        default_bit_length: usize,
        last_field_end: Option<usize>,
    ) -> Self {
        let start = if let Some(prev) = &last_field_end {
            *prev
        } else {
            0
        };
        match builder {
            BuilderRange::Range(bit_range) => Self {
                bit_range: bit_range.clone(),
                ty: BuiltRangeType::SingleElement,
            },
            BuilderRange::Size(bit_length) => {
                let bit_range = start..(start + *bit_length as usize);
                Self {
                    bit_range,
                    ty: BuiltRangeType::SingleElement,
                }
            }
            BuilderRange::None => {
                let bit_range = start..(start + default_bit_length);
                Self {
                    bit_range,
                    ty: BuiltRangeType::SingleElement,
                }
            }
            BuilderRange::ElementArray { sizings, size } => {
                let bit_range = match &size {
                    BuilderRangeArraySize::Size(element_bit_length) => {
                        let mut total_bits = *element_bit_length as usize;
                        for size in sizings {
                            total_bits *= size;
                        }
                        start..(start + total_bits)
                    }
                    BuilderRangeArraySize::Range(range) => range.clone(),
                };

                Self {
                    bit_range,
                    ty: BuiltRangeType::ElementArray(sizings.clone()),
                }
            }
            BuilderRange::BlockArray { sizings, size } => {
                let bit_range = match &size {
                    BuilderRangeArraySize::Size(total_bits) => {
                        start..(start + *total_bits as usize)
                    }
                    BuilderRangeArraySize::Range(range) => range.clone(),
                };
                Self {
                    bit_range,
                    ty: BuiltRangeType::BlockArray(sizings.clone()),
                }
            }
        }
    }
}
