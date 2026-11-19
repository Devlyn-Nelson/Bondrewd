//! Field model, layout solving and the logical -> physical bit mapping.
//!
//! Every field is reduced to "leaves". A leaf is a value that is converted to a small
//! byte array (its *value bytes*) plus a rule mapping each logical bit of the field to a
//! bit of those value bytes. The struct layout maps each logical bit to a bit of the
//! output buffer. Combining both gives, per output byte, a mask and a shift, which is
//! all the generated code needs. Every endianness, nesting and array kind goes through
//! this single path.

use std::collections::BTreeMap;

use proc_macro2::Span;
use syn::{Error, Expr, Lit, Result, Type};

use crate::attrs::{Endian, FieldAttrs};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prim {
    Unsigned(usize),
    Signed(usize),
    Bool,
    F32,
    F64,
    Char,
}

impl Prim {
    pub fn from_type(ty: &Type) -> Option<Prim> {
        let Type::Path(tp) = ty else { return None };
        if tp.qself.is_some() || tp.path.segments.len() != 1 {
            return None;
        }
        let seg = &tp.path.segments[0];
        if !seg.arguments.is_none() {
            return None;
        }
        Some(match seg.ident.to_string().as_str() {
            "u8" => Prim::Unsigned(1),
            "u16" => Prim::Unsigned(2),
            "u32" => Prim::Unsigned(4),
            "u64" => Prim::Unsigned(8),
            "u128" => Prim::Unsigned(16),
            "i8" => Prim::Signed(1),
            "i16" => Prim::Signed(2),
            "i32" => Prim::Signed(4),
            "i64" => Prim::Signed(8),
            "i128" => Prim::Signed(16),
            "bool" => Prim::Bool,
            "f32" => Prim::F32,
            "f64" => Prim::F64,
            "char" => Prim::Char,
            _ => return None,
        })
    }

    /// Bytes of the integer representation used while packing.
    pub fn repr_bytes(self) -> usize {
        match self {
            Prim::Unsigned(n) | Prim::Signed(n) => n,
            Prim::Bool => 1,
            Prim::F32 | Prim::Char => 4,
            Prim::F64 => 8,
        }
    }

    pub fn default_bits(self) -> usize {
        match self {
            Prim::Bool => 1,
            other => other.repr_bytes() * 8,
        }
    }

    pub fn is_int(self) -> bool {
        matches!(self, Prim::Unsigned(_) | Prim::Signed(_))
    }

