use crate::common::object::Info as ObjectInfo;
use crate::common::r#enum::{AttrInfo as EnumAttrInfo, Info as EnumInfo};
use crate::common::r#struct::Info as StructInfo;
use crate::common::{
    field::{
        Attributes, DataType, Info as FieldInfo, NumberSignage, OverlapOptions, ReserveFieldOption,
        SubInfo as SubFieldInfo,
    },
    Endianness,
};
use crate::common::{AttrInfo, StructEnforcement};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::Error;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Expr, Fields, Ident, Lit, Meta, Token};

use super::r#enum::AttrInfoBuilder;

/// `id_bits` is the amount of bits the enum's id takes.
fn get_id_type(id_bits: usize, span: Span) -> syn::Result<TokenStream> {
    match id_bits {
        0..=8 => Ok(quote! {u8}),
        9..=16 => Ok(quote! {u16}),
        17..=32 => Ok(quote! {u32}),
        33..=64 => Ok(quote! {u64}),
        65..=128 => Ok(quote! {u128}),
        _ => Err(syn::Error::new(span, "id size is invalid")),
    }
}

impl ObjectInfo {
    fn parse_struct_attrs(
        attrs: &[Attribute],
        attrs_info: &mut AttrInfo,
        is_variant: bool,
    ) -> syn::Result<()> {
        for attr in attrs {
            if attr.path().is_ident("bondrewd") {
                let nested =
                    attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                for meta in &nested {
                    if !StructInfo::parse_attrs_meta(attrs_info, meta, is_variant)? {
                        return Err(syn::Error::new(
                            meta.span(),
                            format!(
                                "invalid {} attribute",
                                if is_variant { "variant" } else { "struct" }
                            ),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_enum_attrs(
        attrs: &[Attribute],
        attrs_info: &mut AttrInfo,
        enum_attrs_info: &mut AttrInfoBuilder,
    ) -> syn::Result<()> {
        for attr in attrs {
            if attr.path().is_ident("bondrewd") {
                let nested =
                    attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                for meta in &nested {
                    EnumInfo::parse_attrs_meta(attrs_info, enum_attrs_info, meta)?;
                }
            }
        }
        Ok(())
    }
    // Parses the Expression, looking for a literal number expression
    fn parse_lit_discriminant_expr(input: &Expr) -> syn::Result<u128> {
        match input {
            Expr::Lit(ref lit) => match lit.lit {
                Lit::Int(ref i) => Ok(i.base10_parse()?),
                _ => Err(syn::Error::new(
                    input.span(),
                    "non-integer literals for custom discriminant are illegal.",
                )),
            },
            _ => Err(syn::Error::new(
                input.span(),
                "non-literal expressions for custom discriminant are illegal.",
            )),
        }
    }
    #[allow(clippy::too_many_lines)]
    pub fn parse(input: &DeriveInput) -> syn::Result<Self> {
        // get the struct, error out if not a struct
        let mut attrs = AttrInfo::default();
        let name = input.ident.clone();
        match input.data {
            syn::Data::Struct(ref data) => {
                let tuple = matches!(data.fields, syn::Fields::Unnamed(_));
                Self::parse_struct_attrs(&input.attrs, &mut attrs, false)?;
                let fields = Self::parse_fields(&name, &data.fields, &attrs, None, tuple)?;
                Ok(Self::Struct(StructInfo {
                    name,
                    attrs,
                    fields,
                    vis: crate::common::Visibility(input.vis.clone()),
                    tuple,
                }))
            }
            syn::Data::Enum(ref data) => {
                let mut enum_attrs = AttrInfoBuilder::default();
                Self::parse_enum_attrs(&input.attrs, &mut attrs, &mut enum_attrs)?;
                let mut variants: Vec<StructInfo> = Vec::default();
                let (id_field_type, id_bits) = {
                    let id_bits = if let Some(id_bits) = enum_attrs.id_bits {
                        id_bits
                    } else if let (Some(payload_size), Some(total_size)) =
                        (enum_attrs.payload_bit_size, enum_attrs.total_bit_size)
                    {
                        total_size - payload_size
                    } else {
                        return Err(syn::Error::new(
                            data.enum_token.span(),
                            "Must define the length of the id use #[bondrewd(id_bit_length = AMOUNT_OF_BITS)]",
                        ));
                    };
                    (
                        DataType::Number {
                            size: id_bits.div_ceil(8),
                            sign: NumberSignage::Unsigned,
                            type_quote: get_id_type(id_bits, name.span())?,
                        },
                        id_bits,
                    )
                };
                let id_field = FieldInfo {
                    ident: Box::new(format_ident!("{}", EnumInfo::VARIANT_ID_NAME).into()),
                    ty: id_field_type,
                    attrs: Attributes {
                        endianness: Box::new(attrs.default_endianess.clone()),
                        // this need to accommodate tailing ids, currently this locks the
                        // id field to the first field read from the starting point of reading.
                        // TODO make sure this gets corrected if the id size is unknown.
                        bit_range: 0..id_bits,
                        reserve: ReserveFieldOption::EnumId,
                        overlap: OverlapOptions::None,
                        capture_id: false,
                    },
                };
                for variant in &data.variants {
                    let tuple = matches!(variant.fields, syn::Fields::Unnamed(_));
                    let mut attrs = attrs.clone();
                    if let Some((_, ref expr)) = variant.discriminant {
                        let parsed = Self::parse_lit_discriminant_expr(expr)?;
                        attrs.id = Some(parsed);
                    }
                    Self::parse_struct_attrs(&variant.attrs, &mut attrs, true)?;
                    let variant_name = variant.ident.clone();
                    // TODO currently we always add the id field, but some people might want the id to be a
                    // field in the variant. this would no longer need to insert the id as a "fake-field".
                    let fields = Self::parse_fields(
                        &variant_name,
                        &variant.fields,
                        &attrs,
                        Some(id_field.clone()),
                        tuple,
                    )?;
                    variants.push(StructInfo {
                        name: variant_name,
                        attrs,
                        fields,
                        vis: crate::common::Visibility(input.vis.clone()),
                        tuple,
                    });
                }
                // detect and fix variants without ids and verify non conflict.
                let mut used_ids: Vec<u128> = Vec::default();
                let mut unassigned_indices: Vec<usize> = Vec::default();
                let mut invalid_index: Option<usize> = None;
                let mut largest = 0;
                for (i, variant) in variants.iter().enumerate() {
                    if let Some(ref value) = variant.attrs.id {
                        if used_ids.contains(value) {
                            return Err(Error::new(
                                variant.name.span(),
                                "variant identifier used twice.",
                            ));
                        }
                        used_ids.push(*value);
                    } else {
                        unassigned_indices.push(i);
                    }
                    if variant.attrs.invalid {
                        if invalid_index.is_none() {
                            invalid_index = Some(i);
                        } else {
                            return Err(Error::new(
                                variant.name.span(),
                                "second catch invalid variant found. only 1 is currently allowed.",
                            ));
                        }
                    }
                }
                if !unassigned_indices.is_empty() {
                    let mut current_guess: u128 = 0;
                    for i in unassigned_indices {
                        while used_ids.contains(&current_guess) {
                            current_guess += 1;
                        }
                        variants[i].attrs.id = Some(current_guess);
                        used_ids.push(current_guess);
                        current_guess += 1;
                    }
                }
                for variant in variants.iter() {
                    // verify the size doesn't go over set size.
                    let size = variant.total_bits_no_fill();
                    if largest < size {
                        largest = size;
                    }
                    let variant_id_field = {
                        if let Some(id) = variant.get_id_field()? {
                            id
                        } else {
                            return Err(syn::Error::new(variant.name.span(), "failed to get variant field for variant. (this is a bondrewd issue, please report issue)"));
                        }
                    };

                    if let Some(bit_size) = enum_attrs.payload_bit_size {
                        if bit_size < size - variant_id_field.bit_size() {
                            return Err(Error::new(
                                variant.name.span(),
                                format!("variant is larger than defined payload_size of enum. defined size: {bit_size}. variant size: {}", size- variant_id_field.bit_size()),
                            ));
                        }
                    } else if let (Some(bit_size), Some(id_size)) =
                        (enum_attrs.total_bit_size, enum_attrs.id_bits)
                    {
                        if bit_size - id_size < size - variant_id_field.bit_size() {
                            return Err(Error::new(
                                variant.name.span(),
                                format!("variant with id is larger than defined total_size of enum. defined size: {}. calculated size: {}", bit_size - id_size, size - variant_id_field.bit_size()),
                            ));
                        }
                    }
                }
                if let Some(ii) = invalid_index {
                    let var = variants.remove(ii);
                    variants.push(var);
                }
                // find minimal id size from largest id value
                used_ids.sort_unstable();
                let min_id_size = if let Some(last_id) = used_ids.last() {
                    let mut x = *last_id;
                    // find minimal id size from largest id value
                    let mut n = 0;
                    while x != 0 {
                        x >>= 1;
                        n += 1;
                    }
                    n
                } else {
                    return Err(Error::new(
                        data.enum_token.span(),
                        "found no variants and could not determine size of id".to_string(),
                    ));
                };
                let enum_attrs = match (enum_attrs.payload_bit_size, enum_attrs.total_bit_size) {
                    (Some(payload), None) => {
                        if let Some(id) = enum_attrs.id_bits {
                            EnumAttrInfo {
                                payload_bit_size: payload,
                                id_bits: id,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        } else {
                            EnumAttrInfo {
                                payload_bit_size: payload,
                                id_bits: min_id_size,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        }
                    }
                    (None, Some(total)) => {
                        if let Some(id) = enum_attrs.id_bits {
                            EnumAttrInfo {
                                payload_bit_size: total - id,
                                id_bits: id,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        } else if largest < total {
                            let id = total - largest;
                            EnumAttrInfo {
                                payload_bit_size: largest,
                                id_bits: id,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        } else {
                            return Err(Error::new(
                                data.enum_token.span(),
                                "specified total is not smaller than the largest payload size, meaning there is not room the the variant id.".to_string(),
                            ));
                        }
                    }
                    (Some(payload), Some(total)) => {
                        if let Some(id) = enum_attrs.id_bits {
                            if payload + id != total {
                                return Err(Error::new(
                                    data.enum_token.span(),
                                    format!("total_size, payload_size, and id_size where all specified but id_size ({id}) + payload_size ({payload}) is not equal to total_size ({total})"),
                                ));
                            }
                            if payload < largest {
                                return Err(Error::new(
                                    data.enum_token.span(),
                                    "detected a variant over the maximum defined size.".to_string(),
                                ));
                            }
                            EnumAttrInfo {
                                id_bits: id,
                                id_position: enum_attrs.id_position,
                                payload_bit_size: payload,
                                attrs: attrs.clone(),
                            }
                        } else {
                            EnumAttrInfo {
                                payload_bit_size: largest,
                                id_bits: min_id_size,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        }
                    }
                    _ => {
                        if let Some(id) = enum_attrs.id_bits {
                            EnumAttrInfo {
                                id_bits: id,
                                id_position: enum_attrs.id_position,
                                payload_bit_size: largest,
                                attrs: attrs.clone(),
                            }
                        } else {
                            EnumAttrInfo {
                                payload_bit_size: largest,
                                id_bits: min_id_size,
                                id_position: enum_attrs.id_position,
                                attrs: attrs.clone(),
                            }
                        }
                    }
                };
                if enum_attrs.id_bits < min_id_size {
                    return Err(Error::new(
                        data.enum_token.span(),
                        "the bit size being used is less than required to describe each variant"
                            .to_string(),
                    ));
                }
                if enum_attrs.payload_bit_size + enum_attrs.id_bits < largest {
                    return Err(Error::new(
                        data.enum_token.span(),
                        "the payload size being used is less than largest variant".to_string(),
                    ));
                }
                // let id_field_ty = FieldDataType::Number(
                //     enum_attrs.id_bits,
                //     NumberSignage::Unsigned,
                //     get_id_type(enum_attrs.id_bits, name.span())?,
                // );
                // add fill_bits if needed.
                for v in &mut variants {
                    let first_bit = v.total_bits();
                    if first_bit < largest {
                        let fill_bytes_size = (largest - first_bit).div_ceil(8);
                        let ident = quote::format_ident!("fill_bits");
                        v.fields.push(FieldInfo {
                            ident: Box::new(ident.into()),
                            attrs: Attributes {
                                bit_range: first_bit..largest,
                                endianness: Box::new(Endianness::big()),
                                reserve: ReserveFieldOption::FakeField,
                                overlap: OverlapOptions::None,
                                capture_id: false,
                            },
                            ty: DataType::BlockArray {
                                sub_type: Box::new(SubFieldInfo {
                                    ty: DataType::Number {
                                        size: 1,
                                        sign: NumberSignage::Unsigned,
                                        type_quote: quote! {u8},
                                    },
                                }),
                                length: fill_bytes_size,
                                type_quote: quote! {[u8;#fill_bytes_size]},
                            },
                        });
                    }
                }
                Ok(Self::Enum(EnumInfo {
                    name,
                    variants,
                    attrs: enum_attrs,
                    vis: crate::common::Visibility(input.vis.clone()),
                }))
            }
            syn::Data::Union(_) => Err(Error::new(Span::call_site(), "input can not be a union")),
        }
    }
    #[allow(clippy::too_many_lines)]
    pub fn parse_fields(
        name: &Ident,
        fields: &Fields,
        attrs: &AttrInfo,
        first_field: Option<FieldInfo>,
        tuple: bool,
    ) -> syn::Result<Vec<FieldInfo>> {
        let (mut parsed_fields, is_enum) = if let Some(f) = first_field {
            (vec![f], true)
        } else {
            (Vec::default(), false)
        };
        // get the list of fields in syn form, error out if unit struct (because they have no data, and
        // data packing/analysis don't seem necessary)
        let fields = match fields {
            syn::Fields::Named(ref named_fields) => Some(
                named_fields
                    .named
                    .iter()
                    .cloned()
                    .collect::<Vec<syn::Field>>(),
            ),
            syn::Fields::Unnamed(ref fields) => {
                Some(fields.unnamed.iter().cloned().collect::<Vec<syn::Field>>())
            }
            syn::Fields::Unit => {
                if parsed_fields.first().is_none() {
                    return Err(Error::new(name.span(), "Packing a Unit Struct (Struct with no data) seems pointless to me, so i didn't write code for it."));
                }
                None
            }
        };

        // figure out what the field are and what/where they should be in byte form.
        let mut bit_size = if let Some(id_field) = parsed_fields.first() {
            id_field.bit_size()
        } else {
            0
        };
        if let Some(fields) = fields {
            for (i, ref field) in fields.iter().enumerate() {
                let mut parsed_field = FieldInfo::from_syn_field(field, &parsed_fields, attrs)?;
                if parsed_field.attrs.capture_id {
                    if is_enum {
                        if i == 0 {
                            match (&parsed_fields[0].ty, &mut parsed_field.ty) {
                                (DataType::Number{sign: ref bon_sign, type_quote: ref bon_ty, ..}, DataType::Number{sign: ref user_sign, type_quote: ref user_ty, ..}) => {
                                    if parsed_fields[0].attrs.bit_range != parsed_field.attrs.bit_range {
                                        parsed_field.attrs.bit_range = parsed_fields[0].attrs.bit_range.clone();
                                    }
                                    if bon_sign != user_sign {
                                        return Err(Error::new(field.span(), format!("capture_id field must be unsigned. bondrewd will enforce the type as {bon_ty}")));
                                    }else if bon_ty.to_string() != user_ty.to_string() {
                                        return Err(Error::new(field.span(), format!("capture_id field currently must be {bon_ty} in this instance, because bondrewd makes an assumption about the id type. changing this would be difficult")));
                                    }
                                    let old_id = parsed_fields.remove(0);
                                    if tuple {
                                        parsed_field.ident = old_id.ident;
                                    }
                                }
                                (DataType::Number{ type_quote: bon_ty, ..}, _) => return Err(Error::new(field.span(), format!("capture_id field must be an unsigned number. detected type is {bon_ty}."))),
                                _ => return Err(Error::new(field.span(), "an error with bondrewd has occurred, the id field should be a number but bondrewd did not use a number for the id.")),
                            }
                        } else {
                            return Err(Error::new(
                                field.span(),
                                "capture_id attribute must be the first field.",
                            ));
                        }
                    } else {
                        return Err(Error::new(
                            field.span(),
                            "capture_id attribute is intended for enum variants only.",
                        ));
                    }
                } else {
                    bit_size += parsed_field.bit_size();
                }
                parsed_fields.push(parsed_field);
            }
        }

        match attrs.enforcement {
            StructEnforcement::NoRules => {}
            StructEnforcement::EnforceFullBytes => {
                if bit_size % 8 != 0 {
                    return Err(syn::Error::new(
                        name.span(),
                        "BIT_SIZE modulus 8 is not zero",
                    ));
                }
            }
            StructEnforcement::EnforceBitAmount(expected_total_bits) => {
                if bit_size != expected_total_bits {
                    return Err(syn::Error::new(
                        name.span(),
                        format!(
                            "Bit Enforcement failed because bondrewd detected {bit_size} total bits used by defined fields, but the bit enforcement attribute is defined as {expected_total_bits} bits.",
                        ),
                    ));
                }
            }
        }

        let first_bit = if let Some(last_range) = parsed_fields.iter().last() {
            last_range.attrs.bit_range.end
        } else {
            0_usize
        };
        let auto_fill = match attrs.fill_bits {
            crate::common::FillBits::None => None,
            crate::common::FillBits::Bits(bits) => Some(bits),
            crate::common::FillBits::Auto => {
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
            bit_size += fill_bits;
            let fill_bytes_size = (end_bit - first_bit).div_ceil(8);
            let ident = quote::format_ident!("bondrewd_fill_bits");
            let mut endian = attrs.default_endianess.clone();
            if endian.has_endianness() {
                endian.set_mode(crate::common::EndiannessMode::Standard)
            }
            parsed_fields.push(FieldInfo {
                ident: Box::new(ident.into()),
                attrs: Attributes {
                    bit_range: first_bit..end_bit,
                    endianness: Box::new(endian),
                    reserve: ReserveFieldOption::FakeField,
                    overlap: OverlapOptions::None,
                    capture_id: false,
                },
                ty: DataType::BlockArray {
                    sub_type: Box::new(SubFieldInfo {
                        ty: DataType::Number {
                            size: 1,
                            sign: NumberSignage::Unsigned,
                            type_quote: quote! {u8},
                        },
                    }),
                    length: fill_bytes_size,
                    type_quote: quote! {[u8;#fill_bytes_size]},
                },
            });
        }
        if attrs.default_endianess.is_field_order_reversed() {
            for ref mut field in &mut parsed_fields {
                field.attrs.bit_range = (bit_size - field.attrs.bit_range.end)
                    ..(bit_size - field.attrs.bit_range.start);
            }
            parsed_fields.reverse();
        }
        Ok(parsed_fields)
    }
}
