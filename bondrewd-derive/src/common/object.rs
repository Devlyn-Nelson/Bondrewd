use syn::Ident;

use super::{r#enum::EnumInfo, r#struct::StructInfo};

pub enum ObjectInfo {
    Struct(StructInfo),
    Enum(EnumInfo),
}

impl ObjectInfo {
    // #[cfg(feature = "dyn_fns")]
    // pub fn vis(&self) -> &syn::Visibility {
    //     match self {
    //         ObjectInfo::Struct(s) => &s.vis,
    //         ObjectInfo::Enum(e) => &e.vis,
    //     }
    // }
    pub fn name(&self) -> Ident {
        match self {
            ObjectInfo::Struct(s) => s.name.clone(),
            ObjectInfo::Enum(e) => e.name.clone(),
        }
    }
    pub fn total_bits(&self) -> usize {
        match self {
            Self::Struct(s) => s.total_bits(),
            Self::Enum(info) => info.total_bits(),
        }
    }
    pub fn total_bytes(&self) -> usize {
        self.total_bits().div_ceil(8)
    }
}
