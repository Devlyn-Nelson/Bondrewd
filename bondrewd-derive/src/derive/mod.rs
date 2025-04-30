use std::ops::Range;

use crate::{
    build::ReserveFieldOption,
    solved::{
        field::{ResolverData, SolvedData},
        field_set::SolvedFieldSet,
    },
};

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Visibility};

mod from;
mod into;
pub mod quotes;
use quotes::*;
/// Stores [`TokenStream`] that contain the access (write/read/clear) code for a field.
pub struct GeneratedQuotes {
    pub(crate) read: proc_macro2::TokenStream,
    pub(crate) write: proc_macro2::TokenStream,
    pub(crate) zero: proc_macro2::TokenStream,
}
impl GeneratedQuotes {
    /// Returns the quote that reads a value from bytes
    pub fn read(&self) -> &proc_macro2::TokenStream {
        &self.read
    }
    /// Returns the quote that write a value to bytes
    pub fn write(&self) -> &proc_macro2::TokenStream {
        &self.write
    }
    /// Returns the quote that set the bytes this field are in to zero. (clears the bits so writes can work on dirty set of bits that already had a value)
    pub fn zero(&self) -> &proc_macro2::TokenStream {
        &self.zero
    }
}

/// Returns a u8 mask with provided `num` amount of 1's on the left side (most significant bit)
#[must_use]
pub fn get_left_and_mask(num: usize) -> u8 {
    match num {
        8 => 0b1111_1111,
        7 => 0b1111_1110,
        6 => 0b1111_1100,
        5 => 0b1111_1000,
        4 => 0b1111_0000,
        3 => 0b1110_0000,
        2 => 0b1100_0000,
        1 => 0b1000_0000,
        _ => 0b0000_0000,
    }
}

/// Returns a u8 mask with provided `num` amount of 1's on the right side (least significant bit)
#[must_use]
pub fn get_right_and_mask(num: usize) -> u8 {
    match num {
        8 => 0b1111_1111,
        7 => 0b0111_1111,
        6 => 0b0011_1111,
        5 => 0b0001_1111,
        4 => 0b0000_1111,
        3 => 0b0000_0111,
        2 => 0b0000_0011,
        1 => 0b0000_0001,
        _ => 0b0000_0000,
    }
}

/// calculate the starting bit index for a field.
///
/// Returns the index of the byte the first bits of the field
///
/// # Arguments
/// * `amount_of_bits` - amount of bits the field will be after `into_bytes`.
/// * `right_rotation` - amount of bit Rotations to preform on the field. Note if rotation is not needed
///                         to retain all used bits then a shift could be used.
/// * `last_index` - total struct bytes size minus 1.
#[inline]
#[expect(
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
pub fn get_be_starting_index(
    amount_of_bits: usize,
    right_rotation: i8,
    last_index: usize,
) -> Result<usize, String> {
    let first = ((amount_of_bits as f64 - right_rotation as f64) / 8.0f64).ceil() as usize;
    if last_index < first {
        // TODO figure out a better message for this error. its confusing. and no, i don't know what it means.
        Err(format!("Failed getting the starting index for big endianness, aob: {amount_of_bits}, rr: {right_rotation}, li: {last_index}, f: {first}."))
    } else {
        Ok(last_index - first)
    }
}

impl ResolverData {
    /// Returns the next byte index in sequence based of the given `index` and whether or not the Structure in has a reverse bytes order.
    #[must_use]
    pub fn next_index(&self, index: usize) -> usize {
        if self.flip.is_some() {
            index - 1
        } else {
            index + 1
        }
    }
    /// Returns the `starting_inject_byte` plus or minus `offset` depending on if the bytes order is reversed.
    #[must_use]
    pub fn offset_starting_inject_byte(&self, offset: usize) -> usize {
        if self.flip.is_some() {
            self.starting_inject_byte - offset
        } else {
            self.starting_inject_byte + offset
        }
    }
    #[must_use]
    pub fn fields_last_bits_index(&self) -> usize {
        self.bit_range_end().div_ceil(8) - 1
    }
    #[must_use]
    pub fn bit_range(&self) -> &Range<usize> {
        &self.bit_range
    }
    #[must_use]
    pub fn bit_range_start(&self) -> usize {
        self.bit_range.start
    }
    #[must_use]
    pub fn bit_range_end(&self) -> usize {
        self.bit_range.end
    }
    /// Pure bit length calculation
    #[must_use]
    pub fn bit_length(&self) -> usize {
        self.bit_range.end - self.bit_range.start
    }

