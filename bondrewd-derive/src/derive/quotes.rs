use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

/// This contains incomplete function generation. this should only be used by `StructInfo` or `EnumInfo` internally.
pub struct FieldQuotes {
    pub read_fns: GeneratedFunctions,
    pub write_fns: GeneratedFunctions,
    /// A list of field names to be used in initializing a new struct from bytes.
    pub field_list: TokenStream,
    pub slice_info: Option<CheckSliceNames>,
}
impl FieldQuotes {
    pub fn finish(self) -> GeneratedFunctions {
        let mut read = self.read_fns;
        read.merge(&self.write_fns);
        read
    }
}

pub struct FieldQuotesNew {
    /// A list of field names to be used in initializing a new struct from bytes.
    pub field_list: TokenStream,
    pub slice_info: Option<CheckSliceNames>,
}

pub struct CheckSliceNames {
    /// describes the check slice function name
    pub func: Ident,
    /// describes the check mut slice function name
    pub mut_func: Ident,
    /// describes the check slice struct name
    pub structure: Ident,
    /// describes the check mut slice struct name
    pub mut_structure: Ident,
}

#[derive(Clone, Default)]
pub struct GeneratedFunctions {
    /// Functions that belong in `Bitfields` impl for object.
    pub bitfield_trait: TokenStream,
    /// Functions that belong in impl for object.
    pub non_trait: TokenStream,
    pub dyn_fns: Option<GeneratedDynFunctions>,
}

#[derive(Clone, Default)]
pub struct GeneratedDynFunctions {
    /// Functions that belong in impl for generated checked slice object.
    pub checked_struct: TokenStream,
    /// Functions that belong in `BitfieldsDyn` impl for object.
    pub bitfield_dyn_trait: TokenStream,
}

impl From<GeneratedFunctions> for TokenStream {
    fn from(val: GeneratedFunctions) -> Self {
        let trait_fns = val.bitfield_trait;
        let impl_fns = val.non_trait;

        if let Some(dyn_fns) = val.dyn_fns {
            let unchecked = dyn_fns.checked_struct;
            let dyn_trait_fns = dyn_fns.bitfield_dyn_trait;
            quote! {
                #trait_fns
                #impl_fns
                #unchecked
                #dyn_trait_fns
            }
        } else {
            quote! {
                #trait_fns
                #impl_fns
            }
        }
    }
}

impl GeneratedFunctions {
    pub fn new(dyn_fns: bool) -> Self {
        let out = Self::default();
        if dyn_fns {
            out.with_dyn_fns()
        } else {
            out
        }
    }
    pub fn merge(&mut self, other: &Self) {
        let bitfield_trait_impl_fns = &self.bitfield_trait;
        let other_bitfield_trait_impl_fns = &other.bitfield_trait;
        self.bitfield_trait = quote! {
            #bitfield_trait_impl_fns
            #other_bitfield_trait_impl_fns
        };
        let impl_fns = &self.non_trait;
        let other_impl_fns = &other.non_trait;
        self.non_trait = quote! {
            #impl_fns
            #other_impl_fns
        };
        // dyn
        if let (Some(dyn_fns), Some(other_dyn_fns)) = (&mut self.dyn_fns, &other.dyn_fns) {
            let checked_struct_impl_fns = &dyn_fns.checked_struct;
            let other_checked_struct_impl_fns = &other_dyn_fns.checked_struct;
            dyn_fns.checked_struct = quote! {
                #checked_struct_impl_fns
                #other_checked_struct_impl_fns
            };
            let bitfield_dyn_trait_impl_fns = &dyn_fns.bitfield_dyn_trait;
            let other_bitfield_dyn_trait_impl_fns = &other_dyn_fns.bitfield_dyn_trait;
            dyn_fns.bitfield_dyn_trait = quote! {
                #bitfield_dyn_trait_impl_fns
                #other_bitfield_dyn_trait_impl_fns
            };
        }
    }
    pub fn append_bitfield_trait_impl_fns(&mut self, quote: &TokenStream) {
        let old = &self.bitfield_trait;
        self.bitfield_trait = quote! {
            #old
            #quote
        };
    }
    pub fn append_impl_fns(&mut self, quote: &TokenStream) {
        let old = &self.non_trait;
        self.non_trait = quote! {
            #old
            #quote
        };
    }
    pub fn append_checked_struct_impl_fns(&mut self, quote: &TokenStream) {
        if let Some(dyn_fns) = &mut self.dyn_fns {
            let old = &dyn_fns.checked_struct;
            dyn_fns.checked_struct = quote! {
                #old
                #quote
            };
        }
    }
    pub fn append_bitfield_dyn_trait_impl_fns(&mut self, quote: &TokenStream) {
        if let Some(dyn_fns) = &mut self.dyn_fns {
            let old = &dyn_fns.bitfield_dyn_trait;
            dyn_fns.bitfield_dyn_trait = quote! {
                #old
                #quote
            };
        }
    }
    pub fn with_dyn_fns(mut self) -> Self {
        if self.dyn_fns.is_none() {
            self.dyn_fns = Some(GeneratedDynFunctions::default());
        }
        self
    }
}
pub struct CheckedSliceGenQuotes {
    pub fn_gen: TokenStream,
    pub trait_type: TokenStream,
    pub fn_name: Ident,
}

