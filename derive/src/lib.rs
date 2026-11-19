//! Derive macros for `bondrewd`. See the `bondrewd` crate for documentation.

mod attrs;
mod codegen;
mod dump;
mod layout;

use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Ident, Result};

use attrs::{Endian, Fill, TypeAttrs};
use codegen::{nested_asserts, read_expr, write_stmts, Ctx};
use layout::{classify, place, Field, Kind, Phys, Prim};

/// Derives `Bitfields`: `into_bytes`, `from_bytes`, `BIT_SIZE`, `BYTE_SIZE` and
/// `read_<field>` / `write_<field>` associated functions.
#[proc_macro_derive(Bitfields, attributes(bondrewd))]
pub fn derive_bitfields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    Model::build(&input)
        .map_or_else(|e| e.to_compile_error(), |m| m.finish(m.gen_bitfields(), "bitfields"))
        .into()
}

/// Derives `BitfieldsSlice`: `read_slice_<field>` / `write_slice_<field>` associated
/// functions plus `<Name>Checked` / `<Name>CheckedMut` slice views.
#[proc_macro_derive(BitfieldsSlice, attributes(bondrewd))]
pub fn derive_bitfields_slice(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    Model::build(&input)
        .map_or_else(|e| e.to_compile_error(), |m| m.finish(m.gen_slice(), "bitfields_slice"))
        .into()
}

/// An explicit `bits = "a..b"` placement for a field.
type ExplicitBits = Option<(usize, usize, Span)>;

struct Variant {
    ident: Ident,
    fields: Vec<Field>,
    id: u128,
}

enum Body {
    Struct(Vec<Field>),
    Enum { id: Box<Field>, variants: Vec<Variant>, fallback: usize },
}

struct Model {
    ident: Ident,
    vis: syn::Visibility,
    krate: TokenStream,
    bit_size: usize,
    byte_size: usize,
    phys: Phys,
    body: Body,
    /// Span of a `dump` attribute, if the generated code should be written to disk.
    dump: Option<Span>,
}

fn snake(name: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev_lower = i > 0 && (chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit());
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            if i > 0 && (prev_lower || (next_lower && chars[i - 1].is_uppercase())) {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(*c);
        }
    }
    out
}

fn idx(n: usize) -> Literal {
    Literal::usize_unsuffixed(n)
}

/// Builds the fields of a struct or variant (positions are assigned by `place`).
fn build_fields(
    fields: &syn::Fields,
    type_endian: Endian,
    in_enum: bool,
) -> Result<(Vec<Field>, Vec<ExplicitBits>)> {
    let mut out = Vec::new();
    let mut explicit = Vec::new();
    for (i, f) in fields.iter().enumerate() {
        let attrs = attrs::field_attrs(&f.attrs)?;
        let span = f.ident.as_ref().map_or_else(|| f.ty.span(), Ident::span);
        let (kind, bits) = classify(&f.ty, &attrs, span)?;
        let endian = match attrs.endianness {
            None => type_endian,
            Some((e, s)) => {
                if (e == Endian::Aligned) != (type_endian == Endian::Aligned) {
                    return Err(Error::new(
                        s,
                        "`ale` fields can only be used in `ale` types, and `ale` types only allow `ale` fields",
                    ));
                }
                e
            }
        };
        if let Some(s) = attrs.capture_id {
            if !in_enum {
                return Err(Error::new(s, "`capture_id` is only valid on enum variant fields"));
            }
            if !matches!(kind, Kind::Prim(p) if p.is_int()) {
                return Err(Error::new(s, "`capture_id` fields must be integer primitives"));
            }
        }
        let (name, member) = match &f.ident {
            Some(id) => (id.unraw().to_string(), syn::Member::Named(id.clone())),
            None => (format!("field_{i}"), syn::Member::Unnamed(syn::Index::from(i))),
        };
        explicit.push(attrs.bits);
        out.push(Field {
            name,
            member,
            ty: f.ty.clone(),
            kind,
            endian,
            start: 0,
            bits,
            reserve: attrs.reserve,
            redundant: attrs.redundant,
            capture_id: attrs.capture_id.is_some(),
            span,
        });
    }
    Ok((out, explicit))
}

