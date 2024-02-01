use std::{cmp::Ordering, collections::VecDeque};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma};

use super::gen_field::QuoteInfo;
use crate::structs::common::{get_left_and_mask, get_right_and_mask, Endianness, FieldDataType, FieldInfo, NumberSignage};

fn isolate_bit_index_mask(bit_index: usize) -> u8 {
    match bit_index {
        1 => 0b0100_0000,
        2 => 0b0010_0000,
        3 => 0b0001_0000,
        4 => 0b0000_1000,
        5 => 0b0000_0100,
        6 => 0b0000_0010,
        7 => 0b0000_0001,
        _ => 0b1000_0000,
    }
}
fn rotate_primitive_vec(prim: Vec<u8>, right_shift: i8, field: &FieldInfo) -> syn::Result<Vec<u8>> {
    // REMEMBER SHIFTS ARE BACKWARD BECAUSE YOU COPIED AND PASTED into_bytes
    if right_shift == 0 {
        return Ok(prim);
    }
    let output = match prim.len() {
        1 => {
            let mut temp = u8::from_be_bytes([prim[0]]);
            match right_shift {
                i8::MIN..=-1 => {
                    let left_shift = -right_shift;
                    temp = temp.rotate_left(u32::from(left_shift.unsigned_abs()));
                }
                0..=i8::MAX => {
                    temp = temp.rotate_right(u32::from(right_shift.unsigned_abs()));
                }
            }
            temp.to_be_bytes().to_vec()
        }
        2 => {
            let mut temp = u16::from_be_bytes([prim[0], prim[1]]);
            match right_shift {
                i8::MIN..=-1 => {
                    let left_shift = -right_shift;
                    temp = temp.rotate_left(u32::from(left_shift.unsigned_abs()));
                }
                0..=i8::MAX => {
                    temp = temp.rotate_right(u32::from(right_shift.unsigned_abs()));
                }
            }
            temp.to_be_bytes().to_vec()
        }
        4 => {
            let mut temp = u32::from_be_bytes([prim[0], prim[1], prim[2], prim[3]]);
            match right_shift {
                i8::MIN..=-1 => {
                    let left_shift = -right_shift;
                    temp = temp.rotate_left(u32::from(left_shift.unsigned_abs()));
                }
                0..=i8::MAX => {
                    temp = temp.rotate_right(u32::from(right_shift.unsigned_abs()));
                }
            }
            temp.to_be_bytes().to_vec()
        }
        8 => {
            let mut temp = u64::from_be_bytes([
                prim[0], prim[1], prim[2], prim[3], prim[4], prim[5], prim[6], prim[7],
            ]);
            match right_shift {
                i8::MIN..=-1 => {
                    let left_shift = -right_shift;
                    temp = temp.rotate_left(u32::from(left_shift.unsigned_abs()));
                }
                0..=i8::MAX => {
                    temp = temp.rotate_right(u32::from(right_shift.unsigned_abs()));
                }
            }
            temp.to_be_bytes().to_vec()
        }
        16 => {
            let mut temp = u128::from_be_bytes([
                prim[0], prim[1], prim[2], prim[3], prim[4], prim[5], prim[6], prim[7], prim[8],
                prim[9], prim[10], prim[11], prim[12], prim[13], prim[14], prim[15],
            ]);
            match right_shift {
                i8::MIN..=-1 => {
                    let left_shift = -right_shift;
                    temp = temp.rotate_left(u32::from(left_shift.unsigned_abs()));
                }
                0..=i8::MAX => {
                    temp = temp.rotate_right(u32::from(right_shift.unsigned_abs()));
                }
            }
            temp.to_be_bytes().to_vec()
        }
        _ => {
            return Err(syn::Error::new(
                field.ident.span(),
                "invalid primitive size",
            ));
        }
    };
    Ok(output)
}

