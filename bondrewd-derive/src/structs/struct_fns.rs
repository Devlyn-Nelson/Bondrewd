use crate::structs::common::{FieldDataType, FieldInfo, NumberSignage, StructInfo};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn create_setters_quotes(info: &StructInfo) -> Result<TokenStream, syn::Error> {
    // all of the fields set functions that disallow numbers that are too large to fit into bit length.
    let mut set_fns_quote = quote! {};
    for field in info.fields.iter() {
        let q = make_set_field_quote(field)?;
        set_fns_quote = quote! {
            #set_fns_quote
            #q
        };
    }
    Ok(set_fns_quote)
}

fn make_set_field_quote(field: &FieldInfo) -> Result<TokenStream, syn::Error> {
    let field_name = field.ident().ident();
    Ok(match field.ty {
        FieldDataType::Number(ref size, ref sign, ref type_ident) => {
            let mut full_quote = quote! {
                self.#field_name = value.clone();
                value
            };
            let bit_length = field.bit_size();
            if bit_length != size * 8 {
                match sign {
                    NumberSignage::Signed => {
                        let max: i128 = ((2_u128.pow(bit_length as u32) / 2_u128) - 1) as i128;
                        let min = -max - 1;
                        let max_lit = proc_macro2::Literal::i128_unsuffixed(max);
                        let min_lit = proc_macro2::Literal::i128_unsuffixed(min);
                        full_quote = quote! {
                            if value > #max_lit {
                                self.#field_name = #max_lit;
                                #max_lit
                            } else if value < #min_lit {
                                self.#field_name = #min_lit;
                                #min_lit
                            } else {
                                #full_quote
                            }
                        };
                    }
                    NumberSignage::Unsigned => {
                        let max: u128 = 2_u128.pow(bit_length as u32) - 1;
                        let max_lit = proc_macro2::Literal::u128_unsuffixed(max);
                        full_quote = quote! {
                            if value > #max_lit {
                                self.#field_name = #max_lit;
                                #max_lit
                            }else {
                                #full_quote
                            }
                        };
                    }
                }
            }
            let field_fn_name = format_ident!("set_{}", field_name);
            quote! {
                fn #field_fn_name(&mut self, value: #type_ident) -> #type_ident {
                    #full_quote
                }
            }
        }
        FieldDataType::Float(ref size, ref type_ident) => {
            let mut full_quote = quote! {
                self.#field_name = value;
                value
            };
            let bit_length = field.bit_size();
            if bit_length != *size * 8_usize {
                let max: f64 = if *size == 4_usize {
                    f32::MAX as f64
                } else if *size == 8_usize {
                    f64::MAX
                } else {
                    return Err(syn::Error::new(
                        field.span(),
                        "unsupported floating point size",
                    ));
                };
                let min: f64 = if *size == 4_usize {
                    f32::MIN as f64
                } else if *size == 8_usize {
                    f64::MIN
                } else {
                    return Err(syn::Error::new(
                        field.span(),
                        "unsupported floating point size",
                    ));
                };
                let max_lit = proc_macro2::Literal::f64_unsuffixed(max);
                let min_lit = proc_macro2::Literal::f64_unsuffixed(min);
                full_quote = quote! {
                    if value > #max_lit {
                        self.#field_name = #max_lit;
                        #max_lit
                    }else if value < #min_lit {
                        self.#field_name = #min_lit;
                        #min_lit
                    }else {
                        #full_quote
                    }
                };
            }
            let field_fn_name = format_ident!("set_{}", field_name);
            quote! {
                fn #field_fn_name(&mut self, value: #type_ident) -> #type_ident {
                    #full_quote
                }
            }
        }
        FieldDataType::Enum(_, _, ref type_ident) => {
            let field_fn_name = format_ident!("set_{}", field_name);
            quote! {
                fn #field_fn_name(&mut self, value: #type_ident) {
                    self.value;
                }
            }
        }
        FieldDataType::Struct(_, ref type_ident) => {
            let field_fn_name = format_ident!("{}_mut_ref", field_name);
            quote! {
                fn #field_fn_name(&mut self) -> &mut #type_ident {
                    &mut self.#field_name
                }
            }
        }
        FieldDataType::Char(ref size, ref type_ident) => {
            let mut full_quote = quote! {
                self.#field_name = value.clone();
                value
            };
            let bit_length = field.bit_size();
            if bit_length != size * 8 {
                let max: u32 = 2_u32.pow(bit_length as u32) - 1;
                full_quote = quote! {
                    if value as u32 > #max {
                        self.#field_name = #max;
                        #max
                    }else {
                        #full_quote
                    }
                };
            }
            let field_fn_name = format_ident!("set_{}", field_name);
            quote! {
                fn #field_fn_name(&mut self, value: #type_ident) -> #type_ident {
                    #full_quote
                }
            }
        }
        FieldDataType::ElementArray(_, _, _) => {
            quote! {}
        }
        FieldDataType::BlockArray(_, _, _) => {
            quote! {}
        }
        FieldDataType::Boolean => {
            let field_fn_name = format_ident!("set_{}", field_name);
            quote! {
                fn #field_fn_name(&mut self, value: bool) {
                    self.#field_name = value;
                }
            }
        }
    })
}