    pub fn flip(&self) -> Option<&usize> {
        self.flip.as_ref()
    }
}

pub struct ResolverDataLittleAdditive {
    pub right_shift: i8,
    pub first_bit_mask: u8,
    pub last_bit_mask: u8,
}
impl From<&ResolverData> for ResolverDataLittleAdditive {
    fn from(qi: &ResolverData) -> Self {
        let amount_of_bits = qi.bit_length();
        let bits_in_last_byte = (amount_of_bits - qi.available_bits_in_first_byte) % 8;
        // how many times to shift the number right.
        // NOTE if negative shift left.
        // NOTE if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        let mut bits_needed_in_msb = amount_of_bits % 8;
        if bits_needed_in_msb == 0 {
            bits_needed_in_msb = 8;
        }
        #[expect(clippy::cast_possible_truncation)]
        let mut right_shift: i8 =
            (bits_needed_in_msb as i8) - ((qi.available_bits_in_first_byte % 8) as i8);
        if right_shift == 8 {
            right_shift = 0;
        }
        // because we are applying bits in place we need masks in insure we don't effect other fields
        // data. we need one for the first byte and the last byte.
        let first_bit_mask = get_right_and_mask(qi.available_bits_in_first_byte);
        let last_bit_mask = if bits_in_last_byte == 0 {
            get_left_and_mask(8)
        } else {
            get_left_and_mask(bits_in_last_byte)
        };
        Self {
            right_shift,
            first_bit_mask,
            last_bit_mask,
        }
    }
}
pub struct ResolverDataNestedAdditive {
    pub right_shift: i8,
}
impl From<&ResolverData> for ResolverDataNestedAdditive {
    fn from(quote_info: &ResolverData) -> Self {
        #[expect(clippy::cast_possible_truncation)]
        let right_shift: i8 = 8_i8 - ((quote_info.available_bits_in_first_byte % 8) as i8);
        Self { right_shift }
    }
}

pub struct ResolverDataBigAdditive {
    pub right_shift: i8,
    pub first_bit_mask: u8,
    pub last_bit_mask: u8,
    pub bits_in_last_byte: usize,
}
impl From<&ResolverData> for ResolverDataBigAdditive {
    fn from(qi: &ResolverData) -> Self {
        let amount_of_bits = qi.bit_length();
        let bits_in_last_byte = (amount_of_bits - qi.available_bits_in_first_byte) % 8;
        // how many times to shift the number right.
        // NOTE if negative shift left.
        // NOT if negative AND amount_of_bits == size of the fields data size (8bit for a u8, 32 bits
        // for a f32) then use the last byte in the fields byte array after shifting for the first
        // used byte in the buffer.
        #[allow(clippy::cast_possible_truncation)]
        let mut right_shift: i8 =
            ((amount_of_bits % 8) as i8) - ((qi.available_bits_in_first_byte % 8) as i8);
        if right_shift < 0 {
            right_shift += 8;
        }
        // because we are applying bits in place we need masks in insure we don't effect other fields
        // data. we need one for the first byte and the last byte.
        let first_bit_mask = get_right_and_mask(qi.available_bits_in_first_byte);
        let last_bit_mask = if bits_in_last_byte == 0 {
            get_left_and_mask(8)
        } else {
            get_left_and_mask(bits_in_last_byte)
        };
        Self {
            right_shift,
            first_bit_mask,
            last_bit_mask,
            bits_in_last_byte,
        }
    }
}

