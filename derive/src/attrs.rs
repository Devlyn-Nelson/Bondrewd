//! Parsing of `#[bondrewd(...)]` attributes.

use proc_macro2::Span;
use syn::meta::ParseNestedMeta;
use syn::{Attribute, Error, LitInt, LitStr, Result, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endian {
    /// Most significant bit first, logical bit 0 is the left most bit of byte 0.
    Big,
    /// Little endian bytes, each byte most significant bit first.
    Little,
    /// Aligned little endian: the buffer is one little endian integer, values are
    /// stored least significant bit first.
    Aligned,
}

impl Endian {
    fn parse(lit: &LitStr) -> Result<Self> {
        match lit.value().to_ascii_lowercase().as_str() {
            "be" | "big" | "msb" | "big_endian" => Ok(Endian::Big),
            "le" | "little" | "lsb" | "little_endian" => Ok(Endian::Little),
            "ale" | "aligned" | "aligned_little_endian" => Ok(Endian::Aligned),
            other => Err(Error::new(
                lit.span(),
                format!("unknown endianness \"{other}\", expected \"be\", \"le\" or \"ale\""),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Fill {
    /// Round the bit size up to a whole byte.
    ToByte,
    /// Pad the bit size to an exact amount of bits.
    Bits(usize),
}

/// Attributes valid on a struct or enum.
#[derive(Default)]
pub struct TypeAttrs {
    pub endianness: Option<Endian>,
    pub reverse: bool,
    pub back: bool,
    pub enforce_bits: Option<(usize, Span)>,
    pub enforce_bytes: Option<(usize, Span)>,
    pub enforce_full_bytes: Option<Span>,
    pub fill: Option<Fill>,
    pub id_bits: Option<(usize, Span)>,
    pub crate_path: Option<syn::Path>,
    /// Write the generated code to `target/bondrewd_dump/`.
    pub dump: Option<Span>,
}

/// Attributes valid on a variant.
#[derive(Default)]
pub struct VariantAttrs {
    pub id: Option<(u128, Span)>,
    pub invalid: bool,
}

/// Attributes valid on a field.
#[derive(Default)]
pub struct FieldAttrs {
    pub bit_length: Option<(usize, Span)>,
    pub element_bits: Option<(usize, Span)>,
    pub block_bits: Option<(usize, Span)>,
    pub endianness: Option<(Endian, Span)>,
    pub bits: Option<(usize, usize, Span)>,
    pub reserve: bool,
    pub redundant: bool,
    pub capture_id: Option<Span>,
}

fn int<T: std::str::FromStr>(meta: &ParseNestedMeta) -> Result<(T, Span)>
where
    T::Err: std::fmt::Display,
{
    let lit: LitInt = meta.value()?.parse()?;
    Ok((lit.base10_parse()?, lit.span()))
}

fn bytes_to_bits((v, s): (usize, Span)) -> (usize, Span) {
    (v * 8, s)
}

fn set<T>(slot: &mut Option<T>, value: T, meta: &ParseNestedMeta) -> Result<()> {
    if slot.is_some() {
        return Err(meta.error("this property was already defined"));
    }
    *slot = Some(value);
    Ok(())
}

fn each<F: FnMut(ParseNestedMeta) -> Result<()>>(attrs: &[Attribute], mut f: F) -> Result<()> {
    for attr in attrs.iter().filter(|a| a.path().is_ident("bondrewd")) {
        attr.parse_nested_meta(&mut f)?;
    }
    Ok(())
}

pub fn type_attrs(attrs: &[Attribute], is_enum: bool) -> Result<TypeAttrs> {
    let mut out = TypeAttrs::default();
    each(attrs, |meta| {
        let p = &meta.path;
        if p.is_ident("endianness") || p.is_ident("default_endianness") {
            let lit: LitStr = meta.value()?.parse()?;
            set(&mut out.endianness, Endian::parse(&lit)?, &meta)
        } else if p.is_ident("reverse") {
            out.reverse = true;
            Ok(())
        } else if p.is_ident("bit_traversal") {
            let lit: LitStr = meta.value()?.parse()?;
            out.back = match lit.value().as_str() {
                "front" => false,
                "back" => true,
                _ => return Err(Error::new(lit.span(), "expected \"front\" or \"back\"")),
            };
            Ok(())
        } else if p.is_ident("enforce_bits") {
            set(&mut out.enforce_bits, int(&meta)?, &meta)
        } else if p.is_ident("enforce_bytes") {
            set(&mut out.enforce_bytes, int(&meta)?, &meta)
        } else if p.is_ident("enforce_full_bytes") {
            set(&mut out.enforce_full_bytes, p.get_ident().unwrap().span(), &meta)
        } else if p.is_ident("fill_bits") {
            let fill = if meta.input.peek(Token![=]) {
                Fill::Bits(int(&meta)?.0)
            } else {
                Fill::ToByte
            };
            set(&mut out.fill, fill, &meta)
        } else if p.is_ident("fill_bytes") {
            set(&mut out.fill, Fill::Bits(int::<usize>(&meta)?.0 * 8), &meta)
        } else if p.is_ident("dump") {
            out.dump = Some(p.get_ident().unwrap().span());
            Ok(())
        } else if p.is_ident("crate_path") {
            let lit: LitStr = meta.value()?.parse()?;
            set(&mut out.crate_path, lit.parse()?, &meta)
        } else if is_enum && p.is_ident("id_bit_length") {
            set(&mut out.id_bits, int(&meta)?, &meta)
        } else if is_enum && p.is_ident("id_byte_length") {
            set(&mut out.id_bits, bytes_to_bits(int(&meta)?), &meta)
        } else {
            Err(meta.error("unknown bondrewd attribute for this item"))
        }
    })?;
    Ok(out)
}

pub fn variant_attrs(attrs: &[Attribute]) -> Result<VariantAttrs> {
    let mut out = VariantAttrs::default();
    each(attrs, |meta| {
        if meta.path.is_ident("id") || meta.path.is_ident("variant_id") {
            set(&mut out.id, int(&meta)?, &meta)
        } else if meta.path.is_ident("invalid") {
            out.invalid = true;
            Ok(())
        } else {
            Err(meta.error("unknown bondrewd attribute for a variant"))
        }
    })?;
    Ok(out)
}

pub fn field_attrs(attrs: &[Attribute]) -> Result<FieldAttrs> {
    let mut out = FieldAttrs::default();
    each(attrs, |meta| {
        let p = &meta.path;
        if p.is_ident("bit_length") {
            set(&mut out.bit_length, int(&meta)?, &meta)
        } else if p.is_ident("byte_length") {
            set(&mut out.bit_length, bytes_to_bits(int(&meta)?), &meta)
        } else if p.is_ident("element_bit_length") {
            set(&mut out.element_bits, int(&meta)?, &meta)
        } else if p.is_ident("element_byte_length") {
            set(&mut out.element_bits, bytes_to_bits(int(&meta)?), &meta)
        } else if p.is_ident("block_bit_length") {
            set(&mut out.block_bits, int(&meta)?, &meta)
        } else if p.is_ident("block_byte_length") {
            set(&mut out.block_bits, bytes_to_bits(int(&meta)?), &meta)
        } else if p.is_ident("endianness") {
            let lit: LitStr = meta.value()?.parse()?;
            set(&mut out.endianness, (Endian::parse(&lit)?, lit.span()), &meta)
        } else if p.is_ident("bits") {
            let lit: LitStr = meta.value()?.parse()?;
            let value = lit.value();
            let parsed = value.split_once("..").and_then(|(a, b)| {
                Some((a.trim().parse::<usize>().ok()?, b.trim().parse::<usize>().ok()?))
            });
            match parsed {
                Some((a, b)) if a < b => set(&mut out.bits, (a, b, lit.span()), &meta),
                _ => Err(Error::new(lit.span(), "expected a range like \"0..8\"")),
            }
        } else if p.is_ident("reserve") {
            out.reserve = true;
            Ok(())
        } else if p.is_ident("redundant") || p.is_ident("overlapping_bits") {
            out.redundant = true;
            Ok(())
        } else if p.is_ident("capture_id") {
            out.capture_id = Some(p.get_ident().unwrap().span());
            Ok(())
        } else {
            Err(meta.error("unknown bondrewd attribute for a field"))
        }
    })?;
    Ok(out)
}