fn add_sign_fix_quote(
    field: &FieldInfo,
    amount_of_bits: usize,
    right_shift: i8,
) -> syn::Result<Option<TokenStream>> {
    if let FieldDataType::Number(ref size, ref sign, _) = field.ty {
        if amount_of_bits != size * 8 {
            if let NumberSignage::Signed = sign {
                let (bit_to_isolate, sign_index) = match field.attrs.endianness.as_ref() {
                    Endianness::Big => (
                        field.attrs.bit_range.start % 8,
                        field.attrs.bit_range.start / 8,
                    ),
                    Endianness::Little => {
                        let skip_bytes = (amount_of_bits / 8) * 8;
                        let sign_bit_index = field.attrs.bit_range.start + skip_bytes;
                        // TODO fix bit isolators to fix signed numbers.
                        (sign_bit_index % 8, sign_bit_index / 8)
                    }
                    Endianness::None => return Ok(None),
                };
                let sign_mask = isolate_bit_index_mask(bit_to_isolate);
                let sign_bit = quote! {
                    (input_byte_buffer[#sign_index] & #sign_mask)
                };
                let mut unused_bits = (size * 8) - amount_of_bits;
                let mut buffer: VecDeque<u8> = VecDeque::default();
                for _i in 0..*size {
                    if unused_bits > 7 {
                        buffer.push_back(get_left_and_mask(8));
                        unused_bits -= 8;
                    } else if unused_bits != 0 {
                        buffer.push_back(get_left_and_mask(unused_bits));
                        unused_bits = 0;
                    } else {
                        buffer.push_back(get_left_and_mask(0));
                    }
                }
                let mut bit_buffer: Punctuated<u8, Comma> = Punctuated::default();
                match field.attrs.endianness.as_ref() {
                    Endianness::Big => {
                        buffer = VecDeque::from(rotate_primitive_vec(
                            buffer.into(),
                            right_shift,
                            field,
                        )?);
                        while {
                            if let Some(c) = buffer.pop_front() {
                                bit_buffer.push(c);
                                true
                            } else {
                                false
                            }
                        } {}
                    }
                    Endianness::Little => {
                        match right_shift.cmp(&0) {
                            Ordering::Greater => {
                                buffer = buffer
                                    .into_iter()
                                    .map(|x| x.rotate_right(u32::from(right_shift.unsigned_abs())))
                                    .collect();
                            }
                            Ordering::Less => {
                                let left_shift = u32::from(right_shift.unsigned_abs());
                                buffer = buffer
                                    .into_iter()
                                    .map(|x| x.rotate_left(left_shift))
                                    .collect();
                            }
                            Ordering::Equal => {}
                        }
                        while {
                            if let Some(c) = buffer.pop_back() {
                                bit_buffer.push(c);
                                true
                            } else {
                                false
                            }
                        } {}
                    }
                    Endianness::None => return Ok(None),
                }
                return Ok(Some(quote! {
                    if #sign_bit == #sign_mask {[#bit_buffer]} else {[0u8;#size]}
                }));
            }
        }
    }
    Ok(None)
}

fn add_sign_fix_quote_single_bit(
    field_access: TokenStream,
    field: &FieldInfo,
    amount_of_bits: usize,
    byte_index: usize,
) -> TokenStream {
    if let FieldDataType::Number(ref size, ref sign, _) = field.ty {
        if amount_of_bits != *size * 8 {
            if let NumberSignage::Signed = sign {
                let bit_to_isolate = field.attrs.bit_range.start % 8;
                let sign_mask = isolate_bit_index_mask(bit_to_isolate);
                let neg_mask = get_left_and_mask(bit_to_isolate + 1);
                let sign_bit = quote! {
                    (input_byte_buffer[#byte_index] & #sign_mask)
                };
                let add_me = quote! {
                    if #sign_bit == #sign_mask {#neg_mask | #field_access} else {0u8 | #field_access}
                };
                return add_me;
            }
        }
    }
    field_access
}

impl FieldInfo {
    pub fn get_read_le_single_byte_quote(
        &self,
        quote_info: &QuoteInfo,
    ) -> syn::Result<TokenStream> {
        // TODO make multi-byte values that for some reason use less then 9 bits work in here.
        // currently only u8 and i8 fields will work here. verify bool works it might.
        // amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the
        // left)
        if 8 < (quote_info.zeros_on_left() + quote_info.amount_of_bits()) {
            return Err(syn::Error::new(
                self.ident.span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (quote_info.zeros_on_left() + quote_info.amount_of_bits());
        // combining the left and right masks will give us a mask that keeps the amount og bytes we
        // have in the position we need them to be in for this byte. we use available_bytes for
        // right mask because param is amount of 1's on the side specified (right), and
        // available_bytes is (8 - zeros_on_left) which is equal to ones_on_right.
        let mask = get_right_and_mask(quote_info.available_bits_in_first_byte())
            & get_left_and_mask(8 - zeros_on_right);
        // calculate how many left shifts need to occur to the number in order to position the bytes
        // we want to keep in the position we want.
        if 8 - quote_info.amount_of_bits() < self.attrs.bit_range.start % 8 {
            return Err(syn::Error::new(
                self.ident.span(),
                format!(
                    "calculating be left_shift failed {} , {}",
                    quote_info.amount_of_bits(),
                    self.attrs.bit_range.start % 8
                ),
            ));
        }
        let shift_left = (8 - quote_info.amount_of_bits()) - (self.attrs.bit_range.start % 8);
        // a quote that puts the field into a byte buffer we assume exists (because this is a
        // fragment).
        // NOTE the mask used here is only needed if we can NOT guarantee the field is only using the
        // bits the size attribute says it can. for example if our field is a u8 but the bit_length
        // attribute say to only use 2 bits, then the possible values are 0-3. so if the u8 (0-255)
        // is set to 4 then the extra bit being used will be dropped by the mask making the value 0.
        // FEATURE remove the "#mask & " from this quote to make it faster. but that means the
        // numbers have to be correct. if you want the no-mask feature then suggested enforcement of
        // the number would be:
        //      - generate setters that make a mask that drops bits not desired. (fast)
        //      - generate setters that check if its above max_value for the bit_length and set it
        //          to the max_value if its larger. (prevents situations like the 2bit u8 example
        //          in the note above)
        // both of these could benefit from a return of the number that actually got set.
        let starting_inject_byte = quote_info.starting_inject_byte();
        let field_buffer_name = quote_info.field_buffer_name();
        let output_quote = match self.ty {
            FieldDataType::Number(_, ref sign, ref ident) => {
                let mut field_value = quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left)};
                if let NumberSignage::Signed = sign {
                    field_value = add_sign_fix_quote_single_bit(field_value, self, quote_info.amount_of_bits(), starting_inject_byte);
                    let mut value = quote!{
                        let mut #field_buffer_name = #field_value;
                    };
                    value = quote!{
                        {
                            #value
                            #field_buffer_name as #ident
                        }
                    };
                    value
                }else{
                    quote!{
                        #field_value as #ident
                    }
                }
            }
            FieldDataType::Boolean => {
                quote!{(input_byte_buffer[#starting_inject_byte] & #mask) != 0}
            }
            FieldDataType::Char(_, _) => quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left) as u32},
            FieldDataType::Enum(ref ident, _, _) => quote!{((input_byte_buffer[#starting_inject_byte] & #mask) >> #shift_left) as #ident},
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(self.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Float(_, _) => return Err(syn::Error::new(self.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad.")),
        };
        Ok(output_quote)
    }
    pub fn get_read_le_multi_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        right_shift: i8,
        first_bit_mask: u8,
        last_bit_mask: u8,
    ) -> syn::Result<TokenStream> {
        let rust_type_size = self.ty.size();
        // Allocate a buffer to put the bits into as we extract them. if signed we need to be able fill
        // the unused bits with zero or 1 depending on if it is negative when read.
        let new_array_quote =
            if let Some(a) = add_sign_fix_quote(self, quote_info.amount_of_bits(), right_shift)? {
                a
            } else {
                quote! {[0u8;#rust_type_size]}
            };
        let field_buffer_name = &quote_info.field_buffer_name();
        // Create the full buffer initiation quotes.
        let mut full_quote = quote! {
            let mut #field_buffer_name: [u8;#rust_type_size] = #new_array_quote;
        };
        // generate the reading code. the while loop will do all except for the last byte.
        let fields_last_bits_index = quote_info.fields_last_bits_index();
        let current_bit_mask = get_right_and_mask(quote_info.available_bits_in_first_byte());
        #[allow(clippy::cast_possible_truncation)]
        let mid_shift: u32 = 8 - quote_info.available_bits_in_first_byte() as u32;
        let next_bit_mask = get_left_and_mask(mid_shift as usize);
        let mut i = 0;
        while i != fields_last_bits_index {
            let start = quote_info.offset_starting_inject_byte(i);
            if quote_info.available_bits_in_first_byte() == 0 && right_shift == 0 {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#start];
                };
            } else {
                if current_bit_mask == u8::MAX {
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] |= input_byte_buffer[#start];
                    };
                } else if current_bit_mask != 0 {
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] |= input_byte_buffer[#start] & #current_bit_mask;
                    };
                }
                if quote_info.available_bits_in_first_byte() + (8 * i) < quote_info.amount_of_bits()
                    && next_bit_mask != 0
                {
                    let next_index = quote_info.next_index(start);
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] |= input_byte_buffer[#next_index] & #next_bit_mask;
                    }
                }
                if mid_shift != 0 {
                    full_quote = quote! {
                        #full_quote
                        #field_buffer_name[#i] = #field_buffer_name[#i].rotate_left(#mid_shift);
                    };
                }
            }
            i += 1;
        }
        // finish read the last byte's bits.
        let used_bits = quote_info.available_bits_in_first_byte() + (8 * i);
        if right_shift > 0 {
            let start = quote_info.offset_starting_inject_byte(i);
            let right_shift: u32 = u32::from(right_shift.unsigned_abs());
            if used_bits < quote_info.amount_of_bits() {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#start] & #current_bit_mask;
                    #field_buffer_name[#i] |= input_byte_buffer[#start + 1] & #last_bit_mask;
                };
            } else {
                let mut last_mask = first_bit_mask;
                if quote_info.amount_of_bits() < used_bits {
                    last_mask &= !get_right_and_mask(used_bits - quote_info.amount_of_bits());
                }
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= input_byte_buffer[#start] & #last_mask;
                };
            }
            full_quote = quote! {
                #full_quote
                #field_buffer_name[#i] = #field_buffer_name[#i].rotate_left(#right_shift);
            };
        } else {
            let start = quote_info.offset_starting_inject_byte(i);
            // this should give us the last index of the field
            let left_shift: u32 = u32::from(right_shift.unsigned_abs());
            let mid_mask = first_bit_mask & last_bit_mask;
            if mid_mask == u8::MAX {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= (input_byte_buffer[#start]);
                    #field_buffer_name[#i] = #field_buffer_name[#i].rotate_right(#left_shift);
                };
            } else {
                full_quote = quote! {
                    #full_quote
                    #field_buffer_name[#i] |= (input_byte_buffer[#start] & #mid_mask);
                    #field_buffer_name[#i] = #field_buffer_name[#i].rotate_right(#left_shift);
                };
            }
        }
        full_quote = quote! {
            #full_quote
            #field_buffer_name
        };
        // generate code to transform buffer into rust type.
        let output = match self.ty {
            FieldDataType::Number(_, _, ref type_quote) |
            FieldDataType::Enum(ref type_quote, _, _) => {
                let apply_field_to_buffer = quote! {
                    #type_quote::from_le_bytes({
                        #full_quote
                    })
                };
                apply_field_to_buffer
            }
            FieldDataType::Float(_, _) => {
                let alt_type_quote = if rust_type_size == 4 {
                    quote!{u32}
                }else if rust_type_size == 8 {
                    quote!{u64}
                }else{
                    return Err(syn::Error::new(self.ident.span(), "unsupported floating type"))
                };
                let apply_field_to_buffer = quote! {
                    #alt_type_quote::from_le_bytes({
                        #full_quote
                    })
                };
                apply_field_to_buffer
            }
            FieldDataType::Char(_, _) => {
                let apply_field_to_buffer = quote! {
                    u32::from_le_bytes({
                        #full_quote
                    })
                };
                apply_field_to_buffer
            }
            FieldDataType::Boolean => return Err(syn::Error::new(self.ident.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Struct(_, _) => return Err(syn::Error::new(self.ident.span(), "Struct was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_be_math_to_field_access_quote, which is bad."))
        };

        Ok(output)
    }
    pub fn get_read_ne_single_byte_quote(
        &self,
        quote_info: &QuoteInfo,
    ) -> syn::Result<TokenStream> {
        if 8 < (quote_info.zeros_on_left() + quote_info.amount_of_bits()) {
            return Err(syn::Error::new(
                self.ident.span(),
                "calculating zeros_on_right failed",
            ));
        }
        let zeros_on_right = 8 - (quote_info.zeros_on_left() + quote_info.amount_of_bits());
        let mask = get_right_and_mask(quote_info.available_bits_in_first_byte())
            & get_left_and_mask(8 - zeros_on_right);
        if 8 < quote_info.amount_of_bits()
            || 8 - quote_info.amount_of_bits() < self.attrs.bit_range.start % 8
        {
            return Err(syn::Error::new(
                self.ident.span(),
                "calculating ne shift_left failed",
            ));
        }
        let starting_inject_byte = quote_info.starting_inject_byte();
        let output = match self.ty {
            FieldDataType::Number(_, _, _) => return Err(syn::Error::new(self.ident.span(), "Number was not given Endianness, please report this")),
            FieldDataType::Boolean => {
                quote!{(((input_byte_buffer[#starting_inject_byte] & #mask)) != 0)}
            }
            FieldDataType::Char(_, _) => return Err(syn::Error::new(self.ident.span(), "Char not supported for single byte insert logic")),
            FieldDataType::Enum(_, _, _) => return Err(syn::Error::new(self.ident.span(), "Enum was given Endianness which should be described by the struct implementing Bitfield")),
            FieldDataType::Struct(_, _) => {
                let used_bits_in_byte = 8 - quote_info.available_bits_in_first_byte();
                quote!{([((input_byte_buffer[#starting_inject_byte] & #mask)) << #used_bits_in_byte])}
            }
            FieldDataType::Float(_, _) => return Err(syn::Error::new(self.ident.span(), "Float not supported for single byte insert logic")),
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };
        Ok(output)
    }
    pub fn get_read_ne_multi_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        right_shift: i8,
    ) -> syn::Result<TokenStream> {
        // here we finish the buffer setup and give it the value returned by to_bytes from the number
        let full_quote = match self.ty {
            FieldDataType::Number(_, _,_ ) |
            FieldDataType::Float(_, _) |
            FieldDataType::Char(_, _) => return Err(syn::Error::new(self.ident.span(), "Char was not given Endianness, please report this.")),
            FieldDataType::Boolean => return Err(syn::Error::new(self.ident.span(), "matched a boolean data type in generate code for bits that span multiple bytes in the output")),
            FieldDataType::Enum(_, _, _) => return Err(syn::Error::new(self.ident.span(), "Enum was not given Endianness, please report this.")),
            FieldDataType::Struct(ref size, _) => {
                let buffer_ident = format_ident!("{}_buffer", self.ident().ident());
                let mut quote_builder = quote!{let mut #buffer_ident: [u8;#size] = [0u8;#size];};
                match right_shift.cmp(&0) {
                    Ordering::Greater => {
                        // right shift (this means that the last bits are in the first byte)
                        // because we are applying bits in place we need masks in insure we don't effect other fields
                        // data. we need one for the first byte and the last byte.
                        let current_bit_mask = get_right_and_mask(quote_info.available_bits_in_first_byte());
                        let next_bit_mask = get_left_and_mask(8 - quote_info.available_bits_in_first_byte());
                        let right_shift: u32 = u32::from(right_shift.unsigned_abs());
                        for i in 0..*size {
                            let start = quote_info.offset_starting_inject_byte(i);
                            let mut first = if current_bit_mask == u8::MAX {
                                quote!{
                                    #buffer_ident[#i] = input_byte_buffer[#start];
                                }
                            }else{
                                quote!{
                                    #buffer_ident[#i] = input_byte_buffer[#start] & #current_bit_mask;
                                }
                            };
                            if quote_info.available_bits_in_first_byte() + (8 * i) < quote_info.amount_of_bits() && next_bit_mask != 0 {
                                let next_index = quote_info.next_index(start);
                                first = quote!{
                                    #first
                                    #buffer_ident[#i] |= input_byte_buffer[#next_index] & #next_bit_mask;
                                };
                            }
                            quote_builder = quote!{
                                #quote_builder
                                #first
                                #buffer_ident[#i] = #buffer_ident[#i].rotate_left(#right_shift);
                            };
                        }
                    }
                    Ordering::Less => {
                        return Err(syn::Error::new(
                            self.ident.span(),
                            "left shifting struct was removed to see if it would ever happen",
                        ));
                        //TODO this might be impossible for structs
                        // left shift (this means that the last bits are in the first byte)
                        // because we are applying bits in place we need masks in insure we don't effect other fields
                        // data. we need one for the first byte and the last byte.
                        /*let current_bit_mask = get_right_and_mask(available_bits_in_first_byte);
                        let next_bit_mask = get_left_and_mask(8 - available_bits_in_first_byte);
                        let left_shift = right_shift.clone().abs() as u32;
                        for i in 0..*size {
                            let start = if let None = flip {starting_inject_byte + i}else{starting_inject_byte - i};
                            let mut first = quote!{
                                #buffer_ident[#i] = input_byte_buffer[#start] & #current_bit_mask;
                            };
                            if i + 1 <= *size {
                                first = quote!{
                                    #first
                                    #buffer_ident[#i] = input_byte_buffer[#start #operator 1] & #next_bit_mask;
                                };
                            }
                            quote_builder = quote!{
                                #quote_builder
                                #first
                                #buffer_ident[#i] = #buffer_ident[#i].rotate_right(#left_shift);
                            };
                        }*/
                    }
                    Ordering::Equal => {
                        // no shift can be more faster.
                        let current_bit_mask = get_right_and_mask(quote_info.available_bits_in_first_byte());
                        for i in 0..*size {
                            let start = quote_info.offset_starting_inject_byte(i);
                            if i == 0{
                                quote_builder = quote!{
                                    #quote_builder
                                    #buffer_ident[#i] = input_byte_buffer[#start] & #current_bit_mask;
                                };
                            }else{
                                quote_builder = quote!{
                                    #quote_builder
                                    #buffer_ident[#i] = input_byte_buffer[#start];
                                }
                            }
                        }
                    }
                }
                // return the value
                quote_builder = quote!{
                    #quote_builder
                    #buffer_ident
                };
                quote_builder
            }
            FieldDataType::ElementArray(_, _, _) | FieldDataType::BlockArray(_, _, _) => return Err(syn::Error::new(self.ident.span(), "an array got passed into apply_ne_math_to_field_access_quote, which is bad."))
        };

        Ok(full_quote)
    }
    pub fn get_read_be_single_byte_quote(
        &self,
        quote_info: &QuoteInfo,
    ) -> syn::Result<TokenStream> {
        todo!("write be generate fns")
    }
    pub fn get_read_be_multi_byte_quote(
        &self,
        quote_info: &QuoteInfo,
        right_shift: i8,
        first_bit_mask: u8,
        last_bit_mask: u8,
    ) -> syn::Result<TokenStream> {
        todo!("write be generate fns")
    }
}
