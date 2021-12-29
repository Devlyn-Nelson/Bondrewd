use proc_macro2::{Literal, Span};
use quote::format_ident;
use syn::parse::Error;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Expr, Ident, Lit, Meta, NestedMeta, Variant};

#[derive(Eq, Debug, Clone)]
pub enum EnumVariantBuilderType {
    UnsignedValue,
    CatchAll,
    CatchPrimitive(Option<Ident>),
    Skip,
}

impl PartialEq for EnumVariantBuilderType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::UnsignedValue => false,
            Self::CatchAll => true,
            Self::CatchPrimitive(name) => {
                if let Self::CatchPrimitive(other_name) = other {
                    true
                } else {
                    false
                }
            }
            Self::Skip => {
                if let Self::Skip = other {
                    true
                } else {
                    false
                }
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct EnumVariantBuilder {
    pub name: Ident,
    pub value: EnumVariantBuilderType,
}

#[derive(Debug)]
pub enum EnumVariantType {
    UnsignedValue(proc_macro2::Literal),
    CatchAll(proc_macro2::Literal),
    CatchPrimitive(Option<Ident>),
    Skip(proc_macro2::Literal),
}
#[derive(Debug)]
pub struct EnumVariant {
    pub name: Ident,
    pub value: EnumVariantType,
}

pub struct EnumInfo {
    pub name: Ident,
    pub variants: Vec<EnumVariant>,
    pub primitive: Ident,
}

enum ParseMetaResult {
    FoundInvalid,
    None,
    InvalidConflict(proc_macro2::Span, Ident),
}

impl EnumInfo {
    fn parse_meta(
        meta: Meta,
        invalid_found: &mut Option<EnumVariantBuilder>,
        primitive_type: &mut Option<Ident>,
        var: &syn::Variant,
    ) -> syn::Result<ParseMetaResult> {
        match meta {
            Meta::NameValue(_) => Ok(ParseMetaResult::None),
            Meta::Path(name_value) => {
                if let Some(name) = name_value.get_ident() {
                    match name.to_string().as_str() {
                        "invalid" => {
                            if let Some(ref name) = invalid_found {
                                return Ok(ParseMetaResult::InvalidConflict(
                                    var.ident.span(),
                                    var.ident.clone(),
                                ));
                            } else {
                                match var.fields {
                                    syn::Fields::Named(ref named) => match named.named.len() {
                                        1 => {
                                            if let syn::Type::Path(ref path) = named.named[0].ty {
                                                if let Some(prim_ident) = path.path.get_ident() {
                                                    if let Some(ref prim_ty) = primitive_type {
                                                        if prim_ident.to_string()
                                                            != prim_ty.to_string()
                                                        {
                                                            return Err(syn::Error::new(var.ident.span(), "primitive type does not match enums defined primitive type"));
                                                        }
                                                    } else {
                                                        let mut invalid = Some(EnumVariantBuilder{
                                                            name: var.ident.clone(),
                                                            value: EnumVariantBuilderType::CatchPrimitive(if let Some(ref name) = named.named.iter().collect::<Vec<&syn::Field>>()[0].ident{
                                                                Some(name.clone())
                                                            }else{
                                                                return Err(syn::Error::new(var.ident.span(), "named value didn't have name"));
                                                            }),
                                                        });
                                                        std::mem::swap(invalid_found, &mut invalid);
                                                    }
                                                }
                                            } else {
                                                return Err(syn::Error::new(var.ident.span(), "catch invalid variants with a field must contain a unsigned primitive"));
                                            }
                                            let mut invalid = Some(EnumVariantBuilder {
                                                name: var.ident.clone(),
                                                value: EnumVariantBuilderType::CatchPrimitive(
                                                    if let Some(ref name) = named
                                                        .named
                                                        .iter()
                                                        .collect::<Vec<&syn::Field>>()[0]
                                                        .ident
                                                    {
                                                        Some(name.clone())
                                                    } else {
                                                        return Err(syn::Error::new(
                                                            var.ident.span(),
                                                            "named value didn't have name",
                                                        ));
                                                    },
                                                ),
                                            });
                                            std::mem::swap(invalid_found, &mut invalid);

                                            Ok(ParseMetaResult::FoundInvalid)
                                        }
                                        _ => {
                                            return Err(syn::Error::new(var.ident.span(), "Invalid Variants must have either no fields or 1 field containing the primitive type the enum will become"));
                                        }
                                    },
                                    syn::Fields::Unnamed(ref unnamed) => {
                                        match unnamed.unnamed.len() {
                                            1 => {
                                                if let syn::Type::Path(ref path) = unnamed.unnamed[0].ty {
                                                    if let Some(prim_ident) = path.path.get_ident(){
                                                        if let Some(ref prim_ty) = primitive_type {
                                                            if prim_ident.to_string() != prim_ty.to_string() {
                                                                return Err(syn::Error::new(var.ident.span(), "primitive type does not match enums defined primitive type"));
                                                            }
                                                        }else{
                                                            let mut invalid = Some(EnumVariantBuilder{
                                                                name: var.ident.clone(),
                                                                value: EnumVariantBuilderType::CatchPrimitive(None)
                                                            });
                                                            std::mem::swap(invalid_found,&mut invalid);
                                                        }
                                                    }
                                                }else{
                                                    return Err(syn::Error::new(var.ident.span(), "catch invalid variants with a field must contain a unsigned primitive"));
                                                }
                                                let mut invalid = Some(EnumVariantBuilder{
                                                    name: var.ident.clone(),
                                                    value: EnumVariantBuilderType::CatchPrimitive(None)
                                                });
                                                std::mem::swap(invalid_found,&mut invalid);
                                                Ok(ParseMetaResult::FoundInvalid)
                                            }
                                            _ => {
                                                return Err(syn::Error::new(var.ident.span(), "Variants must have either no fields or 1 field containing the primitive type the enum will become"))
                                            }
                                        }
                                    }
                                    syn::Fields::Unit => {
                                        let mut invalid = Some(EnumVariantBuilder {
                                            name: var.ident.clone(),
                                            value: EnumVariantBuilderType::CatchAll,
                                        });
                                        std::mem::swap(invalid_found, &mut invalid);
                                        Ok(ParseMetaResult::FoundInvalid)
                                    }
                                }
                            }
                        }
                        _ => Ok(ParseMetaResult::None),
                    }
                } else {
                    Ok(ParseMetaResult::None)
                }
            }
            Meta::List(meta_list) => {
                if meta_list.path.is_ident("bondrewd_enum") {
                    for nested_meta in meta_list.nested {
                        match nested_meta {
                            NestedMeta::Meta(meta) => {
                                match Self::parse_meta(meta, invalid_found, primitive_type, var)? {
                                    ParseMetaResult::FoundInvalid => {
                                        return Ok(ParseMetaResult::FoundInvalid)
                                    }
                                    ParseMetaResult::None => {}
                                    ParseMetaResult::InvalidConflict(span, name) => {
                                        return Ok(ParseMetaResult::InvalidConflict(span, name))
                                    }
                                }
                            }
                            NestedMeta::Lit(_) => {}
                        }
                    }
                    Ok(ParseMetaResult::None)
                } else {
                    Ok(ParseMetaResult::None)
                }
            }
        }
    }

    // Parses the Expression, looking for a literal number expression
    fn parse_lit_discriminant_expr(input: &Expr) -> syn::Result<usize> {
        match input {
            Expr::Lit(ref lit) => match lit.lit {
                Lit::Int(ref i) => Ok(i.base10_parse()?),
                _ => Err(syn::Error::new(
                    input.span(),
                    "Non-integer literals for custom discriminant are illegal.",
                )),
            },
            _ => Err(syn::Error::new(
                input.span(),
                "non-literal expressions for custom discriminant are illegal.",
            )),
        }
    }

    fn parse_attrs(
        attrs: &Vec<Attribute>,
        var: &Variant,
        primitive_type: &mut Option<Ident>,
        invalid_found: &mut Option<EnumVariantBuilder>,
        i: &usize,
    ) -> syn::Result<bool> {
        let mut temp: Option<Ident> = None;
        let mut temp_invalid = invalid_found.clone();
        for attr in attrs {
            match Self::parse_meta(attr.parse_meta()?, &mut temp_invalid, &mut temp, &var)? {
                ParseMetaResult::FoundInvalid => {
                    std::mem::swap(&mut temp, primitive_type);
                    std::mem::swap(&mut temp_invalid, invalid_found);
                    return Ok(true);
                }
                ParseMetaResult::None => {}
                ParseMetaResult::InvalidConflict(span, name) => {
                    return Err(syn::Error::new(
                        span,
                        format!("Invalid already found [{}]", name),
                    ));
                }
            }
        }
        Ok(false)
    }

    pub fn parse(input: &DeriveInput) -> syn::Result<Self> {
        // get the struct, error out if not a struct
        let data = match input.data {
            syn::Data::Enum(ref data) => data,
            _ => {
                return Err(Error::new(Span::call_site(), "input must be an enum"));
            }
        };
        let mut primitive_type: Option<Ident> = None;
        for attr in &input.attrs {
            match attr.parse_meta()? {
                Meta::NameValue(_) => {}
                Meta::Path(_) => {}
                Meta::List(meta_list) => {
                    if meta_list.path.is_ident("bondrewd_enum") {
                        for nested_meta in meta_list.nested {
                            match nested_meta {
                                NestedMeta::Meta(meta) => match meta {
                                    Meta::Path(path) => {
                                        if path.is_ident("u8") {
                                            primitive_type = Some(quote::format_ident!("u8"));
                                            break;
                                        } else {
                                            return Err(syn::Error::new(
                                                input.ident.span(),
                                                "the only supported enum type is u8 currently",
                                            ));
                                        }
                                    }
                                    _ => {}
                                },
                                NestedMeta::Lit(_) => {}
                            }
                        }
                    }
                }
            }
        }
        // get the list of fields in syn form, error out if unit struct (because they have no data, and
        // data packing/analysis don't seem necessary)
        let mut literal_variants: std::collections::BTreeMap<usize, EnumVariant> =
            Default::default();
        let mut unknown_variants: std::collections::VecDeque<EnumVariantBuilder> =
            Default::default();
        let mut invalid_found: Option<EnumVariantBuilder> = None;
        let mut out_of_order_indices: usize = 0;
        let last_variant = data.variants.len() - 1;
        for (var, i) in data.variants.iter().zip(0..data.variants.len()) {
            match var.fields {
                syn::Fields::Named(ref named) => {
                    match named.named.len() {
                        1 => {
                            if let syn::Type::Path(ref path) = named.named[0].ty {
                                if let Some(prim_ident) = path.path.get_ident(){
                                    if let Some(ref prim_ty) = primitive_type {
                                        if prim_ident.to_string() != prim_ty.to_string() {
                                            return Err(syn::Error::new(var.ident.span(), "primitive type does not match enums defined primitive type"));
                                        }
                                    }else{
                                        primitive_type = Some(prim_ident.clone());
                                    }
                                }
                            }else{
                                return Err(syn::Error::new(var.ident.span(), "catch invalid variants with a field must contain a unsigned primitive"));
                            }
                            if let Some(conflict) = invalid_found {
                                return Err(syn::Error::new(var.ident.span(), format!("conflicting Invalid Variant named [{}]", conflict.name)));
                            }
                            invalid_found = Some(EnumVariantBuilder{
                                name: var.ident.clone(),
                                value: EnumVariantBuilderType::CatchPrimitive(if let Some(ref name) = named.named.iter().collect::<Vec<&syn::Field>>()[0].ident{
                                    Some(name.clone())
                                }else{
                                    return Err(syn::Error::new(var.ident.span(), "named value didn't have name"));
                                })
                            });
                            unknown_variants.push_back(EnumVariantBuilder{
                                name: var.ident.clone(),
                                value: EnumVariantBuilderType::Skip,
                            });
                        }
                        _ => {
                            return Err(syn::Error::new(var.ident.span(), "Variants must have either no fields or 1 field containing the primitive type the enum will become"))
                        }
                    }
                }
                syn::Fields::Unnamed(ref unnamed) => {
                    match unnamed.unnamed.len() {
                        1 => {
                            if let syn::Type::Path(ref path) = unnamed.unnamed[0].ty {
                                if let Some(prim_ident) = path.path.get_ident(){
                                    if let Some(ref prim_ty) = primitive_type {
                                        if prim_ident.to_string() != prim_ty.to_string() {
                                            return Err(syn::Error::new(var.ident.span(), "primitive type does not match enums defined primitive type"));
                                        }
                                    }else{
                                        primitive_type = Some(prim_ident.clone());
                                    }
                                }
                            }else{
                                return Err(syn::Error::new(var.ident.span(), "catch invalid variants with a field must contain a unsigned primitive"));
                            }
                            if let Some(conflict) = invalid_found {
                                return Err(syn::Error::new(var.ident.span(), format!("conflicting Invalid Variant named [{}]", conflict.name)));
                            }
                            invalid_found = Some(EnumVariantBuilder{
                                name: var.ident.clone(),
                                value: EnumVariantBuilderType::CatchPrimitive(None)
                            });
                            unknown_variants.push_back(EnumVariantBuilder{
                                name: var.ident.clone(),
                                value: EnumVariantBuilderType::Skip,
                            });
                        }
                        _ => {
                            return Err(syn::Error::new(var.ident.span(), "Variants must have either no fields or 1 field containing the primitive type the enum will become"))
                        }
                    }
                }
                _ => {
                    // Check for an invalid already existing, and if this is the last variant in the variants
                    if invalid_found.is_none() && last_variant == i {
                        invalid_found = Some(EnumVariantBuilder{
                            name: var.ident.clone(),
                            value: EnumVariantBuilderType::CatchAll,
                        });
                        if let Some((_, ref discriminant)) = var.discriminant {
                            // Parse the discriminant and validate its able to be used
                            let discriminant_val = Self::parse_lit_discriminant_expr(discriminant)?;
                            if let Some(oh_no) = literal_variants.insert(discriminant_val, EnumVariant{
                                value: EnumVariantType::UnsignedValue(Literal::usize_unsuffixed(i)),
                                name: var.ident.clone(),
                            }) {
                                return Err(syn::Error::new(var.ident.span(), "Literal Values conflict"));
                            }
                            literal_variants.insert(i.clone(), EnumVariant{
                                name: var.ident.clone(),
                                value: EnumVariantType::Skip(Literal::usize_unsuffixed(i)),
                            });
                        }else{
                            unknown_variants.push_back(EnumVariantBuilder{
                                name: var.ident.clone(),
                                value: EnumVariantBuilderType::Skip,
                            });
                        }
                    } else {
                        // This is one of the possible variants to use, check for a custom discriminant
                        if let Some((_, ref discriminant)) = var.discriminant {
                            // Parse the discriminant and validate its able to be used
                            let discriminant_val = Self::parse_lit_discriminant_expr(discriminant)?;
                            if let Some(oh_no) = literal_variants.insert(discriminant_val, EnumVariant{
                                value: EnumVariantType::Skip(Literal::usize_unsuffixed(i)),
                                name: var.ident.clone(),
                            }) {
                                return Err(syn::Error::new(var.ident.span(), "Literal Values conflict"));
                            }
                        } else if invalid_found.is_none() {
                            if Self::parse_attrs(&var.attrs, &var, &mut primitive_type, &mut invalid_found, &i)? {
                                unknown_variants.push_back(EnumVariantBuilder{
                                    name: var.ident.clone(),
                                    value: EnumVariantBuilderType::Skip,
                                });
                            }
                        } else {
                            // This is a simple usage of a bunch of unit variants in a row
                            unknown_variants.push_back(EnumVariantBuilder {
                                value: EnumVariantBuilderType::UnsignedValue,
                                name: var.ident.clone(),
                            });
                        }
                    }
                }
            }
        }
        let mut skipped: Option<Literal> = None;
        let mut variants: Vec<EnumVariant> = Default::default();
        for i in 0..=last_variant {
            if let Some(enum_var) = literal_variants.remove(&i) {
                if let EnumVariantType::Skip(_) = enum_var.value {
                    if skipped.is_some() {
                        // CHECK if needed
                        return Err(syn::Error::new(
                            enum_var.name.span(),
                            "two skips. please open issue for this",
                        ));
                    }
                    skipped = Some(Literal::usize_unsuffixed(i));
                } else {
                    variants.push(enum_var);
                }
                continue;
            }
            if let Some(unknown_variant) = unknown_variants.pop_front() {
                match unknown_variant.value {
                    EnumVariantBuilderType::CatchAll => {
                        variants.push(EnumVariant {
                            name: unknown_variant.name,
                            value: EnumVariantType::CatchAll(Literal::usize_unsuffixed(i)),
                        });
                        continue;
                    }
                    EnumVariantBuilderType::CatchPrimitive(name) => {
                        variants.push(EnumVariant {
                            name: unknown_variant.name,
                            value: EnumVariantType::CatchPrimitive(name),
                        });
                        continue;
                    }
                    EnumVariantBuilderType::Skip => {
                        skipped = Some(Literal::usize_unsuffixed(i));
                    }
                    EnumVariantBuilderType::UnsignedValue => {
                        variants.push(EnumVariant {
                            name: unknown_variant.name,
                            value: EnumVariantType::UnsignedValue(Literal::usize_unsuffixed(i)),
                        });
                        continue;
                    }
                }
            }
            if let Some(ref invalid) = invalid_found {
                if let Some(ref index) = skipped {
                    if i == last_variant {
                        variants.push(EnumVariant {
                            value: match invalid.value {
                                EnumVariantBuilderType::CatchAll => {
                                    EnumVariantType::CatchAll(index.clone())
                                }
                                EnumVariantBuilderType::CatchPrimitive(ref name) => {
                                    EnumVariantType::CatchPrimitive(name.clone())
                                }
                                _ => {
                                    return Err(syn::Error::new(
                                        invalid.name.span(),
                                        "Invalid found is not an invalid type, please open issue.",
                                    ));
                                }
                            },
                            name: invalid.name.clone(),
                        });
                    } else {
                        return Err(syn::Error::new(
                            input.span(),
                            "Invalid was not last variant, please open issue.",
                        ));
                    }
                } else {
                    return Err(syn::Error::new(
                        input.span(),
                        "Invalid found but skip never takes place, please open issue.",
                    ));
                }
            } else {
                return Err(syn::Error::new(input.span(), "No Invalid found"));
            }
        }

        let info = EnumInfo {
            name: input.ident.clone(),
            variants,
            primitive: if let Some(prim) = primitive_type {
                if prim.to_string().as_str() == "u8" {
                    prim
                } else {
                    return Err(syn::Error::new(
                        input.ident.span(),
                        "primitive_type must be u8 for now",
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    input.ident.span(),
                    "add #[bondrewd_enum(u8)] as struct attribute to avoid problems caused by future changed please.",
                ));
            },
        };

        Ok(info)
    }
}
