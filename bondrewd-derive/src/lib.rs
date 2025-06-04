#![allow(unreachable_code, dead_code, unused_variables)]

use std::fmt::Display;

use build::field_set::GenericBuilder;
use proc_macro2::TokenStream;
use quote::quote;
use solved::field_set::Solved;
use syn::{parse_macro_input, DeriveInput};

mod build;
mod derive;
mod masked;
mod solved;

#[derive(Clone)]
pub(crate) struct SplitTokenStream {
    read: TokenStream,
    write: TokenStream,
}

impl SplitTokenStream {
    pub(crate) fn merge(self) -> TokenStream {
        let read = self.read;
        let write = self.write;
        quote! {
            #read
            #write
        }
    }
    pub(crate) fn merged(&self) -> TokenStream {
        let read = &self.read;
        let write = &self.write;
        quote! {
            #read
            #write
        }
    }
    pub fn insert(&mut self, other: Self) {
        let my_read = &mut self.read;
        let my_write = &mut self.write;
        let other_read = other.read;
        let other_write = other.write;
        *my_read = quote! {
            #other_read
            #my_read
        };
        *my_write = quote! {
            #other_write
            #my_write
        };
    }
    pub fn clear(&mut self) {
        self.read = TokenStream::default();
        self.write = TokenStream::default();
    }
}

impl Default for SplitTokenStream {
    fn default() -> Self {
        Self {
            read: TokenStream::new(),
            write: TokenStream::new(),
        }
    }
}

#[derive(Clone)]
pub(crate) enum GenerationFlavor {
    Standard {
        /// Functions that belong in `Bitfields` impl for object.
        trait_fns: SplitTokenStream,
        /// Functions that belong in impl for object.
        impl_fns: SplitTokenStream,
    },
    Dynamic {
        /// Functions that belong in `BitfieldsDyn` impl for object.
        trait_fns: SplitTokenStream,
        /// Functions that belong in impl for object.
        impl_fns: SplitTokenStream,
    },
    Slice {
        /// Functions that belong in `BitfieldsSlice` impl for object.
        trait_fns: SplitTokenStream,
        /// Functions that belong in impl for object.
        impl_fns: SplitTokenStream,
        /// Functions that belong in `BitfieldsSlice` impl for object.
        struct_fns: SplitTokenStream,
    },
    Hex {
        /// Functions that belong in `Bitfields` impl for object.
        trait_fns: TokenStream,
    },
    HexDynamic {
        /// Functions that belong in `Bitfields` impl for object.
        trait_fns: TokenStream,
    },
}

impl Display for GenerationFlavor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GenerationFlavor::Standard {
                trait_fns,
                impl_fns,
            } => "standard",
            GenerationFlavor::Dynamic {
                trait_fns,
                impl_fns,
            } => "dynamic",
            GenerationFlavor::Slice {
                trait_fns,
                impl_fns,
                struct_fns,
            } => "slice",
            GenerationFlavor::Hex { trait_fns } => "hex",
            GenerationFlavor::HexDynamic { trait_fns } => "hex_dynamic",
        };
        write!(f, "{}", s)
    }
}

