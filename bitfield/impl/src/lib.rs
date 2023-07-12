#![feature(let_chains)]

use proc_macro2::*;
use syn::{*, spanned::Spanned};
use quote::*;

#[proc_macro_attribute]
pub fn bitfield(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    let item_struct = parse_macro_input!(input as ItemStruct);
    let vis_struct = &item_struct.vis;
    let name_struct = &item_struct.ident;

    let mut errors: Vec<Error> = vec![];
    let mut field_methods = vec![];

    let mut from = 0u32;
    for Field { ty, ident, ..} in &item_struct.fields {
        if let Some(name_field) = ident
            && let Type::Path(TypePath { qself: None, path }) = ty
            && let Some(ty) = path.get_ident()
            && let Some(bits) = type_bits(ty)
        {
            let bits_type = match bits {
                1..=8 => "u8",
                9..=16 => "u16",
                17..=32 => "u32",
                33..=64 => "u64",
                _ => "u128",
            };
            let bits_type = format_ident!("{bits_type}");                        
            let fn_set = format_ident!("set_{name_field}");
            let fn_get = format_ident!("get_{name_field}");
            field_methods.push(quote! {
                pub fn #fn_get(&self) -> #bits_type {
                    ::bitfield::get(&self.data, #from as usize, #bits as usize) as #bits_type
                }
                pub fn #fn_set(&mut self, val: #bits_type) {
                    ::bitfield::set(&mut self.data, val as u64, #from as usize, #bits as usize)
                }
            });

            from += bits;
        } else {
            errors.push(Error::new(ty.span(), "unknown type"));
        }
    }

    let size = (from + u8::BITS - 1) / u8::BITS;
    let (impl_generics, types, where_clause) = item_struct.generics.split_for_impl();
    let errors = errors.iter().map(Error::to_compile_error);

    quote! {
        #(#errors)*

        #[repr(C)]
        #vis_struct struct #name_struct #types
            #where_clause
        {
            data: [u8; #size as usize],
        }

        impl #impl_generics #name_struct #types
            #where_clause
        {
            pub fn new() -> Self {
                Self { data: <[u8; #size as usize] as Default>::default() }
            }

            #(#field_methods)*
        }
    }.into()
}

fn type_bits(id: &Ident) -> Option<u32> {
    let id = id.to_string();
    let (b, bits) = id.split_at(1);
    if b == "B" && let Ok(bits) = bits.parse() {
        Some(bits)
    } else {
        None
    }
}