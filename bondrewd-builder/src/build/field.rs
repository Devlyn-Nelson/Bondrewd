use crate::solved::field::{DynamicIdent, ResolverArrayType};

use super::{
    get_lit_int, get_lit_range, ArraySizings, BuilderRange, BuilderRangeArraySize, Endianness,
    OverlapOptions, ReserveFieldOption,
};

use darling::{FromField, FromMeta};
use quote::format_ident;
use syn::{spanned::Spanned, Error, Expr, Field, Ident, LitStr, Type};

#[derive(Debug)]
pub struct DataBuilder {
    /// The name or ident of the field.
    pub(crate) id: DynamicIdent,
    /// The approximate data type of the field. when solving, this must be
    /// filled.
    pub(crate) ty: DataType,
    /// Describes the properties of which techniques to use for bit extraction
    /// and modifications the inputs that they can have. When None, we are expecting
    /// either a Nested Type or the get it from the default.
    pub(crate) endianness: Option<Endianness>,
    /// The range of bits that this field will use.
    /// TODO this should become a new Range system that allows dynamic start and/or end bit-indices.
    pub(crate) bit_range: BuilderRange,
    /// Describes when the field should be considered.
    pub(crate) reserve: ReserveFieldOption,
    /// How much you care about the field overlapping other fields.
    pub(crate) overlap: OverlapOptions,
    pub(crate) is_captured_id: bool,
}

#[derive(Debug)]
pub struct ArrayBuilder {
    pub(crate) array_ty: ResolverArrayType,
    pub(crate) sizings: ArraySizings,
}

#[derive(Debug, Clone, Copy)]
pub enum RustByteSize {
    One,
    Two,
    Four,
    Eight,
    Sixteen,
}

impl RustByteSize {
    #[must_use]
    pub fn bytes(&self) -> usize {
        match self {
            RustByteSize::One => 1,
            RustByteSize::Two => 2,
            RustByteSize::Four => 4,
            RustByteSize::Eight => 8,
            RustByteSize::Sixteen => 16,
        }
    }
    #[must_use]
    pub fn bits(&self) -> usize {
        match self {
            RustByteSize::One => 8,
            RustByteSize::Two => 16,
            RustByteSize::Four => 32,
            RustByteSize::Eight => 64,
            RustByteSize::Sixteen => 128,
        }
    }
}

impl DataType {
    #[must_use]
    pub fn rust_size(&self) -> usize {
        match self {
            DataType::Number(number_type, rust_byte_size) => rust_byte_size.bytes(),
            DataType::Nested {
                ident,
                rust_byte_size,
            } => *rust_byte_size,
        }
    }
}

#[derive(Clone, Debug)]
pub enum DataType {
    /// field is a number or primitive. if the endianess is `None`, it will not solve.
    Number(NumberType, RustByteSize),
    /// This is a nested structure and does not have a know type. and the name of the struct shall be stored
    /// within.
    Nested {
        ident: String,
        rust_byte_size: usize,
    },
}

pub struct ParsedDataType {
    pub data_type: DataType,
    pub sizings: Option<Vec<usize>>,
}