impl GenerationFlavor {
    pub(crate) fn clear(&mut self) {
        match self {
            GenerationFlavor::Standard {
                trait_fns,
                impl_fns,
            }
            | GenerationFlavor::Dynamic {
                trait_fns,
                impl_fns,
            } => {
                *trait_fns = SplitTokenStream::default();
                // *impl_fns = SplitTokenStream::default();
            }
            GenerationFlavor::Slice {
                trait_fns,
                impl_fns,
                struct_fns,
            } => {
                *trait_fns = SplitTokenStream::default();
                // *impl_fns = SplitTokenStream::default();
                // *struct_fns = SplitTokenStream::default();
            }
            GenerationFlavor::Hex { trait_fns } | GenerationFlavor::HexDynamic { trait_fns } => {
                *trait_fns = TokenStream::new();
            }
        }
    }
    pub(crate) fn new_from_type(&self) -> Self {
        match self {
            GenerationFlavor::Standard {
                trait_fns,
                impl_fns,
            } => Self::standard(),
            GenerationFlavor::Dynamic {
                trait_fns,
                impl_fns,
            } => Self::dynamic(),
            GenerationFlavor::Slice {
                trait_fns,
                impl_fns,
                struct_fns,
            } => Self::slice(),
            GenerationFlavor::Hex { trait_fns } => Self::hex(),
            GenerationFlavor::HexDynamic { trait_fns } => Self::hex_dynamic(),
        }
    }
    pub(crate) fn standard() -> Self {
        Self::Standard {
            trait_fns: SplitTokenStream::default(),
            impl_fns: SplitTokenStream::default(),
        }
    }
    pub(crate) fn dynamic() -> Self {
        Self::Dynamic {
            trait_fns: SplitTokenStream::default(),
            impl_fns: SplitTokenStream::default(),
        }
    }
    pub(crate) fn slice() -> Self {
        Self::Slice {
            trait_fns: SplitTokenStream::default(),
            impl_fns: SplitTokenStream::default(),
            struct_fns: SplitTokenStream::default(),
        }
    }
    pub(crate) fn hex() -> Self {
        Self::Hex {
            trait_fns: TokenStream::new(),
        }
    }
    pub(crate) fn hex_dynamic() -> Self {
        Self::HexDynamic {
            trait_fns: TokenStream::new(),
        }
    }
    pub(crate) fn merge(&mut self, other: &Self) {
        match (self, other) {
            (
                Self::Standard {
                    trait_fns,
                    impl_fns,
                },
                Self::Standard {
                    trait_fns: other_trait_fns,
                    impl_fns: other_impl_fns,
                },
            ) => {
                let read_trait_fns = &mut trait_fns.read;
                let other_read_trait_fns = &other_trait_fns.read;
                *read_trait_fns = quote! {
                    #read_trait_fns
                    #other_read_trait_fns
                };
                let read_impl_fns = &mut impl_fns.read;
                let other_read_impl_fns = &other_impl_fns.read;
                *read_impl_fns = quote! {
                    #read_impl_fns
                    #other_read_impl_fns
                };
                let write_trait_fns = &mut trait_fns.write;
                let other_write_trait_fns = &other_trait_fns.write;
                *write_trait_fns = quote! {
                    #write_trait_fns
                    #other_write_trait_fns
                };
                let write_impl_fns = &mut impl_fns.write;
                let other_write_impl_fns = &other_impl_fns.write;
                *write_impl_fns = quote! {
                    #write_impl_fns
                    #other_write_impl_fns
                };
            }
            (
                Self::Dynamic {
                    trait_fns,
                    impl_fns,
                },
                Self::Dynamic {
                    trait_fns: other_trait_fns,
                    impl_fns: other_impl_fns,
                },
            ) => {
                let read_trait_fns = &mut trait_fns.read;
                let other_read_trait_fns = &other_trait_fns.read;
                *read_trait_fns = quote! {
                    #read_trait_fns
                    #other_read_trait_fns
                };
                let read_impl_fns = &mut impl_fns.read;
                let other_read_impl_fns = &other_impl_fns.read;
                *read_impl_fns = quote! {
                    #read_impl_fns
                    #other_read_impl_fns
                };
                let write_trait_fns = &mut trait_fns.write;
                let other_write_trait_fns = &other_trait_fns.write;
                *write_trait_fns = quote! {
                    #write_trait_fns
                    #other_write_trait_fns
                };
                let write_impl_fns = &mut impl_fns.write;
                let other_write_impl_fns = &other_impl_fns.write;
                *write_impl_fns = quote! {
                    #write_impl_fns
                    #other_write_impl_fns
                };
            }
            (
                Self::Slice {
                    trait_fns,
                    impl_fns,
                    struct_fns,
                },
                Self::Slice {
                    trait_fns: other_trait_fns,
                    impl_fns: other_impl_fns,
                    struct_fns: other_struct_fns,
                },
            ) => {
                let read_trait_fns = &mut trait_fns.read;
                let other_read_trait_fns = &other_trait_fns.read;
                *read_trait_fns = quote! {
                    #read_trait_fns
                    #other_read_trait_fns
                };
                let read_impl_fns = &mut impl_fns.read;
                let other_read_impl_fns = &other_impl_fns.read;
                *read_impl_fns = quote! {
                    #read_impl_fns
                    #other_read_impl_fns
                };
                let read_struct_fns = &mut struct_fns.read;
                let other_read_struct_fns = &other_struct_fns.read;
                *read_struct_fns = quote! {
                    #read_struct_fns
                    #other_read_struct_fns
                };
                let write_trait_fns = &mut trait_fns.write;
                let other_write_trait_fns = &other_trait_fns.write;
                *write_trait_fns = quote! {
                    #write_trait_fns
                    #other_write_trait_fns
                };
                let write_impl_fns = &mut impl_fns.write;
                let other_write_impl_fns = &other_impl_fns.write;
                *write_impl_fns = quote! {
                    #write_impl_fns
                    #other_write_impl_fns
                };
                let write_struct_fns = &mut struct_fns.write;
                let other_write_struct_fns = &other_struct_fns.write;
                *write_struct_fns = quote! {
                    #write_struct_fns
                    #other_write_struct_fns
                };
            }
            _ => {
                // Hex traits don't actually generate anything other than the trait impl which is 1 line.
            }
        }
    }
}

fn do_thing(input: proc_macro::TokenStream, flavor: GenerationFlavor) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // parse the input into a StructInfo which contains all the information we
    // along with some helpful structures to generate our Bitfield code.
    let struct_info = match GenericBuilder::parse(&input) {
        Ok(parsed_struct) => parsed_struct,
        Err(err) => {
            return proc_macro::TokenStream::from(err.to_compile_error());
        }
    };
    // println!("Solving: {}", struct_info.name());
    let solved: Solved = match struct_info.try_into() {
        Ok(s) => s,
        Err(err) => {
            return proc_macro::TokenStream::from(err.to_compile_error());
        }
    };
    // println!("Generating: {}", solved.name);
    match solved.gen(flavor) {
        Ok(gen) => gen.into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

#[proc_macro_derive(Bitfields, attributes(bondrewd,))]
pub fn derive_bitfields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_thing(input, GenerationFlavor::standard())
}

#[proc_macro_derive(BitfieldsSlice, attributes(bondrewd,))]
pub fn derive_bitfields_slice(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_thing(input, GenerationFlavor::slice())
}

#[proc_macro_derive(BitfieldsDyn, attributes(bondrewd,))]
pub fn derive_bitfields_dyn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_thing(input, GenerationFlavor::dynamic())
}
