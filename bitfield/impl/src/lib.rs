#![feature(let_chains)]

use proc_macro2::Span;
//use proc_macro2::*;
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
            let bits_type = quote!(<#ty as ::bitfield::Specifier>::Value);
            let fn_set = format_ident!("set_{name_field}");
            let fn_get = format_ident!("get_{name_field}");
            field_methods.push(quote! {
                pub fn #fn_get(&self) -> #bits_type {
                    let u = ::bitfield::get_generic #generics_const(&self.data);
                    <#ty as ::bitfield::Specifier>::get(u)
                }
                pub fn #fn_set(&mut self, v: #bits_type) {
                    let u = <#ty as ::bitfield::Specifier>::set(v);
                    ::bitfield::set_generic #generics_const(&mut self.data, u)
                }
            });
            vec_bits.push(quote!(<#ty as ::bitfield::Specifier>::BITS));
        } else {
            errors.push(Error::new(ty.span(), "unknown type"));
        }
    }

    let (impl_generics, types, where_clause) = item_struct.generics.split_for_impl();
    let errors = errors.iter().map(Error::to_compile_error);
 
    quote! {
        #(#errors)*

        #[repr(C)]
        #vis_struct struct #name_struct #types
            #where_clause
        {
            data: [u8; Self::BYTES],
        }

        const _: () = {
            // 重定义名称 实际上无用，强制报错时使用类型全名
            trait TotalSizeIsMultipleOfEightBits {}
            struct SevenMod8;
            struct ZeroMod8;
            const _: usize =
                <
                    <
                        [u8; #name_struct::BITS % 8]
                            as ::bitfield::checks::CheckSizeMod8
                    >::Target
                        as ::bitfield::checks::TotalSizeIsMultipleOfEightBits
                >::SIZE;
            ()
        };

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
    }.into()
}

#[proc_macro_derive(BitfieldSpecifier)]
pub fn derive_bitfield_specifier(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;
    let enum_def = &input.data;

    let mut variants_ident = vec![];
    let mut errors = vec![];

    let bits = if let Data::Enum(enum_def) = enum_def {
        for v in &enum_def.variants {
            if v.fields.len() == 0 {
                variants_ident.push(&v.ident);
            } else {
                errors.push(Error::new(v.ident.span(), "type fails"));
            }
        }

        let variants_len = variants_ident.len();
        let bits = bits_u64(variants_len as u64);
        if (1 << bits) != variants_len {
            errors.push(Error::new(Span::call_site(), "BitfieldSpecifier expected a number of variants which is a power of 2"));
        }
        bits
    } else {
        errors.push(Error::new(enum_name.span(), "must be an enum"));
        1
    };
    let variants_len = variants_ident.len();
    let errors = errors.iter().map(Error::to_compile_error);
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    
    let variant_checks = variants_ident.iter().map(|vi| {
        let vi_span = syn::spanned::Spanned::span(&vi);
        quote_spanned! { vi_span =>
            const _: bool =
                <
                    <
                        ::bitfield::checks::StaticBoolean<
                            {(#enum_name::#vi as usize) < #variants_len}
                        > as ::bitfield::checks::BooleanTarget
                    >::Target
                        as ::bitfield::checks::DiscriminantInRange
                >::VALUE;
        }
    });
    
    let output = quote! {
        #(#errors)*
        impl #impl_generics ::bitfield::Specifier for #enum_name #type_generics #where_clause {
            const BITS: u32 = #bits;
            type Value = Self;
            fn get(u: u64) -> Self {
                Self::try_from_u64(u).unwrap()
            }
            fn set(v: Self) -> u64 {
                Self::into_u64(v)
            }
        }

        impl #impl_generics #enum_name #type_generics #where_clause {
            fn into_u64(e: #enum_name #type_generics) -> u64 {
                e as u64
            }

            fn try_from_u64(u: u64) -> std::result::Result<Self, ()> {
                #(
                    if u == (#enum_name::#variants_ident as u64) {
                        ::core::result::Result::Ok(#enum_name::#variants_ident)
                    }
                )else*
                else {
                    ::core::result::Result::Err(())
                }
            }
        }

        const _: () = {
            struct False;
            struct True;
            trait DiscriminantInRange {}
            #(#variant_checks)*
            ()
        };
    };

    //eprintln!("{}", output.to_string());
    output.into()
}

fn bits_u64(v: u64) -> u32 {
    if v == 0 {
        1
    } else {
        let mut bits = 1;
        loop {
            if ((v - 1) >> bits) == 0 {
                break;
            }
            bits += 1;
        }
        bits
    }
}

mod test {
    #[test]
    fn bits_u64() {
        use super::bits_u64;
        assert_eq!(1, bits_u64(0));
        assert_eq!(1, bits_u64(1));
        assert_eq!(2, bits_u64(4));
        assert_eq!(3, bits_u64(7));
        assert_eq!(3, bits_u64(8));
        assert_eq!(4, bits_u64(9));
    }
}