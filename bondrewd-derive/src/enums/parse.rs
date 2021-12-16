use proc_macro2::Span;
use syn::parse::Error;
use syn::{DeriveInput, Ident, Meta, NestedMeta};

#[derive(Eq, Debug)]
pub enum EnumVariantType {
    UnsignedValue(u8),
    CatchAll(u8),
    CatchPrimitive,
}

impl PartialEq for EnumVariantType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::UnsignedValue(ref value) => {
                if let Self::UnsignedValue(ref other_value) = other {
                    value == other_value
                } else {
                    false
                }
            }
            Self::CatchAll(_) => {
                if let Self::CatchAll(_) = other {
                    true
                } else {
                    false
                }
            }
            Self::CatchPrimitive => {
                if let Self::CatchPrimitive = other {
                    true
                } else {
                    false
                }
            }
        }
    }
}

pub struct EnumVariant {
    pub name: Ident,
    pub value: EnumVariantType,
}

pub struct EnumInfo {
    pub name: Ident,
    pub variants: Vec<EnumVariant>,
    pub primitive: Ident,
}

impl EnumInfo {
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
        let mut variants: Vec<EnumVariant> = Default::default();
        let mut invalid_found: Option<usize> = None;
        let last_variant = data.variants.len() - 1;
        for (var, i) in data.variants.iter().zip(0u8..data.variants.len() as u8) {
            let mut finished = false;
            for attr in &var.attrs {
                match attr.parse_meta()? {
                    Meta::NameValue(ref name_value) => {
                        if let Some(name) = name_value.path.get_ident() {
                            match name.to_string().as_str() {
                                /*"value" => {
                                    if let Lit::Int(val) = name_value.lit {
                                        match val.base10_parse::<u16>() {
                                            Ok(value) => {
                                                variants.push(EnumVariant {
                                                    value: EnumVariantType::UnsignedValue(value),
                                                    name: var.ident.clone(),
                                                });
                                                break;
                                            }
                                            Err(err) => {
                                                return Err(Error::new(
                                                    var.ident.span(),
                                                    format!("struct_size must provided a number that can be parsed as a u16 or u8 [{}]", err),
                                                ));
                                            }
                                        }
                                    } else {
                                        return Err(Error::new(
                                            var.ident.span(),
                                            format!(
                                                "defining a struct_size requires a Int Literal"
                                            ),
                                        ));
                                    }
                                }*/
                                _ => {}
                            }
                        }
                    }
                    Meta::Path(name_value) => {
                        if let Some(name) = name_value.get_ident() {
                            match name.to_string().as_str() {
                                "invalid" => {
                                    if let Some(index) = invalid_found {
                                        return Err(syn::Error::new(
                                            var.ident.span(),
                                            format!(
                                                "Invalid already found [{}]",
                                                variants[index].name
                                            ),
                                        ));
                                    } else {
                                        match var.fields {
                                            syn::Fields::Named(ref named) => {
                                                match named.named.len() {
                                                    1 => {
                                                        if let syn::Type::Path(ref path) =
                                                            named.named[0].ty
                                                        {
                                                            if let Some(prim_ident) =
                                                                path.path.get_ident()
                                                            {
                                                                if let Some(ref prim_ty) =
                                                                    primitive_type
                                                                {
                                                                    if prim_ident.to_string()
                                                                        != prim_ty.to_string()
                                                                    {
                                                                        return Err(syn::Error::new(var.ident.span(), "primitive type does not match enums defined primitive type"));
                                                                    }
                                                                } else {
                                                                    primitive_type =
                                                                        Some(prim_ident.clone());
                                                                }
                                                            }
                                                        } else {
                                                            return Err(syn::Error::new(var.ident.span(), "catch invalid variants with a field must contain a unsigned primitive"));
                                                        }
                                                        finished = true;
                                                        invalid_found = Some(variants.len());
                                                        variants.push(EnumVariant {
                                                            name: var.ident.clone(),
                                                            value: EnumVariantType::CatchPrimitive,
                                                        });
                                                    }
                                                    _ => {
                                                        return Err(syn::Error::new(var.ident.span(), "Invalid Variants must have either no fields or 1 field containing the primitive type the enum will become"));
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
                                                        invalid_found = Some(variants.len());
                                                        finished = true;
                                                        variants.push(EnumVariant {
                                                            name: var.ident.clone(),
                                                            value: EnumVariantType::CatchPrimitive,
                                                        });
                                                    }
                                                    _ => {
                                                        return Err(syn::Error::new(var.ident.span(), "Variants must have either no fields or 1 field containing the primitive type the enum will become"))
                                                    }
                                                }
                                            }
                                            syn::Fields::Unit => {
                                                finished = true;
                                                invalid_found = Some(variants.len());
                                                variants.push(EnumVariant {
                                                    name: var.ident.clone(),
                                                    value: EnumVariantType::CatchAll(i),
                                                })
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Meta::List(_meta_list) => {
                        //TODO impl nested attributes
                    }
                }
            }
            if finished {
                break;
            }
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
                            invalid_found = Some(variants.len());
                            variants.push(EnumVariant {
                                name: var.ident.clone(),
                                value: EnumVariantType::CatchPrimitive
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
                            invalid_found = Some(variants.len());
                            variants.push(EnumVariant {
                                name: var.ident.clone(),
                                value: EnumVariantType::CatchPrimitive
                            });
                        }
                        _ => {
                            return Err(syn::Error::new(var.ident.span(), "Variants must have either no fields or 1 field containing the primitive type the enum will become"))
                        }
                    }
                }
                _ => {
                    if invalid_found.is_none() && last_variant == i as usize{
                        variants.push(EnumVariant {
                            name: var.ident.clone(),
                            value: EnumVariantType::CatchAll(i),
                        })
                    }else{
                        variants.push(EnumVariant {
                            value: EnumVariantType::UnsignedValue(i),
                            name: var.ident.clone(),
                        });
                    }
                }
            }
        }

        let amount_of_variants = variants.len();
        for i in 0..amount_of_variants {
            for ii in i + 1..amount_of_variants {
                if variants[i].value == variants[ii].value {
                    return Err(syn::Error::new(
                        variants[ii].name.span(),
                        format!("Field has same value as {}", variants[i].name),
                    ));
                }
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