impl DataType {
    pub fn type_ident(&self) -> syn::Result<Ident> {
        Ok(match self {
            DataType::Number(number_type, rust_byte_size) => {
                let ty = match number_type {
                    NumberType::Float => 'f',
                    NumberType::Unsigned => 'u',
                    NumberType::Signed => 'i',
                    NumberType::Char => return Ok(format_ident!("char")),
                    NumberType::Bool => return Ok(format_ident!("bool")),
                };
                let bits = rust_byte_size.bits();
                // TODOthis might not do numbers correctly.
                format_ident!("{ty}{bits}")
            }
            DataType::Nested { ident, .. } => Ident::from_string(&ident)?,
        })
    }
    pub fn needs_endianness(&self) -> bool {
        match self {
            DataType::Number(number_type, rust_byte_size) => match number_type {
                NumberType::Bool => false,
                _ => true,
            },
            DataType::Nested {
                ident,
                rust_byte_size,
            } => false,
        }
    }
    /// the returned vec<usize> is the sizings for an `ArrayBuilder`
    pub fn parse(
        ty: &syn::Type,
        attrs: &mut DataDarlingSimplified,
        default_endianness: &Endianness,
    ) -> syn::Result<ParsedDataType> {
        Self::parse_with_option(ty, attrs, default_endianness, None)
    }
    /// the returned vec<usize> is the sizings for an `ArrayBuilder`
    #[allow(clippy::too_many_lines)]
    fn parse_with_option(
        ty: &syn::Type,
        attrs: &mut DataDarlingSimplified,
        default_endianness: &Endianness,
        array_option: Option<Vec<usize>>,
    ) -> syn::Result<ParsedDataType> {
        let data_type = match ty {
            Type::Path(ref path) => {
                let out = Self::parse_path(&path.path, attrs)?;
                ParsedDataType {
                    data_type: out,
                    sizings: array_option.map(|mut thing| {
                        thing.reverse();
                        thing
                    }),
                }
            }
            Type::Array(ref array_path) => {
                // arrays must use a literal for length, because its would be hard any other way.
                let lit_int = get_lit_int(
                    &array_path.len,
                    &Ident::new("array_length", ty.span()),
                    None,
                )?;
                let mut array_info = if let Some(info) = array_option {
                    info
                } else {
                    vec![lit_int.base10_parse()?]
                };
                if let Ok(array_length) = lit_int.base10_parse::<usize>() {
                    array_info.push(array_length);
                    Self::parse_with_option(
                        array_path.elem.as_ref(),
                        attrs,
                        default_endianness,
                        Some(array_info),
                    )?
                    // match attrs.ty {
                    //     AttrBuilderType::ElementArray(ref element_bit_size, ref sub) => {
                    //         attrs.bit_range = match std::mem::take(&mut attrs.bit_range) {
                    //             BuilderRange::Range(ref range) => {
                    //                 if range.end < range.start {
                    //                     return Err(syn::Error::new(
                    //                         ty.span(),
                    //                         "range end is less than range start",
                    //                     ));
                    //                 }
                    //                 if range.end - range.start != *element_bit_size * array_length {
                    //                     return Err(
                    //                                 syn::Error::new(
                    //                                     ty.span(),
                    //                                     "Element arrays bit range didn't match (element bit size * array length)"
                    //                                 )
                    //                             );
                    //                 }
                    //                 BuilderRange::Range(range.clone())
                    //             }
                    //             BuilderRange::LastEnd(ref last_end) => BuilderRange::Range(
                    //                 *last_end..last_end + (array_length * *element_bit_size),
                    //             ),
                    //             BuilderRange::None => {
                    //                 return Err(syn::Error::new(
                    //                     ty.span(),
                    //                     "failed getting Range for element array",
                    //                 ));
                    //             }
                    //         };
                    //         let mut sub_attrs = attrs.clone();
                    //         if let Type::Array(_) = array_path.elem.as_ref() {
                    //         } else if let Some(ref ty) = sub.as_ref() {
                    //             sub_attrs.ty = ty.clone();
                    //         } else {
                    //             sub_attrs.ty = AttrBuilderType::None;
                    //         }
                    //         let mut sub_ty =
                    //             Self::parse(&array_path.elem, &mut sub_attrs, default_endianness)?;

                    //         match sub_ty {
                    //             DataType::Enum { ref mut size, .. }
                    //             | DataType::Struct { ref mut size, .. } => {
                    //                 *size = size.div_ceil(array_length);
                    //             }
                    //             _ => {}
                    //         }

                    //         let type_ident = &sub_ty.type_quote();
                    //         DataType::ElementArray {
                    //             sub_type: Box::new(SubFieldInfo { ty: sub_ty }),
                    //             length: array_length,
                    //             type_quote: quote! {[#type_ident;#array_length]},
                    //         }
                    //     }
                    //     AttrBuilderType::BlockArray(_) => {
                    //         let mut sub_attrs = attrs.clone();
                    //         if let Type::Array(_) = array_path.elem.as_ref() {
                    //         } else {
                    //             sub_attrs.ty = AttrBuilderType::None;
                    //         }

                    //         let sub_ty =
                    //             Self::parse(&array_path.elem, &mut sub_attrs, default_endianness)?;
                    //         attrs.endianness = sub_attrs.endianness;
                    //         let type_ident = &sub_ty.type_quote();
                    //         DataType::BlockArray {
                    //             sub_type: Box::new(SubFieldInfo { ty: sub_ty }),
                    //             length: array_length,
                    //             type_quote: quote! {[#type_ident;#array_length]},
                    //         }
                    //     }
                    //     AttrBuilderType::Enum(_, _) | AttrBuilderType::Struct(_) => {
                    //         let mut sub_attrs = attrs.clone();
                    //         if let Type::Array(_) = array_path.elem.as_ref() {
                    //         } else {
                    //             sub_attrs.ty = attrs.ty.clone();
                    //         }

                    //         let sub_ty =
                    //             Self::parse(&array_path.elem, &mut sub_attrs, default_endianness)?;
                    //         attrs.endianness = sub_attrs.endianness;
                    //         let type_ident = &sub_ty.type_quote();
                    //         DataType::BlockArray {
                    //             sub_type: Box::new(SubFieldInfo { ty: sub_ty }),
                    //             length: array_length,
                    //             type_quote: quote! {[#type_ident;#array_length]},
                    //         }
                    //     }
                    //     AttrBuilderType::None => {
                    //         let mut sub_attrs = attrs.clone();
                    //         if let Type::Array(_) = array_path.elem.as_ref() {
                    //         } else {
                    //             sub_attrs.ty = AttrBuilderType::None;
                    //         }
                    //         let sub_ty =
                    //             Self::parse(&array_path.elem, &mut sub_attrs, default_endianness)?;
                    //         attrs.bit_range = match std::mem::take(&mut attrs.bit_range) {
                    //             BuilderRange::Range(ref range) => {
                    //                 if range.end < range.start {
                    //                     return Err(syn::Error::new(
                    //                         ty.span(),
                    //                         "range end is less than range start",
                    //                     ));
                    //                 }
                    //                 if range.end - range.start % array_length != 0 {
                    //                     return Err(
                    //                                 syn::Error::new(
                    //                                     ty.span(),
                    //                                     "Array Inference failed because given total bit_length does not split up evenly between elements, perhaps try using `element_bit_length` attribute"
                    //                                 )
                    //                             );
                    //                 }
                    //                 BuilderRange::Range(range.clone())
                    //             }
                    //             BuilderRange::LastEnd(ref last_end) => {
                    //                 let element_bit_length = sub_ty.get_element_bit_length();
                    //                 BuilderRange::Range(
                    //                     *last_end..last_end + (array_length * element_bit_length),
                    //                 )
                    //             }
                    //             BuilderRange::None => {
                    //                 return Err(syn::Error::new(
                    //                     ty.span(),
                    //                     "failed getting Range for element array",
                    //                 ));
                    //             }
                    //         };
                    //         let type_ident = &sub_ty.type_quote();
                    //         DataType::ElementArray {
                    //             sub_type: Box::new(SubFieldInfo { ty: sub_ty }),
                    //             length: array_length,
                    //             type_quote: quote! {[#type_ident;#array_length]},
                    //         }
                    //     }
                    // }
                } else {
                    return Err(Error::new(
                        array_path.bracket_token.span.span(),
                        "failed parsing array length as literal integer",
                    ));
                }
            }
            _ => {
                return Err(syn::Error::new(ty.span(), "Unsupported field type"));
            }
        };
        // if the type is a number and its endianess is None (numbers should have endianess) then we
        // apply the structs default (which might also be None)
        if attrs.endianness.is_none() && data_type.data_type.rust_size() == 1 {
            // currently nested fields that are 1 byte or less are expected to go through big endian logic.
            attrs.endianness = Some(Endianness::big())
        }
        if data_type.data_type.needs_endianness() && attrs.endianness.is_none() {
            attrs.endianness = Some(default_endianness.clone());
        }

        Ok(data_type)
    }
    #[allow(clippy::too_many_lines)]
    fn parse_path(path: &syn::Path, attrs: &mut DataDarlingSimplified) -> syn::Result<DataType> {
        if let Some(last_segment) = path.segments.last() {
            let type_quote = &last_segment.ident;
            let field_type_name = last_segment.ident.to_string();
            match field_type_name.as_str() {
                "bool" => match attrs.bits {
                    #[allow(clippy::range_plus_one)]
                    DataBuilderRange::None => {
                        Ok(DataType::Number(NumberType::Bool, RustByteSize::One))
                    }
                    _ => Ok(DataType::Number(NumberType::Bool, RustByteSize::One)),
                },
                "u8" => Ok(DataType::Number(NumberType::Unsigned, RustByteSize::One)),
                "i8" => Ok(DataType::Number(NumberType::Signed, RustByteSize::One)),
                "u16" => Ok(DataType::Number(NumberType::Unsigned, RustByteSize::Two)),
                "i16" => Ok(DataType::Number(NumberType::Signed, RustByteSize::Two)),
                "f32" => {
                    if let DataBuilderRange::Range(ref span) = attrs.bits {
                        if 32 != span.end - span.start {
                            return Err(syn::Error::new(path.span(), format!("f32 must be full sized, if this is a problem for you open an issue.. provided bit length = {}.", span.end - span.start)));
                        }
                    }
                    Ok(DataType::Number(NumberType::Float, RustByteSize::Four))
                }
                "u32" => Ok(DataType::Number(NumberType::Unsigned, RustByteSize::Four)),
                "i32" => Ok(DataType::Number(NumberType::Signed, RustByteSize::Four)),
                "char" => Ok(DataType::Number(NumberType::Char, RustByteSize::Four)),
                "f64" => {
                    if let DataBuilderRange::Range(ref span) = attrs.bits {
                        if 64 != span.end - span.start {
                            return Err(syn::Error::new(path.span(), format!("f64 must be full sized, if this is a problem for you open an issue. provided bit length = {}.", span.end - span.start)));
                        }
                    }
                    Ok(DataType::Number(NumberType::Float, RustByteSize::Eight))
                }
                "u64" => Ok(DataType::Number(NumberType::Unsigned, RustByteSize::Eight)),
                "i64" => Ok(DataType::Number(NumberType::Signed, RustByteSize::Eight)),
                "u128" => Ok(DataType::Number(
                    NumberType::Unsigned,
                    RustByteSize::Sixteen,
                )),
                "i128" => Ok(DataType::Number(NumberType::Signed, RustByteSize::Sixteen)),
                "usize" | "isize" => Err(Error::new(
                    path.span(),
                    "usize and isize are not supported due to ambiguous sizing".to_string(),
                )),
                _ => Ok(DataType::Nested {
                    ident: type_quote.to_string(),
                    rust_byte_size: match attrs.bits {
                        DataBuilderRange::Range(ref range) => (range.end - range.start).div_ceil(8),
                        DataBuilderRange::Size(size) => size,
                        DataBuilderRange::None => {
                            return Err(Error::new(
                                path.span(),
                                format!("Can not determine size of field type. If the type is a struct or enum that implements the Bondrewd::Bitfield traits you need to define the `bit_length` via attribute of the same name, because bondrewd has no way to determine the size of another struct at compile time. [{field_type_name}]"),
                            ));
                        }
                    },
                }),
            }
        } else {
            Err(Error::new(path.span(), "field has no Type?"))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NumberType {
    /// Floating point numbers
    ///
    /// # Valid
    /// - f32
    /// - f64
    Float,
    /// Unsigned numbers
    ///
    /// # Valid
    /// - u8
    /// - u16
    /// - u32
    /// - u64
    /// - u128
    Unsigned,
    /// Signed numbers
    ///
    /// # Valid
    /// - i8
    /// - i16
    /// - i32
    /// - i64
    /// - i128
    Signed,
    /// Just `Char`
    Char,
    /// Boolean types
    Bool,
}

pub enum IDontExist {
    Range,
    Size,
}

impl DataBuilder {
    #[must_use]
    pub fn new(name: DynamicIdent, ty: DataType) -> Self {
        Self {
            id: name,
            ty,
            endianness: None,
            bit_range: BuilderRange::None,
            reserve: ReserveFieldOption::NotReserve,
            overlap: OverlapOptions::None,
            is_captured_id: false,
        }
    }
    #[must_use]
    pub fn id(&self) -> &DynamicIdent {
        &self.id
    }

    pub fn set_endianess(&mut self, e: Endianness) {
        self.endianness = Some(e);
    }

    pub fn with_endianess(mut self, e: Endianness) -> Self {
        self.endianness = Some(e);
        self
    }
    pub fn bit_length(&self) -> usize {
        // TODO test if this should not include redundant bytes or and verify none of the call sites require that.
        self.bit_range.bit_length()
    }
    pub fn parse(
        field: &syn::Field,
        fields: &[DataBuilder],
        default_endianness: &Endianness,
    ) -> syn::Result<Self> {
        let ident: DynamicIdent = if let Some(ref name) = field.ident {
            name.clone().into()
        } else {
            (fields.len(), field.span()).into()
            // return Err(Error::new(Span::call_site(), "all fields must be named"));
        };
        // parse all attrs. which will also give us the bit locations
        // NOTE read only attribute assumes that the value should not effect the placement of the rest og
        let last_relevant_field = fields.iter().filter(|x| !x.overlap.is_redundant()).last();

        // let mut attrs_builder = AttrBuilder::parse(field, last_relevant_field)?;
        let mut attrs = DataDarling::from_field(field)?.simplify(field)?;
        // check the field for supported types.
        let data_type = DataType::parse(&field.ty, &mut attrs, default_endianness)?;

        // TODO make sure fields that don't have a solved range here get solved during the solve process.
        // let attrs: Attributes = match attrs_builder.try_into() {
        //     Ok(attr) => attr,
        //     Err(fix_me) => {
        //         let mut start = 0;
        //         if let Some(last_value) = last_relevant_field {
        //             start = last_value.attrs.bit_range.end;
        //         }
        //         fix_me.fix(start..start + (data_type.size() * 8))
        //     }
        // };

        // construct the field we are parsed.
        // let new_field = FieldInfo {
        //     ident: Box::new(ident),
        //     ty: data_type,
        //     attrs,
        // };
        let overlap = if attrs.redundant {
            if attrs.overlapping_bits.is_none() {
                OverlapOptions::Redundant
            } else {
                return Err(Error::new(field.span(), "Field has `overlapping_bits` and `redundant` defined. \
                Only 1 of these is allowed on a single field, if the entire fields overlaps use `redundant` \
                otherwise use `overlapping_bits`."));
            }
        } else {
            attrs
                .overlapping_bits
                .map(|bits| OverlapOptions::Allow(bits))
                .unwrap_or(OverlapOptions::None)
        };
        let reserve = if attrs.read_only {
            if attrs.reserve {
                return Err(Error::new(field.span(), "Field has `read_only` and `reserve` defined. \
                Only 1 of these is allowed on a single field, if there is no need to read the values \
                during a `from_bytes` call use `reserve`, if you want the value to be read use `read_only`."));
            } else {
                ReserveFieldOption::ReadOnly
            }
        } else if attrs.reserve {
            ReserveFieldOption::ReserveField
        } else {
            ReserveFieldOption::NotReserve
        };
        let bit_range = if let Some(sizings) = data_type.sizings {
            if let Some(a_ty) = attrs.array {
                match a_ty {
                    DataDarlingSimplifiedArrayType::Block(size) => match attrs.bits {
                        DataBuilderRange::Range(range) => {
                            if range.end - range.start != size {
                                return Err(Error::new(field.span(), "`bits` attribute's total bit length and the size provided for the block array size do not match."));
                            }

                            BuilderRange::BlockArray {
                                sizings,
                                size: BuilderRangeArraySize::Range(range.clone()),
                            }
                        }
                        DataBuilderRange::Size(other_size) => {
                            if other_size != size {
                                return Err(Error::new(
                                    field.span(),
                                    "attributes contain conflicting total bit length.",
                                ));
                            }

                            BuilderRange::BlockArray {
                                sizings,
                                size: BuilderRangeArraySize::Size(size),
                            }
                        }
                        DataBuilderRange::None => BuilderRange::None,
                    },
                    DataDarlingSimplifiedArrayType::Element(size) => {
                        let mut total_size = size;
                        for s in &sizings {
                            total_size *= s;
                        }
                        match attrs.bits {
                            DataBuilderRange::Range(range) => {
                                if range.end - range.start != total_size {
                                    return Err(Error::new(field.span(), "`bits` attribute's total bit length and the size provided for the block array size do not match."));
                                }

                                BuilderRange::ElementArray {
                                    sizings,
                                    size: BuilderRangeArraySize::Range(range.clone()),
                                }
                            }
                            DataBuilderRange::Size(other_size) => {
                                if other_size != total_size {
                                    return Err(Error::new(
                                        field.span(),
                                        "attributes contain conflicting total bit length.",
                                    ));
                                }

                                BuilderRange::ElementArray {
                                    sizings,
                                    size: BuilderRangeArraySize::Size(size),
                                }
                            }
                            DataBuilderRange::None => BuilderRange::None,
                        }
                    }
                }
            } else {
                let mut elements = 1;
                for s in &sizings {
                    elements *= s;
                }
                match attrs.bits {
                    DataBuilderRange::Range(range) => {
                        if (range.end - range.start) % elements != 0 {
                            return Err(Error::new(field.span(), "`bits` attribute's total bit length and does not evenly divide by elements in array."));
                        }

                        BuilderRange::ElementArray {
                            sizings,
                            size: BuilderRangeArraySize::Range(range.clone()),
                        }
                    }
                    DataBuilderRange::Size(size) => {
                        if size % elements != 0 {
                            return Err(Error::new(field.span(), "attributes defined bit length does not evenly divide by elements in array."));
                        }

                        BuilderRange::ElementArray {
                            sizings,
                            size: BuilderRangeArraySize::Size(size / elements),
                        }
                    }
                    DataBuilderRange::None => BuilderRange::None,
                }
            }
        } else {
            if attrs.array.is_some() {
                return Err(Error::new(field.span(), "The attributes provided imply this is an array but bondrewd's type determination says it is not. if the type is not an array verify you are not using an attribute starting with `element` or `block`."));
            }
            attrs.bits.into()
        };
        let new_field = Self {
            id: if let Some(id) = &field.ident {
                id.into()
            } else {
                return Err(Error::new(
                    field.span(),
                    "Currently unnamed fields are not supported.",
                ));
            },
            ty: data_type.data_type,
            endianness: attrs.endianness,
            bit_range,
            reserve,
            overlap,
            is_captured_id: attrs.capture_id,
        };
        // TODO i think the overlap checking happens during solve process, please verify.
        // // check to verify there are no overlapping bit ranges from previously parsed fields.
        // for (i, parsed_field) in fields.iter().enumerate() {
        //     if parsed_field.overlapping(&new_field) {
        //         return Err(Error::new(
        //             Span::call_site(),
        //             format!("fields {} and {} overlap", i, fields.len()),
        //         ));
        //     }
        // }

        Ok(new_field)
    }
}

#[derive(Debug, FromField)]
pub struct DataDarling {
    endianness: Option<LitStr>,
    bit_length: Option<usize>,
    byte_length: Option<usize>,
    bits: Option<syn::Expr>,
    element_bit_length: Option<usize>,
    element_byte_length: Option<usize>,
    block_bit_length: Option<usize>,
    block_byte_length: Option<usize>,
    overlapping_bits: Option<usize>,
    reserve: bool,
    read_only: bool,
    capture_id: bool,
    redundant: bool,
}

impl DataDarling {
    fn bits(&self) -> syn::Result<Option<DataBuilderRange>> {
        if let Some(b) = &self.bits {
            Ok(Some(DataBuilderRange::range_from_expr(b)?))
        } else {
            Ok(None)
        }
    }
    fn endianness(&self, field: &Field) -> syn::Result<Option<Endianness>> {
        let Some(val) = &self.endianness else {
            return Ok(None);
        };
        Ok(Some(Endianness::from_expr(val)?))
    }
    fn simplify(self, field: &Field) -> Result<DataDarlingSimplified, syn::Error> {
        let mut bit_defs = 0;
        if self.bit_length.is_some() {
            bit_defs += 1;
        }
        if self.byte_length.is_some() {
            bit_defs += 1;
        }
        if self.bits.is_some() {
            bit_defs += 1;
        }
        if bit_defs > 1 {
            return Err(Error::new(field.span(), "please use only one of the following attributes: `bit_length`, `byte_length`, `bits`"));
        }
        let bits = {
            let thing = self.bits()?.or(self
                .bit_length
                .or(self.byte_length.map(|bytes| bytes * 8))
                .map(|bits| DataBuilderRange::Size(bits)));
            // let Some(out) = thing else{
            //     return Err(syn::Error::new(field.span(), "Could not determine amount of bits to use for field, either `element_bit_length` or `element_byte_length` attributes"));
            // };
            thing.unwrap_or(DataBuilderRange::None)
        };

        let element_bit_length = if self.element_bit_length.is_some()
            && self.element_byte_length.is_some()
        {
            return Err(syn::Error::new(field.span(), "please use either `element_bit_length` or `element_byte_length` attributes, not both"));
        } else {
            self.element_bit_length
                .or(self.element_byte_length.map(|bytes| bytes * 8))
        };
        let block_bit_length =
            if self.block_bit_length.is_some() && self.block_byte_length.is_some() {
                return Err(syn::Error::new(
                field.span(),
                "please use either `block_bit_length` or `block_byte_length` attributes, not both",
            ));
            } else {
                self.block_bit_length
                    .or(self.block_byte_length.map(|bytes| bytes * 8))
            };

        Ok(DataDarlingSimplified {
            endianness: self.endianness(field)?,
            array: if let Some(bit_len) = element_bit_length {
                if block_bit_length.is_some() {
                    return Err(syn::Error::new(
                        field.span(),
                        "Array type can not be both element and block, check field attributes.",
                    ));
                } else {
                    Some(DataDarlingSimplifiedArrayType::Element(bit_len))
                }
            } else if let Some(block_len) = block_bit_length {
                Some(DataDarlingSimplifiedArrayType::Block(block_len))
            } else {
                None
            },
            bits,
            overlapping_bits: self.overlapping_bits,
            reserve: self.reserve,
            read_only: self.read_only,
            capture_id: self.capture_id,
            redundant: self.redundant,
        })
    }
}

#[derive(Debug)]
pub struct DataDarlingSimplified {
    endianness: Option<Endianness>,
    bits: DataBuilderRange,
    array: Option<DataDarlingSimplifiedArrayType>,
    overlapping_bits: Option<usize>,
    reserve: bool,
    read_only: bool,
    capture_id: bool,
    redundant: bool,
}

#[derive(Clone, Debug)]
pub enum DataBuilderRange {
    /// A range of bits to use. solve this is easy, but note that it is an exclusive range, meaning the
    /// end is NOT included.
    Range(std::ops::Range<usize>),
    /// Amount of bits to consume
    Size(usize),
    /// Will not solve, must be another variant.
    None,
}

impl DataBuilderRange {
    /// This is intended for use in `bondrewd-derive`.
    ///
    /// Tries to extract a range from a `&Expr`. there is no need to check the type of expr.
    /// If the Result returns `Err` then a parsing error occurred and should be reported as an error to user.
    /// If `Ok(None)`, no error but `expr` was not valid for housing a range.
    pub fn range_from_expr(expr: &Expr) -> syn::Result<Self> {
        let lit = get_lit_range(expr)?;
        Ok(Self::Range(lit))
    }
}

#[derive(Debug)]
pub enum DataDarlingSimplifiedArrayType {
    /// Value is total bits to use for block.
    Block(usize),
    /// Value is total bits to use for each element.
    Element(usize),
}
