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
    //let mut fields = vec![];

    let mut bits_struct = 0u32;
    for f in item_struct.fields {
        if let Type::Path(TypePath { qself: None, path }) = &f.ty
            && let Some(ty) = path.get_ident()
            && let Some(bits) = type_bits(ty)
        {
            bits_struct += bits;
        } else {
            errors.push(Error::new(f.ty.span(), "unknown type"));
        }
    }

    let size = (bits_struct + u8::BITS - 1) / u8::BITS;
    let (_, types, where_clause) = item_struct.generics.split_for_impl();
    let errors = errors.iter().map(Error::to_compile_error);

    quote! {
        #(#errors)*

        #[repr(C)]
        #vis_struct struct #name_struct #types
            #where_clause
        {
            data: [u8; #size as usize],
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