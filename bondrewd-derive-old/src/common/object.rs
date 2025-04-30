use syn::Ident;

use super::{r#enum::Info as EnumInfo, r#struct::Info as StructInfo};

#[derive(Debug)]
pub enum Info {
    Struct(StructInfo),
    Enum(EnumInfo),
}

impl Info {
    // #[cfg(feature = "dyn_fns")]
    // pub fn vis(&self) -> &syn::Visibility {
    //     match self {
    //         Info::Struct(s) => &s.vis,
    //         Info::Enum(e) => &e.vis,
    //     }
    // }
    pub fn name(&self) -> Ident {
        match self {
            Info::Struct(s) => s.name.clone(),
            Info::Enum(e) => e.name.clone(),
        }
    }
    pub fn total_bits(&self) -> usize {
        match self {
            Self::Struct(s) => s.total_bits(),
            Self::Enum(info) => info.total_bits(),
        }
    }
    pub fn total_bits_no_fill(&self) -> usize {
        match self {
            Self::Struct(s) => s.total_bits_no_fill(),
            Self::Enum(info) => info.total_bits_no_fill(),
        }
    }
    pub fn total_bytes(&self) -> usize {
        self.total_bits().div_ceil(8)
    }
}
