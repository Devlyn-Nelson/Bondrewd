use quote::format_ident;

use crate::common::{
    field::{
        Endianness, FieldAttrs, FieldDataType, FieldInfo, NumberSignage, OverlapOptions,
        ReserveFieldOption,
    },
    r#enum::EnumInfo,
};

impl EnumInfo {
    pub fn generate_id_field(&self) -> syn::Result<FieldInfo> {
        let e = match &self.attrs.attrs.default_endianess {
            Endianness::None | Endianness::Little => Endianness::Little,
            Endianness::Big => Endianness::Big,
        };
        Ok(FieldInfo {
            ident: Box::new(format_ident!("{}", EnumInfo::VARIANT_ID_NAME).into()),
            ty: FieldDataType::Number(
                self.attrs.id_bits.div_ceil(8),
                NumberSignage::Unsigned,
                self.id_type_ident()?,
            ),
            attrs: FieldAttrs {
                endianness: Box::new(e),
                bit_range: 0..self.attrs.id_bits,
                reserve: ReserveFieldOption::NotReserve,
                overlap: OverlapOptions::None,
                capture_id: false,
            },
        })
    }
}
