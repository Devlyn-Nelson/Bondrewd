//! Turns leaves into read expressions and write statements.

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use syn::Type;

use crate::attrs::Endian;
use crate::layout::{chunks, Chunk, Kind, Phys, Prim, ValueMap};

pub struct Ctx<'a> {
    pub krate: &'a TokenStream,
    pub phys: Phys,
}

fn idx(n: usize) -> Literal {
    Literal::usize_unsuffixed(n)
}

/// Masks are emitted as binary so dumped code shows which bits are touched.
fn u8_lit(n: u8) -> Literal {
    let s = format!("{n:08b}");
    format!("0b{}_{}u8", &s[..4], &s[4..]).parse().expect("valid literal")
}

fn read_chunks(chunks: &[Chunk], buf: &TokenStream) -> TokenStream {
    chunks
        .iter()
        .map(|c| {
            let (pb, vb) = (idx(c.pb), idx(c.vb));
            let src = if c.mask == 0xFF {
                quote!(#buf[#pb])
            } else {
                let m = u8_lit(c.mask);
                quote!((#buf[#pb] & #m))
            };
            let val = match c.shift {
                0 => src,
                s if s > 0 => {
                    let s = idx(s as usize);
                    quote!((#src << #s))
                }
                s => {
                    let s = idx((-s) as usize);
                    quote!((#src >> #s))
                }
            };
            quote!(__v[#vb] |= #val;)
        })
        .collect()
}

fn write_chunks(chunks: &[Chunk], buf: &TokenStream) -> TokenStream {
    chunks
        .iter()
        .map(|c| {
            let (pb, vb) = (idx(c.pb), idx(c.vb));
            if c.mask == 0xFF && c.shift == 0 {
                return quote!(#buf[#pb] = __v[#vb];);
            }
            let val = match c.shift {
                0 => quote!(__v[#vb]),
                s if s > 0 => {
                    let s = idx(s as usize);
                    quote!((__v[#vb] >> #s))
                }
                s => {
                    let s = idx((-s) as usize);
                    quote!((__v[#vb] << #s))
                }
            };
            let m = u8_lit(c.mask);
            let keep = u8_lit(!c.mask);
            quote!(#buf[#pb] = (#buf[#pb] & #keep) | (#val & #m);)
        })
        .collect()
}

fn int_ident(p: Prim) -> proc_macro2::Ident {
    match p {
        Prim::Signed(n) => format_ident!("i{}", n * 8),
        other => format_ident!("u{}", other.repr_bytes() * 8),
    }
}

fn nested_trait(ctx: &Ctx, bits: usize) -> TokenStream {
    let krate = ctx.krate;
    let size = idx(bits.div_ceil(8));
    quote!(#krate::Bitfields<#size>)
}

/// Expression reading the value of a field occupying `start..start + bits`.
pub fn read_expr(ctx: &Ctx, kind: &Kind, ty: &Type, endian: Endian, start: usize, bits: usize, buf: &TokenStream) -> TokenStream {
    match kind {
        Kind::Prim(p) => {
            let n = idx(p.repr_bytes());
            let ops = read_chunks(&chunks(&ctx.phys, start, bits, ValueMap::Int { endian, len: bits }), buf);
            let uint = format_ident!("u{}", p.repr_bytes() * 8);
            let convert = match p {
                Prim::Unsigned(_) => quote!(#uint::from_le_bytes(__v)),
                Prim::Signed(w) => {
                    let int = int_ident(*p);
                    let pad = w * 8 - bits;
                    if pad == 0 {
                        quote!(#int::from_le_bytes(__v))
                    } else {
                        let pad = idx(pad);
                        quote!((#int::from_le_bytes(__v) << #pad) >> #pad)
                    }
                }
                Prim::Bool => quote!(__v[0] != 0),
                Prim::F32 => quote!(f32::from_bits(u32::from_le_bytes(__v))),
                Prim::F64 => quote!(f64::from_bits(u64::from_le_bytes(__v))),
                Prim::Char => {
                    quote!(::core::char::from_u32(u32::from_le_bytes(__v)).unwrap_or(::core::char::REPLACEMENT_CHARACTER))
                }
            };
            quote!({ let mut __v = [0u8; #n]; #ops #convert })
        }
        Kind::Nested => {
            let n = idx(bits.div_ceil(8));
            let ops = read_chunks(&chunks(&ctx.phys, start, bits, ValueMap::Logical), buf);
            let tr = nested_trait(ctx, bits);
            quote!({ let mut __v = [0u8; #n]; #ops <#ty as #tr>::__from_logical(__v) })
        }
        Kind::Element { len, elem_ty, elem, elem_bits } => {
            let items = (0..*len).map(|k| read_expr(ctx, elem, elem_ty, endian, start + k * elem_bits, *elem_bits, buf));
            quote!([#(#items),*])
        }
        Kind::Block { len, elem } => {
            let items = block_elements(*len, *elem, start, bits).into_iter().map(|e| match e {
                Some((s, w)) => read_expr(ctx, &Kind::Prim(*elem), ty, endian, s, w, buf),
                None => quote!(0),
            });
            quote!([#(#items),*])
        }
    }
}

/// Start and width of every element of a block array, `None` when it is truncated away.
///
/// Elements are packed in array order, each in the field's endianness. When the block is
/// shorter than the whole array, the most significant bits of element 0 (then element 1,
/// ...) are removed. For `be` this equals treating the array as one big number with
/// element 0 most significant, and for `le` it matches bondrewd 0.3.
fn block_elements(len: usize, elem: Prim, start: usize, bits: usize) -> Vec<Option<(usize, usize)>> {
    let full = elem.repr_bytes() * 8;
    let mut remaining = bits;
    let mut widths: Vec<usize> = (0..len)
        .rev()
        .map(|_| {
            let w = remaining.min(full);
            remaining -= w;
            w
        })
        .collect();
    widths.reverse();
    let mut at = start;
    widths
        .into_iter()
        .map(|w| {
            let e = (w > 0).then_some((at, w));
            at += w;
            e
        })
        .collect()
}

/// Statements writing `val` (an owned expression) into a field occupying `start..start + bits`.
#[allow(clippy::too_many_arguments)]
pub fn write_stmts(
    ctx: &Ctx,
    kind: &Kind,
    ty: &Type,
    endian: Endian,
    start: usize,
    bits: usize,
    buf: &TokenStream,
    val: &TokenStream,
    depth: usize,
) -> TokenStream {
    match kind {
        Kind::Prim(p) => {
            let n = idx(p.repr_bytes());
            let ops = write_chunks(&chunks(&ctx.phys, start, bits, ValueMap::Int { endian, len: bits }), buf);
            let bytes = match p {
                Prim::Unsigned(_) | Prim::Signed(_) => quote!((#val).to_le_bytes()),
                Prim::Bool => quote!([(#val) as u8]),
                Prim::F32 | Prim::F64 => quote!((#val).to_bits().to_le_bytes()),
                Prim::Char => quote!(((#val) as u32).to_le_bytes()),
            };
            quote!({ let __v: [u8; #n] = #bytes; #ops })
        }
        Kind::Nested => {
            let n = idx(bits.div_ceil(8));
            let ops = write_chunks(&chunks(&ctx.phys, start, bits, ValueMap::Logical), buf);
            let tr = nested_trait(ctx, bits);
            quote!({ let __v: [u8; #n] = <#ty as #tr>::__into_logical(#val); #ops })
        }
        Kind::Element { len, elem_ty, elem, elem_bits } => {
            let names: Vec<_> = (0..*len).map(|k| format_ident!("__e{}_{}", depth, k)).collect();
            let writes = names.iter().enumerate().map(|(k, name)| {
                write_stmts(ctx, elem, elem_ty, endian, start + k * elem_bits, *elem_bits, buf, &quote!(#name), depth + 1)
            });
            quote!({ let [#(#names),*] = #val; #(#writes)* })
        }
        Kind::Block { len, elem } => {
            let writes = block_elements(*len, *elem, start, bits).into_iter().enumerate().filter_map(|(k, e)| {
                let (s, w) = e?;
                let k = idx(k);
                Some(write_stmts(ctx, &Kind::Prim(*elem), ty, endian, s, w, buf, &quote!(__a[#k]), depth + 1))
            });
            quote!({ let __a = #val; #(#writes)* })
        }
    }
}

/// Compile time checks that nested types have the bit size the field claims.
pub fn nested_asserts(ctx: &Ctx, kind: &Kind, ty: &Type, bits: usize) -> TokenStream {
    match kind {
        Kind::Nested => {
            let tr = nested_trait(ctx, bits);
            let b = idx(bits);
            let msg = format!(
                "bit length given for nested type `{}` does not match its BIT_SIZE",
                quote!(#ty).to_string().replace(' ', "")
            );
            quote!(const _: () = ::core::assert!(<#ty as #tr>::BIT_SIZE == #b, #msg);)
        }
        Kind::Element { elem_ty, elem, elem_bits, .. } => nested_asserts(ctx, elem, elem_ty, *elem_bits),
        _ => TokenStream::new(),
    }
}
