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
    let mut field_checks = vec![];

    let mut vec_bits = vec![quote!(0)];
    for Field { ty, ident, attrs,..} in &item_struct.fields {
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

            // find #[bits = 9]
            let id_bits = format_ident!("bits");
            for attr in attrs {
                if let Meta::NameValue(
                        MetaNameValue { path, value, .. }
                    ) = &attr.meta
                    && path.is_ident(&id_bits)
                {
                    if let Expr::Lit(ExprLit { lit: Lit::Int(bits), ..}) = value
                    {
                        field_checks.push(quote_spanned ! { bits.span() =>
                            const _: [u8; #bits as usize] = [0; #ty::BITS as usize];
                        });
                    } else {
                        errors.push(Error::new(value.span(), "invalid, must be an int which > 0"));
                    }
                }
            }
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
            // 方案1，来自 static_assertions
            // 报错信息不够明确
            //const _: [(); 0 - !{ const ASSERT: bool = $x; ASSERT } as usize] = [];
            //const _: [(); (#name_struct::BITS % 8 as usize)] = [];

            /*
            // 推荐方案
            // 1. const fn 替代关联常量, 少一个 as， 报错多一段指向 check 函数
            //   函数必须是 const
            //   不能放在 trait 里面，因为无法标记 const
            //   只能提供类型， 不要求提供值， 即不能作为参数，也不能用常量赋值
            // 2. 直接用 [u8; 0] 而不是再找到 ZeroMod8, 再少一个 as, 报错的类型不同
            const _: () = {
                const fn check<T: ::bitfield::checks::TotalSizeIsMultipleOfEightBits>() {}
                check::<[u8; #name_struct::BITS % 8]>();
            };
            */

            // workgrounds
            // 1. 强制报错时使用类型全名
            trait TotalSizeIsMultipleOfEightBits {}
            struct SevenMod8;
            struct ZeroMod8;
            const _: () =
                <
                    <
                        [u8; #name_struct::BITS % 8]
                            as ::bitfield::checks::CheckSizeMod8
                    >::Target // 2. 多一次类型转换 [u8; 0] --> ZeroMod8
                        as ::bitfield::checks::TotalSizeIsMultipleOfEightBits
                >::CHECK_CONST; // 3. 多一个关联常量， 多一个 as
        };
        #(#field_checks)*

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
            //const _: [(); 0 - !{(#enum_name::#vi as usize) < #variants_len} as usize] = [];
            /*
            const _: () = {
                const fn check<T: ::bitfield::checks::DiscriminantInRange>() {}
                check::<
                    ::bitfield::checks::StaticBool<
                        { (#enum_name::#vi as usize) < #variants_len }
                    >
                >();
            };
            */
            const _: () =
                <
                    <
                        ::bitfield::checks::StaticBool<
                            {(#enum_name::#vi as usize) < #variants_len}
                        > as ::bitfield::checks::BoolTarget
                    >::Target
                        as ::bitfield::checks::DiscriminantInRange
                >::CHECK_CONST;
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