fn finish_size(used: usize, attrs: &TypeAttrs, span: Span) -> Result<usize> {
    let bits = match attrs.fill {
        None => used,
        Some(Fill::ToByte) => used.div_ceil(8) * 8,
        Some(Fill::Bits(n)) => {
            if n < used {
                return Err(Error::new(span, format!("fill_bits = {n} is smaller than the {used} bits in use")));
            }
            n
        }
    };
    if let Some((n, s)) = attrs.enforce_bits
        && n != bits {
            return Err(Error::new(s, format!("expected {n} bits, but {bits} bits are used")));
        }
    if let Some((n, s)) = attrs.enforce_bytes
        && n != bits.div_ceil(8) {
            return Err(Error::new(s, format!("expected {n} bytes, but {} bytes are used", bits.div_ceil(8))));
        }
    if let Some(s) = attrs.enforce_full_bytes
        && bits % 8 != 0 {
            return Err(Error::new(s, format!("{bits} bits is not a whole number of bytes")));
        }
    if bits == 0 {
        return Err(Error::new(span, "bitfields types need at least 1 bit"));
    }
    Ok(bits)
}

impl Model {
    fn build(input: &DeriveInput) -> Result<Model> {
        if !input.generics.params.is_empty() {
            return Err(Error::new_spanned(&input.generics, "generic bitfields types are not supported yet"));
        }
        let is_enum = matches!(input.data, Data::Enum(_));
        let attrs = attrs::type_attrs(&input.attrs, is_enum)?;
        let endian = attrs.endianness.unwrap_or(Endian::Big);
        let span = input.ident.span();
        let (used, body) = match &input.data {
            Data::Struct(s) => {
                let (mut fields, explicit) = build_fields(&s.fields, endian, false)?;
                let used = place(&mut fields, 0, 0, &explicit)?;
                (used, Body::Struct(fields))
            }
            Data::Enum(e) => {
                if e.variants.is_empty() {
                    return Err(Error::new(span, "enums need at least one variant"));
                }
                let mut ids = Vec::new();
                let mut next = 0u128;
                let mut fallback = None;
                for v in &e.variants {
                    let va = attrs::variant_attrs(&v.attrs)?;
                    let id = match (va.id, &v.discriminant) {
                        (Some((id, _)), _) => id,
                        (None, Some((_, syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(i), .. })))) => {
                            i.base10_parse()?
                        }
                        (None, Some((_, expr))) => {
                            return Err(Error::new_spanned(expr, "discriminants must be integer literals"));
                        }
                        (None, None) => next,
                    };
                    if ids.contains(&id) {
                        return Err(Error::new(v.ident.span(), format!("variant id {id} is used twice")));
                    }
                    if va.invalid {
                        if fallback.is_some() {
                            return Err(Error::new(v.ident.span(), "only one variant can be `invalid`"));
                        }
                        fallback = Some(ids.len());
                    }
                    ids.push(id);
                    next = id + 1;
                }
                let max_id = ids.iter().copied().max().unwrap_or(0);
                let needed = (128 - max_id.leading_zeros() as usize).max(1);
                let id_bits = match attrs.id_bits {
                    Some((b, s)) if b < needed || b > 128 => {
                        return Err(Error::new(s, format!("id bit length must be between {needed} and 128")));
                    }
                    Some((b, _)) => b,
                    None => needed,
                };
                let id_prim = Prim::Unsigned(id_bits.div_ceil(8).next_power_of_two());
                let id_ty = format_ident!("u{}", id_prim.repr_bytes() * 8);
                let id = Field {
                    name: "variant_id".into(),
                    member: syn::Member::Named(format_ident!("variant_id")),
                    ty: syn::parse_quote!(#id_ty),
                    kind: Kind::Prim(id_prim),
                    endian,
                    start: 0,
                    bits: id_bits,
                    reserve: false,
                    redundant: false,
                    capture_id: false,
                    span,
                };
                let mut used = id_bits;
                let mut variants = Vec::new();
                for (v, id) in e.variants.iter().zip(ids) {
                    let (mut fields, explicit) = build_fields(&v.fields, endian, true)?;
                    if fields.iter().filter(|f| f.capture_id).count() > 1 {
                        return Err(Error::new(v.ident.span(), "only one field can `capture_id`"));
                    }
                    used = used.max(place(&mut fields, id_bits, id_bits, &explicit)?);
                    variants.push(Variant { ident: v.ident.clone(), fields, id });
                }
                let fallback = fallback.unwrap_or(variants.len() - 1);
                (used, Body::Enum { id: Box::new(id), variants, fallback })
            }
            Data::Union(_) => return Err(Error::new(span, "unions are not supported")),
        };
        let bit_size = finish_size(used, &attrs, span)?;
        let byte_size = bit_size.div_ceil(8);
        let krate = match &attrs.crate_path {
            Some(p) => quote!(#p),
            None => quote!(::bondrewd),
        };
        Ok(Model {
            ident: input.ident.clone(),
            vis: input.vis.clone(),
            krate,
            bit_size,
            byte_size,
            phys: Phys {
                aligned: endian == Endian::Aligned,
                back: attrs.back,
                reverse: attrs.reverse,
                bit_size,
                byte_size,
            },
            body,
            dump: attrs.dump,
        })
    }