    fn check_bits(self, bits: usize, span: Span) -> Result<()> {
        let max = self.repr_bytes() * 8;
        if bits == 0 {
            return Err(Error::new(span, "a field must use at least 1 bit"));
        }
        if bits > max {
            return Err(Error::new(span, format!("this type can hold at most {max} bits")));
        }
        if matches!(self, Prim::F32 | Prim::F64) && bits != max {
            return Err(Error::new(span, format!("floating point fields must use exactly {max} bits")));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub enum Kind {
    Prim(Prim),
    /// A type implementing `Bitfields<{bits.div_ceil(8)}>`.
    Nested,
    /// Each element is its own bitfield of `elem_bits` bits.
    Element { len: usize, elem_ty: Box<Type>, elem: Box<Kind>, elem_bits: usize },
    /// The whole array is one block of bits: elements packed in array order, each in the
    /// field's endianness, with truncation removing the top bits of element 0 first.
    Block { len: usize, elem: Prim },
}

pub struct Field {
    /// Name used for generated function names.
    pub name: String,
    /// Member used to access the value (`one` or `0`).
    pub member: syn::Member,
    pub ty: Type,
    pub kind: Kind,
    pub endian: Endian,
    pub start: usize,
    pub bits: usize,
    pub reserve: bool,
    pub redundant: bool,
    pub capture_id: bool,
    pub span: Span,
}

impl Field {
    pub fn end(&self) -> usize {
        self.start + self.bits
    }
}

fn array_len(expr: &Expr) -> Result<usize> {
    match expr {
        Expr::Lit(l) => match &l.lit {
            Lit::Int(i) => i.base10_parse(),
            _ => Err(Error::new_spanned(expr, "array length must be an integer literal")),
        },
        Expr::Group(g) => array_len(&g.expr),
        Expr::Paren(p) => array_len(&p.expr),
        _ => Err(Error::new_spanned(expr, "array length must be an integer literal")),
    }
}

fn strip(ty: &Type) -> &Type {
    match ty {
        Type::Group(g) => strip(&g.elem),
        Type::Paren(p) => strip(&p.elem),
        other => other,
    }
}

/// Kind of an element inside an element array, given its bit length.
fn element_kind(ty: &Type, bits: usize, span: Span) -> Result<Kind> {
    let ty = strip(ty);
    if let Some(p) = Prim::from_type(ty) {
        p.check_bits(bits, span)?;
        return Ok(Kind::Prim(p));
    }
    if let Type::Array(arr) = ty {
        let len = array_len(&arr.len)?;
        let elem = Prim::from_type(&arr.elem)
            .filter(|p| p.is_int())
            .ok_or_else(|| Error::new_spanned(&arr.elem, "nested arrays must contain integers"))?;
        if bits > len * elem.repr_bytes() * 8 {
            return Err(Error::new(span, "element bit length is larger than the inner array"));
        }
        return Ok(Kind::Block { len, elem });
    }
    if bits == 0 {
        return Err(Error::new(span, "a field must use at least 1 bit"));
    }
    Ok(Kind::Nested)
}

fn default_bits_of(ty: &Type) -> Option<usize> {
    let ty = strip(ty);
    if let Some(p) = Prim::from_type(ty) {
        return Some(p.default_bits());
    }
    if let Type::Array(arr) = ty {
        let len = array_len(&arr.len).ok()?;
        return Some(len * default_bits_of(&arr.elem)?);
    }
    None
}

/// Works out the kind and total bit length of a field.
pub fn classify(ty: &Type, attrs: &FieldAttrs, span: Span) -> Result<(Kind, usize)> {
    let ty = strip(ty);
    if let Type::Array(arr) = ty {
        let len = array_len(&arr.len)?;
        if attrs.element_bits.is_some() && attrs.block_bits.is_some() {
            return Err(Error::new(span, "a field can't be both an element and a block array"));
        }
        let block = attrs.block_bits.or(attrs.bit_length.filter(|_| attrs.element_bits.is_none()));
        if let Some((bits, bspan)) = block {
            let elem = Prim::from_type(&arr.elem).filter(|p| p.is_int()).ok_or_else(|| {
                Error::new_spanned(&arr.elem, "block arrays must contain integer primitives")
            })?;
            let max = len * elem.repr_bytes() * 8;
            if bits == 0 || bits > max {
                return Err(Error::new(bspan, format!("block bit length must be between 1 and {max}")));
            }
            return Ok((Kind::Block { len, elem }, bits));
        }
        let (elem_bits, espan) = match attrs.element_bits {
            Some(v) => v,
            None => (
                default_bits_of(&arr.elem).ok_or_else(|| {
                    Error::new(
                        span,
                        "arrays of non-primitive types need `element_bit_length` or `element_byte_length`",
                    )
                })?,
                span,
            ),
        };
        let elem = element_kind(&arr.elem, elem_bits, espan)?;
        return Ok((
            Kind::Element { len, elem_ty: arr.elem.clone(), elem: Box::new(elem), elem_bits },
            len * elem_bits,
        ));
    }
    if attrs.element_bits.is_some() || attrs.block_bits.is_some() {
        return Err(Error::new(span, "array attributes used on a non-array field"));
    }
    if let Some(p) = Prim::from_type(ty) {
        let (bits, bspan) = attrs.bit_length.unwrap_or((p.default_bits(), span));
        p.check_bits(bits, bspan)?;
        return Ok((Kind::Prim(p), bits));
    }
    match attrs.bit_length {
        Some((0, s)) => Err(Error::new(s, "a field must use at least 1 bit")),
        Some((bits, _)) => Ok((Kind::Nested, bits)),
        None => Err(Error::new(
            span,
            "non-primitive fields need `bit_length` or `byte_length` (equal to the nested type's BIT_SIZE)",
        )),
    }
}

/// How logical bits are stored in a byte buffer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Phys {
    pub aligned: bool,
    pub back: bool,
    pub reverse: bool,
    pub bit_size: usize,
    pub byte_size: usize,
}

impl Phys {
    /// The canonical layout used to embed a type inside another: logical bit `i` is bit
    /// `7 - i % 8` of byte `i / 8`.
    pub fn logical(bit_size: usize) -> Phys {
        Phys { aligned: false, back: false, reverse: false, bit_size, byte_size: bit_size.div_ceil(8) }
    }

    /// Location of bit `j` of the leaf occupying logical bits `start..start + len`.
    pub fn locate(&self, start: usize, len: usize, j: usize) -> (usize, u32) {
        // `back` mirrors the field inside the `bit_size` logical bits, so padding stays
        // where it would be without `back` (matches bondrewd 0.3).
        let start = if self.back { self.bit_size - start - len } else { start };
        let (byte, bit) = if self.aligned {
            let a = self.byte_size * 8 - self.bit_size + start + j;
            (a / 8, (a % 8) as u32)
        } else {
            let f = start + j;
            (f / 8, 7 - (f % 8) as u32)
        };
        if self.reverse { (self.byte_size - 1 - byte, bit) } else { (byte, bit) }
    }
}

/// How logical bits of a leaf map onto its value bytes.
#[derive(Clone, Copy)]
pub enum ValueMap {
    /// Little endian integer bytes; value bits `0..len` are significant.
    Int { endian: Endian, len: usize },
    /// Canonical logical bytes of a nested type.
    Logical,
}

impl ValueMap {
    fn locate(self, j: usize) -> (usize, u32) {
        match self {
            ValueMap::Logical => (j / 8, 7 - (j % 8) as u32),
            ValueMap::Int { endian, len } => {
                let v = match endian {
                    Endian::Big => len - 1 - j,
                    Endian::Aligned => j,
                    Endian::Little => {
                        let (k, r) = (j / 8, j % 8);
                        if k < len / 8 { 8 * k + 7 - r } else { 8 * k + (len % 8) - 1 - r }
                    }
                };
                (v / 8, (v % 8) as u32)
            }
        }
    }
}

/// One byte-to-byte transfer: `value[vb] bit (p + shift)` <-> `buf[pb] bit p` for each
/// bit `p` set in `mask`.
pub struct Chunk {
    pub pb: usize,
    pub vb: usize,
    pub mask: u8,
    pub shift: i32,
}

pub fn chunks(phys: &Phys, start: usize, len: usize, vmap: ValueMap) -> Vec<Chunk> {
    let mut map: BTreeMap<(usize, usize, i32), u8> = BTreeMap::new();
    for j in 0..len {
        let (pb, pbit) = phys.locate(start, len, j);
        let (vb, vbit) = vmap.locate(j);
        *map.entry((pb, vb, vbit as i32 - pbit as i32)).or_default() |= 1 << pbit;
    }
    map.into_iter().map(|((pb, vb, shift), mask)| Chunk { pb, vb, mask, shift }).collect()
}

/// Smallest amount of buffer bytes needed to touch every bit of `start..start + len`.
pub fn bytes_needed(phys: &Phys, start: usize, len: usize) -> usize {
    (0..len).map(|j| phys.locate(start, len, j).0 + 1).max().unwrap_or(0)
}

/// Assigns start positions in declaration order and checks for overlaps.
///
/// `first` is where the first field starts (after an enum's id bits) and `id_bits` is
/// the reserved range at the front that only `capture_id` fields may use.
pub fn place(fields: &mut [Field], first: usize, id_bits: usize, explicit: &[Option<(usize, usize, Span)>]) -> Result<usize> {
    let mut cursor = first;
    for (f, bits_attr) in fields.iter_mut().zip(explicit) {
        if f.capture_id {
            f.start = 0;
            if f.bits < id_bits {
                return Err(Error::new(f.span, format!("capture_id field needs at least {id_bits} bits")));
            }
            f.bits = id_bits;
            continue;
        }
        if let Some((a, b, s)) = *bits_attr {
            if b - a != f.bits {
                return Err(Error::new(
                    s,
                    format!("range is {} bits but the field uses {} bits", b - a, f.bits),
                ));
            }
            f.start = a;
        } else {
            f.start = cursor;
        }
        cursor = cursor.max(f.end());
    }
    for (i, a) in fields.iter().enumerate() {
        if a.redundant || a.capture_id {
            continue;
        }
        if a.start < id_bits {
            return Err(Error::new(a.span, "field overlaps the variant id bits"));
        }
        for b in &fields[i + 1..] {
            if b.redundant || b.capture_id {
                continue;
            }
            if a.start < b.end() && b.start < a.end() {
                return Err(Error::new(
                    b.span,
                    format!("field `{}` overlaps field `{}`; mark it `redundant` if intended", b.name, a.name),
                ));
            }
        }
    }
    Ok(fields.iter().map(Field::end).max().unwrap_or(first).max(first))
}
