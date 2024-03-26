pub mod r#enum;
pub mod field;
pub mod object;
#[cfg(feature = "setters")]
pub mod struct_fns;

use std::ops::Range;
use syn::{Expr, Ident, Lit, LitInt, LitStr};

pub(crate) fn get_lit_str<'a>(
    expr: &'a Expr,
    ident: &Ident,
    example: Option<&str>,
) -> syn::Result<&'a LitStr> {
    let example = if let Some(ex) = example {
        format!("example: `{ex}`")
    } else {
        String::new()
    };
    if let Expr::Lit(ref lit) = expr {
        if let Lit::Str(ref val) = lit.lit {
            Ok(val)
        } else {
            return Err(syn::Error::new(
                ident.span(),
                format!("{ident} requires a integer literal. {example}"),
            ));
        }
    } else {
        return Err(syn::Error::new(
            ident.span(),
            format!("{ident} requires a integer literal. {example}"),
        ));
    }
}

pub(crate) fn get_lit_int<'a>(
    expr: &'a Expr,
    ident: &Ident,
    example: Option<&str>,
) -> syn::Result<&'a LitInt> {
    let example = if let Some(ex) = example {
        format!("example: `{ex}`")
    } else {
        String::new()
    };
    if let Expr::Lit(ref lit) = expr {
        if let Lit::Int(ref val) = lit.lit {
            Ok(val)
        } else {
            return Err(syn::Error::new(
                ident.span(),
                format!("{ident} requires a string literal. {example}"),
            ));
        }
    } else {
        return Err(syn::Error::new(
            ident.span(),
            format!("{ident} requires a string literal. {example}"),
        ));
    }
}

pub(crate) fn get_lit_range<'a>(
    expr: &'a Expr,
    ident: &Ident,
) -> syn::Result<Option<Range<usize>>> {
    if let Expr::Range(ref lit) = expr {
        let start = if let Some(ref v) = lit.start {
            if let Expr::Lit(ref el) = v.as_ref() {
                if let Lit::Int(ref i) = el.lit {
                    i.base10_parse()?
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("start of range must be an integer."),
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("start of range must be an integer literal."),
                ));
            }
        } else {
            return Err(syn::Error::new(
                ident.span(),
                "range for bits must define a start",
            ));
        };
        let end = if let Some(ref v) = lit.end {
            if let Expr::Lit(ref el) = v.as_ref() {
                if let Lit::Int(ref i) = el.lit {
                    i.base10_parse()?
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("end of range must be an integer."),
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("end of range must be an integer literal."),
                ));
            }
        } else {
            return Err(syn::Error::new(
                ident.span(),
                "range for bits must define a end",
            ));
        };
        Ok(Some(match lit.limits {
            syn::RangeLimits::HalfOpen(_) => start..end,
            syn::RangeLimits::Closed(_) => start..end + 1,
        }))
    } else {
        Ok(None)
    }
}