    /// Writes the generated code to the target directory when `dump` was requested.
    fn finish(&self, generated: TokenStream, derive: &str) -> TokenStream {
        let Some(span) = self.dump else { return generated };
        match dump::write(&self.ident.to_string(), derive, &generated) {
            Ok(_) => generated,
            Err(msg) => {
                let err = Error::new(span, msg).to_compile_error();
                quote!(#generated #err)
            }
        }
    }

    fn ctx(&self, phys: Phys) -> Ctx<'_> {
        Ctx { krate: &self.krate, phys }
    }

    fn read(&self, ctx: &Ctx, f: &Field, buf: &TokenStream) -> TokenStream {
        read_expr(ctx, &f.kind, &f.ty, f.endian, f.start, f.bits, buf)
    }

    fn write(&self, ctx: &Ctx, f: &Field, buf: &TokenStream, val: &TokenStream) -> TokenStream {
        write_stmts(ctx, &f.kind, &f.ty, f.endian, f.start, f.bits, buf, val, 0)
    }

    /// Expression building `Self` from the bytes in `buf`.
    fn gen_from(&self, phys: Phys, buf: &TokenStream) -> TokenStream {
        let ctx = self.ctx(phys);
        let construct = |path: TokenStream, fields: &[Field]| {
            let inits = fields.iter().map(|f| {
                let m = &f.member;
                let value = if f.reserve {
                    quote!(::core::default::Default::default())
                } else {
                    self.read(&ctx, f, buf)
                };
                quote!(#m: #value)
            });
            quote!(#path { #(#inits),* })
        };
        match &self.body {
            Body::Struct(fields) => construct(quote!(Self), fields),
            Body::Enum { id, variants, fallback } => {
                let id_read = self.read(&ctx, id, buf);
                let id_ty = &id.ty;
                let arms = variants.iter().enumerate().filter(|(i, _)| i != fallback).map(|(_, v)| {
                    let lit = Literal::u128_unsuffixed(v.id);
                    let ident = &v.ident;
                    let c = construct(quote!(Self::#ident), &v.fields);
                    quote!(#lit => #c,)
                });
                let fb = &variants[*fallback];
                let fb_ident = &fb.ident;
                let fb_c = construct(quote!(Self::#fb_ident), &fb.fields);
                quote!({
                    let __id: #id_ty = #id_read;
                    #[allow(unreachable_patterns)]
                    match __id { #(#arms)* _ => #fb_c, }
                })
            }
        }
    }

    /// Statements writing every field of `self` into `buf`.
    fn gen_into(&self, phys: Phys, buf: &TokenStream) -> TokenStream {
        let ctx = self.ctx(phys);
        let destructure = |path: TokenStream, fields: &[Field]| {
            let binds = fields.iter().map(|f| {
                let m = &f.member;
                if f.reserve || f.redundant {
                    quote!(#m: _)
                } else {
                    let b = format_ident!("__f_{}", f.name);
                    quote!(#m: #b)
                }
            });
            let writes: Vec<_> = fields
                .iter()
                .filter(|f| !f.reserve && !f.redundant)
                .map(|f| {
                    let b = format_ident!("__f_{}", f.name);
                    self.write(&ctx, f, buf, &quote!(#b))
                })
                .collect();
            (quote!(#path { #(#binds),* }), writes)
        };
        match &self.body {
            Body::Struct(fields) => {
                let (pat, writes) = destructure(quote!(Self), fields);
                quote!(let #pat = self; #(#writes)*)
            }
            Body::Enum { id, variants, .. } => {
                let id_ty = &id.ty;
                let arms = variants.iter().map(|v| {
                    let ident = &v.ident;
                    let (pat, writes) = destructure(quote!(Self::#ident), &v.fields);
                    let id_write = if v.fields.iter().any(|f| f.capture_id) {
                        TokenStream::new()
                    } else {
                        let lit = Literal::u128_unsuffixed(v.id);
                        let w = self.write(&ctx, id, buf, &quote!(__id));
                        quote!(let __id: #id_ty = #lit; #w)
                    };
                    quote!(#pat => { #id_write #(#writes)* })
                });
                quote!(match self { #(#arms)* })
            }
        }
    }

    /// Every accessible field with the suffix used in `read_<suffix>` names.
    fn accessors(&self) -> Vec<(String, &Field)> {
        match &self.body {
            Body::Struct(fields) => fields.iter().map(|f| (f.name.clone(), f)).collect(),
            Body::Enum { id, variants, .. } => std::iter::once((id.name.clone(), &**id))
                .chain(variants.iter().flat_map(|v| {
                    let prefix = snake(&v.ident.unraw().to_string());
                    v.fields.iter().map(move |f| (format!("{prefix}_{}", f.name), f))
                }))
                .collect(),
        }
    }

    fn asserts(&self) -> TokenStream {
        let ctx = self.ctx(self.phys);
        let fields: Vec<&Field> = match &self.body {
            Body::Struct(f) => f.iter().collect(),
            Body::Enum { variants, .. } => variants.iter().flat_map(|v| &v.fields).collect(),
        };
        fields.iter().map(|f| nested_asserts(&ctx, &f.kind, &f.ty, f.bits)).collect()
    }

    fn gen_bitfields(&self) -> TokenStream {
        let Model { ident, krate, .. } = self;
        let (n, b) = (idx(self.byte_size), idx(self.bit_size));
        let buf = quote!(__buf);
        let into = self.gen_into(self.phys, &buf);
        let from = self.gen_from(self.phys, &buf);
        let logical = Phys::logical(self.bit_size);
        let logical_fns = (self.phys != logical).then(|| {
            let into = self.gen_into(logical, &buf);
            let from = self.gen_from(logical, &buf);
            quote! {
                #[allow(unused_mut, unused_variables, clippy::all, clippy::pedantic)]
                fn __into_logical(self) -> [u8; #n] { let mut __buf = [0u8; #n]; #into __buf }
                #[allow(unused_variables, clippy::all, clippy::pedantic)]
                fn __from_logical(__buf: [u8; #n]) -> Self { #from }
            }
        });
        let ctx = self.ctx(self.phys);
        let bytes = quote!(bytes);
        let fns = self.accessors().into_iter().map(|(name, f)| {
            let (r, w) = (format_ident!("read_{}", name), format_ident!("write_{}", name));
            let ty = &f.ty;
            let read = self.read(&ctx, f, &bytes);
            let write = self.write(&ctx, f, &bytes, &quote!(value));
            let doc = format!("Bits `{}..{}` of the logical layout.", f.start, f.end());
            quote! {
                #[doc = #doc]
                #[inline]
                #[allow(clippy::all, clippy::pedantic)]
                pub fn #r(bytes: &[u8; #n]) -> #ty { #read }
                #[doc = #doc]
                #[inline]
                #[allow(clippy::all, clippy::pedantic)]
                pub fn #w(bytes: &mut [u8; #n], value: #ty) { #write }
            }
        });
        let asserts = self.asserts();
        quote! {
            #asserts
            impl #krate::Bitfields<#n> for #ident {
                const BIT_SIZE: usize = #b;
                #[allow(unused_mut, unused_variables, clippy::all, clippy::pedantic)]
                fn into_bytes(self) -> [u8; #n] { let mut __buf = [0u8; #n]; #into __buf }
                #[allow(unused_variables, clippy::all, clippy::pedantic)]
                fn from_bytes(__buf: [u8; #n]) -> Self { #from }
                #logical_fns
            }
            impl #ident {
                /// Total amount of bytes the bitfields occupy.
                pub const BYTE_SIZE: usize = #n;
                /// Total amount of bits the bitfields occupy.
                pub const BIT_SIZE: usize = #b;
                #(#fns)*
            }
        }
    }

    fn gen_slice(&self) -> TokenStream {
        let Model { ident, krate, vis, .. } = self;
        let (n, b) = (idx(self.byte_size), idx(self.bit_size));
        let checked = format_ident!("{}Checked", ident);
        let checked_mut = format_ident!("{}CheckedMut", ident);
        let err = quote!(#krate::BitfieldLengthError);
        let ctx = self.ctx(self.phys);
        let accessors = self.accessors();

        let bytes = quote!(bytes);
        let slice_fns = accessors.iter().map(|(name, f)| {
            let (r, w) = (format_ident!("read_slice_{}", name), format_ident!("write_slice_{}", name));
            let ty = &f.ty;
            let need = idx(layout::bytes_needed(&self.phys, f.start, f.bits));
            let read = self.read(&ctx, f, &bytes);
            let write = self.write(&ctx, f, &bytes, &quote!(value));
            quote! {
                /// # Errors
                /// If `bytes` doesn't contain every byte this field touches.
                #[inline]
                #[allow(clippy::all, clippy::pedantic)]
                pub fn #r(bytes: &[u8]) -> ::core::result::Result<#ty, #err> {
                    if bytes.len() < #need { return ::core::result::Result::Err(#krate::BitfieldLengthError(bytes.len(), #need)); }
                    ::core::result::Result::Ok(#read)
                }
                /// # Errors
                /// If `bytes` doesn't contain every byte this field touches.
                #[inline]
                #[allow(clippy::all, clippy::pedantic)]
                pub fn #w(bytes: &mut [u8], value: #ty) -> ::core::result::Result<(), #err> {
                    if bytes.len() < #need { return ::core::result::Result::Err(#krate::BitfieldLengthError(bytes.len(), #need)); }
                    #write
                    ::core::result::Result::Ok(())
                }
            }
        });

        let buffer = quote!(self.buffer);
        let checked_reads: Vec<_> = accessors
            .iter()
            .map(|(name, f)| {
                let r = format_ident!("read_{}", name);
                let ty = &f.ty;
                let read = self.read(&ctx, f, &buffer);
                quote! {
                    #[inline]
                    #[allow(clippy::all, clippy::pedantic)]
                    pub fn #r(&self) -> #ty { #read }
                }
            })
            .collect();
        let checked_writes = accessors.iter().map(|(name, f)| {
            let w = format_ident!("write_{}", name);
            let ty = &f.ty;
            let write = self.write(&ctx, f, &buffer, &quote!(value));
            quote! {
                #[inline]
                #[allow(clippy::all, clippy::pedantic)]
                pub fn #w(&mut self, value: #ty) { #write }
            }
        });
        let from_slice = self.gen_from(self.phys, &bytes);
        let doc = format!("A byte slice checked to contain the {} bytes of [`{ident}`].", self.byte_size);
        let doc_mut = format!("A mutable byte slice checked to contain the {} bytes of [`{ident}`].", self.byte_size);

        quote! {
            impl #ident {
                #(#slice_fns)*
            }

            #[doc = #doc]
            #[derive(Clone, Copy, Debug)]
            #vis struct #checked<'a> { buffer: &'a [u8] }

            impl<'a> #checked<'a> {
                /// The checked bytes (only the first `BYTE_SIZE` are used).
                pub fn bytes(&self) -> &'a [u8] { self.buffer }
                #(#checked_reads)*
            }

            impl ::core::convert::From<#checked<'_>> for #ident {
                #[allow(clippy::all, clippy::pedantic)]
                fn from(checked: #checked<'_>) -> Self { Self::__bondrewd_from_checked(checked.buffer) }
            }

            #[doc = #doc_mut]
            #[derive(Debug)]
            #vis struct #checked_mut<'a> { buffer: &'a mut [u8] }

            impl<'a> #checked_mut<'a> {
                /// The checked bytes (only the first `BYTE_SIZE` are used).
                pub fn bytes(&self) -> &[u8] { self.buffer }
                #(#checked_reads)*
                #(#checked_writes)*
            }

            impl ::core::convert::From<#checked_mut<'_>> for #ident {
                #[allow(clippy::all, clippy::pedantic)]
                fn from(checked: #checked_mut<'_>) -> Self { Self::__bondrewd_from_checked(checked.buffer) }
            }

            impl #ident {
                #[doc(hidden)]
                #[allow(clippy::all, clippy::pedantic)]
                fn __bondrewd_from_checked(bytes: &[u8]) -> Self { #from_slice }
            }

            impl #krate::BitfieldsSlice for #ident {
                const BYTE_SIZE: usize = #n;
                const BIT_SIZE: usize = #b;
                type Checked<'a> = #checked<'a>;
                type CheckedMut<'a> = #checked_mut<'a>;

                #[allow(clippy::all, clippy::pedantic)]
                fn from_slice(bytes: &[u8]) -> ::core::result::Result<Self, #err> {
                    if bytes.len() < #n { return ::core::result::Result::Err(#krate::BitfieldLengthError(bytes.len(), #n)); }
                    ::core::result::Result::Ok(#from_slice)
                }
                fn check_slice(bytes: &[u8]) -> ::core::result::Result<Self::Checked<'_>, #err> {
                    if bytes.len() < #n { return ::core::result::Result::Err(#krate::BitfieldLengthError(bytes.len(), #n)); }
                    ::core::result::Result::Ok(#checked { buffer: bytes })
                }
                fn check_slice_mut(bytes: &mut [u8]) -> ::core::result::Result<Self::CheckedMut<'_>, #err> {
                    if bytes.len() < #n { return ::core::result::Result::Err(#krate::BitfieldLengthError(bytes.len(), #n)); }
                    ::core::result::Result::Ok(#checked_mut { buffer: bytes })
                }
            }
        }
    }
}