pub struct CheckedSliceGen {
    pub read: CheckedSliceGenQuotes,
    pub write: CheckedSliceGenQuotes,
}
impl CheckedSliceGen {
    pub fn new(
        name: &Ident,
        // total_bytes
        check_size: usize,
        enum_name: Option<&Ident>,
    ) -> Self {
        let read = get_check_slice_fn(name, check_size, enum_name);
        let write = get_check_mut_slice_fn(name, check_size, enum_name);
        Self { read, write }
    }
}

/// returns (fn, `fn_name`).
///
/// `name` is the name of the structure or variant
/// `check_size` is the total byte size of the struct or variant
/// `enum_name` if we are generating code for a variant (not a structure) then a
///     Some value containing the prefixed name shall be provided.
///     ex. enum and variant -> `Test::One` = "`test_one`" <- prefixed name
fn get_check_mut_slice_fn(
    name: &Ident,
    // total_bytes
    check_size: usize,
    enum_name: Option<&Ident>,
) -> CheckedSliceGenQuotes {
    let (checked_ident_mut, fn_name) = if let Some(ename) = enum_name {
        (
            format_ident!("{ename}CheckedMut"),
            format_ident!("check_slice_mut_{}", name.to_string().to_case(Case::Snake)),
        )
    } else {
        (
            format_ident!("{name}CheckedMut"),
            format_ident!("check_slice_mut"),
        )
    };
    let comment_mut = format!(
        "Returns a [{checked_ident_mut}] which allows you to read/write any field for a `{}` from/to provided mutable slice.",
        if let Some(ename) = enum_name {
            format!("{ename}::{name}")
        }else{
            name.to_string()
        }
    );
    CheckedSliceGenQuotes {
        fn_gen: quote! {
            #[doc = #comment_mut]
            fn #fn_name<'a>(buffer: &'a mut [u8]) -> Result<#checked_ident_mut<'a>, bondrewd::BitfieldLengthError> {
                let buf_len = buffer.len();
                if buf_len >= #check_size {
                    Ok(#checked_ident_mut {
                        buffer
                    })
                }else{
                    Err(bondrewd::BitfieldLengthError(buf_len, #check_size))
                }
            }
        },
        trait_type: quote! {type CheckedMut<'a> = #checked_ident_mut<'a>;},
        fn_name,
    }
}

/// generates the `check_slice` fn. please do not use, use `CheckedSliceGen`.
/// returns (fn, `fn_name`).
///
/// `name` is the name of the structure or variant
/// `check_size` is the total byte size of the struct or variant
/// `enum_name` if we are generating code for a variant (not a structure) then a
///     Some value containing the prefixed name shall be provided.
///     ex. enum and variant -> `Test::One` = "`test_one`" <- prefixed name
fn get_check_slice_fn(
    name: &Ident,
    // total_bytes
    check_size: usize,
    enum_name: Option<&Ident>,
) -> CheckedSliceGenQuotes {
    let (checked_ident, fn_name) = if let Some(ename) = enum_name {
        (
            format_ident!("{ename}Checked"),
            format_ident!("check_slice_{}", name.to_string().to_case(Case::Snake)),
        )
    } else {
        (format_ident!("{name}Checked"), format_ident!("check_slice"))
    };
    let comment = format!(
        "Returns a [{checked_ident}] which allows you to read any field for a `{}` from provided slice.",
        if let Some(ename) = enum_name {
            format!("{ename}::{name}")
        }else{
            name.to_string()
        }
    );
    CheckedSliceGenQuotes {
        fn_gen: quote! {
            #[doc = #comment]
            fn #fn_name<'a>(buffer: &'a [u8]) -> Result<#checked_ident<'a>, bondrewd::BitfieldLengthError> {
                let buf_len = buffer.len();
                if buf_len >= #check_size {
                    Ok(#checked_ident {
                        buffer
                    })
                }else{
                    Err(bondrewd::BitfieldLengthError(buf_len, #check_size))
                }
            }
        },
        trait_type: quote! {type Checked<'a> = #checked_ident<'a>;},
        fn_name,
    }
}
