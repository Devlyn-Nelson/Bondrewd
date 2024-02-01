

pub struct GeneratedFunctions {
    /// Functions that belong in `Bitfields` impl for object.
    pub bitfield_trait_impl_fns: proc_macro2::TokenStream,
    /// Functions that belong in impl for object.
    pub impl_fns: proc_macro2::TokenStream,
    /// Functions that belong in impl for generated checked slice object.
    pub checked_struct_impl_fns: proc_macro2::TokenStream,
    /// Functions that belong in `BitfieldsDyn` impl for object.
    #[cfg(feature = "dyn_fns")]
    pub bitfield_dyn_trait_impl_fns: proc_macro2::TokenStream,
}

// pub fn generate_functions_enum(info: &EnumInfo) -> Result<GeneratedFunctions, syn::Error> {
//     // function for getting the id of an enum.
//     let mut id_fn = quote! {};
//     let mut bitfield_trait_impl_fns = quote! {};
//     let mut impl_fns = quote! {};
//     #[cfg(feature = "dyn_fns")]
//     let mut bitfield_dyn_trait_impl_fns = quote! {};

//     let from = {
//         let field = info.generate_id_field()?;
//         let flip = false;
//         let field_extractor = get_field_quote(
//             &field,
//             if flip {
//                 // condition use to be `info.attrs.flip` i think this only applies to the variants
//                 // and id_position is what is used here. but it should be done none the less.
//                 Some(info.total_bytes() - 1)
//             } else {
//                 None
//             },
//         )?;
//         let attrs = info.attrs.attrs.clone();
//         let mut fields = vec![field.clone()];
//         fields[0].attrs.bit_range = 0..info.total_bits();
//         let temp_struct_info = StructInfo {
//             name: info.name.clone(),
//             attrs,
//             fields,
//             vis: syn::Visibility::Public(Pub::default()),
//             tuple: false,
//         };
//         let id_field = generate_read_field_fn(&field_extractor, &field, &temp_struct_info, &None);
//         #[cfg(feature = "dyn_fns")]
//         {
//             let id_slice_peek =
//                 generate_read_slice_field_fn(&field_extractor, &field, &temp_struct_info, &None);
//             quote! {
//                 #id_field
//                 #id_slice_peek
//             }
//         }
//         #[cfg(not(feature = "dyn_fns"))]
//         {
//             quote! {
//                 #id_field
//             }
//         }
//     };

//     let into = {
//         let (field_setter, clear_quote) = get_field_quote(
//             &field,
//             if flip {
//                 // condition use to be `info.attrs.flip` i think this only applies to the variants
//                 // and id_position is what is used here. but it should be done none the less.
//                 Some(info.total_bytes() - 1)
//             } else {
//                 None
//             },
//             false,
//         )?;
//         let id_field = generate_write_field_fn(
//             &field_setter,
//             &field,
//             &StructInfo {
//                 name: info.name.clone(),
//                 attrs,
//                 fields,
//                 vis: syn::Visibility::Public(Pub::default()),
//                 tuple: false,
//             },
//             &clear_quote,
//             &None,
//         );
//         let out = quote! {
//             #id_field
//         };
//         let out = {
//             let q = make_checked_mut_func(&info.name, info.total_bytes());
//             quote! {
//                 #out
//                 #q
//             }
//         };
//         out
//     };

//     todo!("finish merged (from AND into) generate functions");
// }
/// the flip value must be the total amount of bytes the result of `into_bytes` should have MINUS ONE,
/// the number is used to invert indices
// fn get_field_quotes(
//     field: &FieldInfo,
//     flip: Option<usize>,
//     with_self: bool,
// ) -> syn::Result<FieldQuotes> {
//     let field_name = field.ident().name();
//     let quote_field_name = match field.ty {
//         FieldDataType::Float(_, _) => {
//             if with_self {
//                 quote! {self.#field_name.to_bits()}
//             } else {
//                 quote! {#field_name.to_bits()}
//             }
//         }
//         FieldDataType::Char(_, _) => {
//             if with_self {
//                 quote! {(self.#field_name as u32)}
//             } else {
//                 quote! {(#field_name as u32)}
//             }
//         }
//         FieldDataType::Enum(_, _, _) => {
//             if with_self {
//                 quote! {((self.#field_name).into_primitive())}
//             } else {
//                 quote! {((#field_name).into_primitive())}
//             }
//         }
//         FieldDataType::ElementArray(_, _, _) => {
//             let mut clear_buffer = quote! {};
//             let mut buffer = quote! {};
//             let mut de_refs: Punctuated<IdentSyn, Comma> = Punctuated::default();
//             let outer_field_name = &field.ident().ident();
//             let sub = field.get_element_iter()?;
//             for sub_field in sub {
//                 let field_name = &sub_field.ident().name();
//                 let (sub_field_quote, clear) = get_field_quote(&sub_field, flip, with_self)?;
//                 buffer = quote! {
//                     #buffer
//                     #sub_field_quote
//                 };
//                 clear_buffer = quote! {
//                     #clear_buffer
//                     #clear
//                 };
//                 de_refs.push(format_ident!("{}", field_name));
//             }
//             buffer = quote! {
//                 let [#de_refs] = #outer_field_name;
//                 #buffer
//             };
//             return Ok((buffer, clear_buffer));
//         }
//         FieldDataType::BlockArray(_, _, _) => {
//             let mut buffer = quote! {};
//             let mut clear_buffer = quote! {};
//             let mut de_refs: Punctuated<IdentSyn, Comma> = Punctuated::default();
//             let outer_field_name = &field.ident().ident();
//             let sub = field.get_block_iter()?;
//             for sub_field in sub {
//                 let field_name = &sub_field.ident().name();
//                 let (sub_field_quote, clear) = get_field_quote(&sub_field, flip, with_self)?;
//                 buffer = quote! {
//                     #buffer
//                     #sub_field_quote
//                 };
//                 clear_buffer = quote! {
//                     #clear_buffer
//                     #clear
//                 };
//                 de_refs.push(format_ident!("{}", field_name));
//             }
//             buffer = quote! {
//                 let [#de_refs] = #outer_field_name;
//                 #buffer
//             };
//             return Ok((buffer, clear_buffer));
//         }
//         _ => {
//             if with_self {
//                 quote! {self.#field_name}
//             } else {
//                 quote! {#field_name}
//             }
//         }
//     };
//     match field.attrs.endianness.as_ref() {
//         Endianness::Big => apply_be_math_to_field_access_quote(field, quote_field_name, flip),
//         Endianness::Little => apply_le_math_to_field_access_quote(field, quote_field_name, flip),
//         Endianness::None => apply_ne_math_to_field_access_quote(field, &quote_field_name, flip),
//     }
// }