pub enum SolvedFieldSetAdditive<'a> {
    Struct {
        name: &'a Ident,
    },
    Variant {
        enum_name: &'a Ident,
        variant_name: Ident,
    },
}

impl<'a> SolvedFieldSetAdditive<'a> {
    pub fn new_struct(value: &'a Ident) -> Self {
        Self::Struct { name: value }
    }
    pub fn new_variant(enum_name: &'a Ident, variant_name: Ident) -> Self {
        Self::Variant {
            enum_name,
            variant_name,
        }
    }
    pub fn is_variant(&self) -> bool {
        matches!(
            self,
            Self::Variant {
                enum_name: _,
                variant_name: _
            }
        )
    }
    pub fn get_prefixed_name(&self, field_name: &Ident) -> Ident {
        match self {
            Self::Struct { .. } => format_ident!("{field_name}"),
            Self::Variant {
                enum_name: _,
                variant_name,
            } => format_ident!("{variant_name}_{field_name}"),
        }
    }
}

impl SolvedFieldSet {
    pub const VARIANT_ID_NAME: &'static str = "variant_id";
    pub const VARIANT_ID_NAME_KEBAB: &'static str = "variant-id";
    // TODO make sure capture id fields in enums do not get read twice.
    pub fn vis(&self) -> &Visibility {
        &self.attrs.vis
    }
    pub fn generate_quotes(
        &self,
        name: &Ident,
        enum_name: Option<&Ident>,
        struct_size: usize,
        dyn_fns: bool,
    ) -> syn::Result<FieldQuotes> {
        // generate basic generated code for field access functions.
        let mut quotes = self.gen_struct_fields(name, enum_name, struct_size, dyn_fns)?;
        // Gather information to finish [`Bitfields::from_bytes`]
        let from_bytes_quote = &quotes.read_fns.bitfield_trait;
        let fields_list = &quotes.field_list;
        // construct from bytes function. use input_byte_buffer as input name because,
        // that is what the field quotes expect to extract from.
        // wrap our list of field names with commas with Self{} so we it instantiate our struct,
        // because all of the from_bytes field quote store there data in a temporary variable with the same
        // name as its destination field the list of field names will be just fine.
        quotes.read_fns.bitfield_trait = quote! {
            fn from_bytes(mut input_byte_buffer: [u8;#struct_size]) -> Self {
                #from_bytes_quote
                Self{
                    #fields_list
                }
            }
        };
        if let Some(dyn_fns) = quotes.read_fns.dyn_fns.as_mut() {
            // do what we did for `Bitfields` impl for `BitfieldsDyn` impl
            let from_bytes_dyn_quote_inner = dyn_fns.bitfield_dyn_trait.clone();
            let comment_take =
                "Creates a new instance of `Self` by copying field from the bitfields, \
            removing bytes that where used. \n # Errors\n If the provided `Vec<u8>` does not have \
            enough bytes an error will be returned."
                    .to_string();
            let comment = "Creates a new instance of `Self` by copying field from the bitfields. 
             # Errors\n If the provided `Vec<u8>` does not have enough bytes an error will be returned.".to_string();
            dyn_fns.bitfield_dyn_trait = quote! {
                #[doc = #comment]
                fn from_slice(input_byte_buffer: &[u8]) -> Result<Self, bondrewd::BitfieldLengthError> {
                    if input_byte_buffer.len() < Self::BYTE_SIZE {
                        return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                    }
                    let out = {
                        #from_bytes_dyn_quote_inner
                        Self {
                            #fields_list
                        }
                    };
                    Ok(out)
                }
            };
            #[cfg(feature = "std")]
            {
                let from_bytes_dyn_quote = &dyn_fns.bitfield_dyn_trait;
                dyn_fns.bitfield_dyn_trait = quote! {
                    #from_bytes_dyn_quote
                    #[doc = #comment_take]
                    fn from_vec(input_byte_buffer: &mut Vec<u8>) -> Result<Self, bondrewd::BitfieldLengthError> {
                        if input_byte_buffer.len() < Self::BYTE_SIZE {
                            return Err(bondrewd::BitfieldLengthError(input_byte_buffer.len(), Self::BYTE_SIZE));
                        }
                        let out = {
                            #from_bytes_dyn_quote_inner
                            Self {
                                #fields_list
                            }
                        };
                        let _ = input_byte_buffer.drain(..Self::BYTE_SIZE);
                        Ok(out)
                    }
                };
            }
        }
        let into_bytes_quote = &quotes.write_fns.bitfield_trait;
        quotes.write_fns.bitfield_trait = quote! {
            fn into_bytes(self) -> [u8;#struct_size] {
                let mut output_byte_buffer: [u8;#struct_size] = [0u8;#struct_size];
                #into_bytes_quote
                output_byte_buffer
            }
        };
        Ok(quotes)
    }
    pub(crate) fn gen_struct_fields(
        &self,
        name: &Ident,
        enum_name: Option<&Ident>,
        struct_size: usize,
        dyn_fns: bool,
    ) -> syn::Result<FieldQuotes> {
        let set_add = if let Some(ename) = enum_name {
            // We what to use the name of the struct because enum variants are just StructInfos internally.
            let vn = format_ident!("{}", name.to_string().to_case(Case::Snake));
            SolvedFieldSetAdditive::new_variant(&ename, vn)
        } else {
            SolvedFieldSetAdditive::new_struct(name)
        };
        let mut gen_read = GeneratedFunctions::new(dyn_fns);
        let mut gen_write = GeneratedFunctions::new(dyn_fns);
        // TODO If we are building code for an enum variant that does not capture the id
        // then we should skip the id field to avoid creating an get_id function for each variant.
        let mut field_name_list = quote! {};
        for field in &self.fields {
            // TODO verify we want to hide all fake_fields.
            if matches!(field.attr_reserve(), ReserveFieldOption::FakeField) {
                continue;
            }
            // TODO capture_id may not need to be run fully, capture id fields will
            // rely on the fact it was already read for the matching process.
            let field_access = field.get_quotes()?;
            self.make_read_fns(
                field,
                &set_add,
                &mut field_name_list,
                &mut gen_read,
                &field_access,
                struct_size,
            )?;
            self.make_write_fns(field, &set_add, &mut gen_write, &field_access, struct_size)?;
        }
        // Do checked struct of this type
        let checked = if self.fields.is_empty() {
            None
        } else if let (Some(dyn_fns_read), Some(dyn_fns_write)) =
            (&mut gen_read.dyn_fns, &mut gen_write.dyn_fns)
        {
            let struct_name = if let Some(e_name) = enum_name {
                quote::format_ident!("{e_name}{name}")
            } else {
                name.clone()
            };
            let vis = self.vis();
            let checked_ident = quote::format_ident!("{struct_name}Checked");
            let checked_mut_ident = quote::format_ident!("{struct_name}CheckedMut");
            let unchecked_functions = &dyn_fns_read.checked_struct;
            let unchecked_mut_functions = &dyn_fns_write.checked_struct;
            let comment = format!("A Structure which provides functions for getting the fields of a [{struct_name}] in its bitfield form.");
            let comment_mut = format!("A Structure which provides functions for getting and setting the fields of a [{struct_name}] in its bitfield form.");
            let unchecked_comment = format!("Panics if resulting `{checked_ident}` does not contain enough bytes to read a field that is attempted to be read.");
            let unchecked_comment_mut = format!("Panics if resulting `{checked_mut_ident}` does not contain enough bytes to read a field that is attempted to be read or written.");
            dyn_fns_write.checked_struct = quote! {
                #[doc = #comment_mut]
                #vis struct #checked_mut_ident<'a> {
                    buffer: &'a mut [u8],
                }
                impl<'a> #checked_mut_ident<'a> {
                    #unchecked_functions
                    #unchecked_mut_functions
                    #[doc = #unchecked_comment_mut]
                    pub fn from_unchecked_slice(data: &'a mut [u8]) -> Self {
                        Self{
                            buffer: data
                        }
                    }
                }
            };
            dyn_fns_read.checked_struct = quote! {
                #[doc = #comment]
                #vis struct #checked_ident<'a> {
                    buffer: &'a [u8],
                }
                impl<'a> #checked_ident<'a> {
                    #unchecked_functions
                    #[doc = #unchecked_comment]
                    pub fn from_unchecked_slice(data: &'a [u8]) -> Self {
                        Self{
                            buffer: data
                        }
                    }
                }
            };
            let ename = if enum_name.is_some() {
                Some(&struct_name)
            } else {
                None
            };

            let check_slice_info = CheckedSliceGen::new(name, self.total_bytes(), ename);

            let check_slice_fn = check_slice_info.fn_gen;
            let impl_fns = gen_read.non_trait;
            gen_read.non_trait = quote! {
                #impl_fns
                #check_slice_fn
            };

            let check_slice_mut_fn = check_slice_info.mut_fn_gen;
            let impl_fns = gen_write.non_trait;
            gen_write.non_trait = quote! {
                #impl_fns
                #check_slice_mut_fn
            };
            Some(CheckSliceNames {
                func: check_slice_info.fn_name,
                mut_func: check_slice_info.mut_fn_name,
                structure: checked_ident,
                mut_structure: checked_mut_ident,
            })
        } else {
            None
        };
        Ok(FieldQuotes {
            read_fns: gen_read,
            write_fns: gen_write,
            field_list: field_name_list,
            slice_info: checked,
        })
    }
    pub(crate) fn make_read_fns(
        &self,
        field: &SolvedData,
        set_add: &SolvedFieldSetAdditive,
        field_name_list: &mut TokenStream,
        gen: &mut GeneratedFunctions,
        field_access: &GeneratedQuotes,
        struct_size: usize,
    ) -> syn::Result<()> {
        let field_name = field.resolver.ident();
        let prefixed_name = set_add.get_prefixed_name(&field_name);

        let mut impl_fns = quote! {};
        let mut checked_struct_impl_fns = if gen.dyn_fns.is_some() {
            Some(quote! {})
        } else {
            None
        };
        let field_extractor = field_access.read();
        self.make_read_fns_inner(
            field,
            &prefixed_name,
            field_extractor,
            &mut impl_fns,
            checked_struct_impl_fns.as_mut(),
            struct_size,
        )?;
        gen.append_impl_fns(&impl_fns);
        if let Some(checked_struct_impl_fns) = &checked_struct_impl_fns {
            gen.append_checked_struct_impl_fns(checked_struct_impl_fns);
        }

        // fake fields do not exist in the actual structure and should only have functions
        // that read or write values into byte arrays.
        if !field.attr_reserve().is_fake_field() {
            // put the name of the field into the list of fields that are needed to create
            // the struct.
            *field_name_list = quote! {#field_name, #field_name_list};
            // TODO line above replaced commented code below, this is old code that i don't think is necessary.
            // field order here shouldn't matter.
            //
            // if field.is_field_order_reversed() {
            //     *field_name_list = quote! {#field_name, #field_name_list}
            // } else {
            //     *field_name_list = quote! {#field_name_list #field_name,}
            // };
            let peek_call = if field.attr_capture_id() {
                // put the field extraction in the actual from bytes.
                if field.attr_reserve().wants_read_fns() {
                    let id_name = format_ident!("{}", Self::VARIANT_ID_NAME);
                    quote! {
                        let #field_name = #id_name;
                    }
                } else {
                    return Err(syn::Error::new(
                        field.resolver.ident().span(),
                        "fields with attribute 'capture_id' are automatically considered 'read_only', meaning it can not have the 'reserve' attribute.",
                    ));
                }
            } else {
                // put the field extraction in the actual from bytes.
                let read_stuff = field_access.read();
                if field.attr_reserve().wants_read_fns() {
                    // let fn_field_name = format_ident!("read_{prefixed_name}");
                    quote! {
                        let #field_name = #read_stuff;
                    }
                } else {
                    quote! { let #field_name = Default::default(); }
                }
            };
            gen.append_bitfield_trait_impl_fns(&peek_call);
            gen.append_bitfield_dyn_trait_impl_fns(&quote! {
                let #field_name = #field_extractor;
            });
        }
        Ok(())
    }
    fn make_read_fns_inner(
        &self,
        field: &SolvedData,
        prefixed_field_name: &Ident,
        field_extractor: &TokenStream,
        peek_quote: &mut TokenStream,
        peek_slice_fns_option: Option<&mut TokenStream>,
        struct_size: usize,
    ) -> syn::Result<()> {
        *peek_quote =
            generate_read_field_fn(field_extractor, field, struct_size, prefixed_field_name)?;
        // make the slice functions if applicable.
        if let Some(peek_slice) = peek_slice_fns_option {
            let peek_slice_quote =
                generate_read_slice_field_fn(field_extractor, field, prefixed_field_name)?;
            *peek_quote = quote! {
                #peek_quote
                #peek_slice_quote
            };
            let peek_slice_unchecked_quote =
                generate_read_slice_field_fn_unchecked(field_extractor, field)?;
            *peek_slice = quote! {
                #peek_slice
                #peek_slice_unchecked_quote
            };
        }
        Ok(())
    }
    pub(crate) fn make_write_fns(
        &self,
        field: &SolvedData,
        set_add: &SolvedFieldSetAdditive,
        gen: &mut GeneratedFunctions,
        field_access: &GeneratedQuotes,
        struct_size: usize,
    ) -> syn::Result<()> {
        let field_name = field.resolver.ident();
        let prefixed_name = set_add.get_prefixed_name(&field_name);
        let (field_setter, clear_quote) = (field_access.write(), field_access.zero());
        if field.attr_reserve().wants_write_fns() && !field.attr_capture_id() {
            if set_add.is_variant() {
                let fn_name = format_ident!("write_{prefixed_name}");
                gen.append_bitfield_trait_impl_fns(&quote! {
                    Self::#fn_name(&mut output_byte_buffer, #field_name);
                });
            } else {
                gen.append_bitfield_trait_impl_fns(&quote! {
                    let #field_name = self.#field_name;
                    #field_setter
                });
            }
        }

        let mut impl_fns = quote! {};
        let mut checked_struct_impl_fns = if gen.dyn_fns.is_some() {
            Some(quote! {})
        } else {
            None
        };
        self.make_write_fns_inner(
            field,
            &prefixed_name,
            field_setter,
            clear_quote,
            &mut impl_fns,
            checked_struct_impl_fns.as_mut(),
            struct_size,
        )?;

        gen.append_impl_fns(&impl_fns);
        if let Some(checked_struct_impl_fns) = checked_struct_impl_fns {
            gen.append_checked_struct_impl_fns(&checked_struct_impl_fns);
        }
        Ok(())
    }
    fn make_write_fns_inner(
        &self,
        field: &SolvedData,
        prefixed_field_name: &Ident,
        field_setter: &TokenStream,
        clear_quote: &TokenStream,
        write_quote: &mut TokenStream,
        write_slice_fns_option: Option<&mut TokenStream>,
        struct_size: usize,
    ) -> syn::Result<()> {
        *write_quote = generate_write_field_fn(
            field_setter,
            clear_quote,
            field,
            struct_size,
            prefixed_field_name,
        )?;
        if let Some(write_slice_fns_option) = write_slice_fns_option {
            let set_slice_quote = generate_write_slice_field_fn(
                field_setter,
                clear_quote,
                field,
                prefixed_field_name,
            )?;
            *write_quote = quote! {
                #write_quote
                #set_slice_quote
            };
            let set_slice_unchecked_quote =
                generate_write_slice_field_fn_unchecked(field_setter, clear_quote, field)?;
            *write_slice_fns_option = quote! {
                #write_slice_fns_option
                #set_slice_unchecked_quote
            };
        }
        Ok(())
    }
    pub fn total_bits(&self) -> usize {
        let mut total: usize = 0;
        for field in &self.fields {
            total += field.bit_length();
        }
        total
    }
    pub fn total_bits_no_fill(&self) -> usize {
        let mut total: usize = 0;
        for field in &self.fields {
            total += field.resolver.bit_size_no_fill();
        }

        total
    }
    pub fn total_bytes(&self) -> usize {
        self.total_bits().div_ceil(8)
    }
    pub fn total_bytes_no_fill(&self) -> usize {
        self.total_bits_no_fill().div_ceil(8)
    }
    pub(crate) fn get_captured_id_name(&self) -> Option<Ident> {
        for field in &self.fields {
            if field.attr_capture_id() {
                return Some(field.resolver.name());
            }
        }
        None
    }
}
/// Generates a `read_field_name()` function.
pub(crate) fn generate_read_field_fn(
    field_quote: &TokenStream,
    field: &SolvedData,
    struct_size: usize,
    prefixed_field_name: &Ident,
) -> syn::Result<TokenStream> {
    let field_name = field.resolver.name();
    let type_ident = field.resolver.ty.get_type_quote()?;
    let bit_range = &field.bit_range();
    let fn_field_name = format_ident!("read_{prefixed_field_name}");
    let comment_bits = if bit_range.end - bit_range.start > 1 {
        format!("bits {} through {}", bit_range.start, bit_range.end - 1)
    } else {
        format!("bit {}", bit_range.start)
    };
    let comment = format!("Reads {comment_bits} within `input_byte_buffer`, getting the `{field_name}` field in bitfield form.");
    Ok(quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(input_byte_buffer: &[u8;#struct_size]) -> #type_ident {
            #field_quote
        }
    })
}
/// Generates a `read_slice_field_name()` function for a slice.
pub(crate) fn generate_read_slice_field_fn(
    field_quote: &TokenStream,
    field: &SolvedData,
    prefixed_field_name: &Ident,
) -> syn::Result<TokenStream> {
    let field_name = field.resolver.name();
    let type_ident = field.resolver.ty.get_type_quote()?;
    let bit_range = &field.bit_range();
    let fn_field_name = format_ident!("read_slice_{prefixed_field_name}");
    let min_length = bit_range.end.div_ceil(8);
    let comment = format!("Returns the value for the `{field_name}` field of a in bitfield form by reading  bits {} through {} in `input_byte_buffer`. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned if not enough bytes are present.", bit_range.start, bit_range.end - 1);
    Ok(quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(input_byte_buffer: &[u8]) -> Result<#type_ident, bondrewd::BitfieldLengthError> {
            let slice_length = input_byte_buffer.len();
            if slice_length < #min_length {
                Err(bondrewd::BitfieldLengthError(slice_length, #min_length))
            } else {
                Ok(
                    #field_quote
                )
            }
        }
    })
}
/// For use on generated Checked Slice Structures.
///
/// Generates a `read_field_name()` function for a slice.
///
/// # Warning
/// generated code does NOT check if the slice is large enough to be read from, Checked Slice Structures
/// are nothing but a slice ref that has been checked to contain enough bytes for any
/// `read_slice_field_name` functions.
pub(crate) fn generate_read_slice_field_fn_unchecked(
    field_quote: &TokenStream,
    field: &SolvedData,
) -> syn::Result<TokenStream> {
    let field_name = field.resolver.name();
    let type_ident = field.resolver.ty.get_type_quote()?;
    let bit_range = &field.bit_range();
    let fn_field_name = format_ident!("read_{field_name}");
    let comment_bits = if bit_range.end - bit_range.start > 1 {
        format!("bits {} through {}", bit_range.start, bit_range.end - 1)
    } else {
        format!("bit {}", bit_range.start)
    };
    let comment = format!(
        "Reads {comment_bits} in pre-checked slice, getting the `{field_name}` field in bitfield form."
    );
    Ok(quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(&self) -> #type_ident {
            let input_byte_buffer: &[u8] = self.buffer;
            #field_quote
        }
    })
}

