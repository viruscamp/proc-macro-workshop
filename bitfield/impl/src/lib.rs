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

    let mut vec_bits = vec![quote!(0)];
    for Field { ty, ident, ..} in &item_struct.fields {
        if let Some(name_field) = ident
        {
            let generics_const = quote!(::<{ Self::BYTES }, { ( #(#vec_bits)+* ) as usize }, { <#ty as ::bitfield::Specifier>::BITS as usize }>);
            let bits_type = quote!(<#ty as ::bitfield::Specifier>::Inner);                        
            let fn_set = format_ident!("set_{name_field}");
            let fn_get = format_ident!("get_{name_field}");
            field_methods.push(quote! {
                pub fn #fn_get(&self) -> #bits_type {
                    ::bitfield::get_generic #generics_const(&self.data) as #bits_type
                }
                pub fn #fn_set(&mut self, v: #bits_type) {
                    ::bitfield::set_generic #generics_const(&mut self.data, v as u64)
                }
            });
            vec_bits.push(quote!(<#ty as ::bitfield::Specifier>::BITS));
        } else {
            errors.push(Error::new(ty.span(), "unknown type"));
        }
    }

    let (impl_generics, types, where_clause) = item_struct.generics.split_for_impl();
    let errors = errors.iter().map(Error::to_compile_error);
    
    let a = quote! {
        #(#errors)*

        #[repr(C)]
        #vis_struct struct #name_struct #types
            #where_clause
        {
            data: [u8; Self::BYTES],
        }

        //const _: usize = <#name_struct as ::bitfield::checks::TotalSizeIsMultipleOfEightBits>::SIZE;

        impl #impl_generics #name_struct #types
            #where_clause
        {
            const BITS: usize = ( #(#vec_bits)+* ) as usize;
            const BYTES: usize = ::bitfield::bits_size_to_byte_size(Self::BITS);
            pub fn new() -> Self {
                Self { data: Default::default() }
            }

            #(#field_methods)*
        }
    };
eprintln!("{}", a.to_string());
    a.into()
}