/// Generates a `write_field_name()` function.
pub(crate) fn generate_write_field_fn(
    field_quote: &TokenStream,
    clear_quote: &TokenStream,
    field: &SolvedData,
    struct_size: usize,
    prefixed_field_name: &Ident,
) -> syn::Result<TokenStream> {
    let field_name = field.resolver.name();
    let type_ident = field.resolver.ty.get_type_quote()?;
    let bit_range = &field.bit_range();
    let fn_field_name = format_ident!("write_{prefixed_field_name}");
    let comment_bits = if bit_range.end - bit_range.start > 1 {
        format!("bits {} through {}", bit_range.start, bit_range.end - 1)
    } else {
        format!("bit {}", bit_range.start)
    };
    let comment = format!("Writes to {comment_bits} within `output_byte_buffer`, setting the `{field_name}` field in bitfield form.");
    Ok(quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(output_byte_buffer: &mut [u8;#struct_size], #field_name: #type_ident) {
            #clear_quote
            #field_quote
        }
    })
}
/// Generates a `write_slice_field_name()` function for a slice.
pub(crate) fn generate_write_slice_field_fn(
    field_quote: &TokenStream,
    clear_quote: &TokenStream,
    field: &SolvedData,
    prefixed_field_name: &Ident,
) -> syn::Result<TokenStream> {
    let field_name = field.resolver.name();
    let type_ident = field.resolver.ty.get_type_quote()?;
    let bit_range = &field.bit_range();
    let fn_field_name = format_ident!("write_slice_{prefixed_field_name}");
    let min_length = bit_range.end.div_ceil(8);
    let comment_bits = if bit_range.end - bit_range.start > 1 {
        format!("bits {} through {}", bit_range.start, bit_range.end - 1)
    } else {
        format!("bit {}", bit_range.start)
    };
    let comment = format!("Writes to {comment_bits} in `input_byte_buffer` if enough bytes are present in slice, setting the `{field_name}` field in bitfield form. Otherwise a [BitfieldLengthError](bondrewd::BitfieldLengthError) will be returned");
    Ok(quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(output_byte_buffer: &mut [u8], #field_name: #type_ident) -> Result<(), bondrewd::BitfieldLengthError> {
            let slice_length = output_byte_buffer.len();
            if slice_length < #min_length {
                Err(bondrewd::BitfieldLengthError(slice_length, #min_length))
            } else {
                #clear_quote
                #field_quote
                Ok(())
            }
        }
    })
}
/// For use on generated Checked Slice Structures.
///
/// Generates a `write_field_name()` function for a slice.
///
/// # Warning
/// generated code does NOT check if the slice can be written to, Checked Slice Structures are nothing
/// but a slice ref that has been checked to contain enough bytes for any `write_slice_field_name`
/// functions.
pub(crate) fn generate_write_slice_field_fn_unchecked(
    field_quote: &TokenStream,
    clear_quote: &TokenStream,
    field: &SolvedData,
) -> syn::Result<TokenStream> {
    let field_name = field.resolver.name();
    let type_ident = field.resolver.ty.get_type_quote()?;
    let bit_range = &field.bit_range();
    let fn_field_name = format_ident!("write_{field_name}");
    let comment_bits = if bit_range.end - bit_range.start > 1 {
        format!("bits {} through {}", bit_range.start, bit_range.end - 1)
    } else {
        format!("bit {}", bit_range.start)
    };
    let comment = format!(
        "Writes to {comment_bits} in pre-checked mutable slice, setting the `{field_name}` field in bitfield form.",
    );
    Ok(quote! {
        #[inline]
        #[doc = #comment]
        pub fn #fn_field_name(&mut self, #field_name: #type_ident) {
            let output_byte_buffer: &mut [u8] = self.buffer;
            #clear_quote
            #field_quote
        }
    })
}
