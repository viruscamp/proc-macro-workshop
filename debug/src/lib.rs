#![feature(let_chains)]

use proc_macro2::*;
use syn::{*, parse::Parse};
use quote::*;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let mut errors = vec![];
    let mut fields_debug = vec![];

    if let Data::Struct(DataStruct {
        fields: Fields::Named(ref fields),
        ..
    }) = input.data
    {
        for f in &fields.named {

            let field_name = &f.ident;
            // find `#[debug = "0b{:08b}"]`
            if let Some(attr_value) = f.attrs.iter().find_map(|attr| {
                if let Meta::NameValue(MetaNameValue {
                        path,
                        value,
                        ..
                    }) = &attr.meta
                    && let Some(attr_name) = path.get_ident()
                    && attr_name.to_string() == "debug"
                {
                    return Some(value);
                }
                return None;
            }) {
                if let Expr::Lit(ExprLit { lit: Lit::Str(fmt_str), .. })= attr_value {
                    fields_debug.push(quote! {
                        .field(stringify!(#field_name), &format_args!(#fmt_str, &self.#field_name))
                    });
                } else {
                    errors.push(Error::new_spanned(&attr_value, "must be valid format string"));
                }
            } else {
                fields_debug.push(quote! {
                    .field(stringify!(#field_name), &self.#field_name)
                });
            }
        }
    } else {
        errors.push(Error::new_spanned(&input, "should be struct"));
    }

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let where_clause_debug = if let Some(where_clause) = where_clause {
        let mut where_clause = where_clause.clone();
        let debug_trait_bound = syn::parse_str::<TraitBound>("::core::fmt::Debug").unwrap();
        where_clause.predicates.iter_mut().for_each(|predicate| {
            if let WherePredicate::Type(x) = predicate {
                x.bounds.extend(Some(TypeParamBound::Trait(debug_trait_bound.clone())))
            }
        });
        where_clause.to_token_stream()
    } else {
        let type_parmas = generics.params.iter()
        .filter_map(|gp| {
            if let GenericParam::Type(tp) = gp {
                Some(tp)
            } else {
                None
            }
        });
        quote! {
            where #(#type_parmas: ::core::fmt::Debug),*
        }
    };

    let errors = errors.iter().map(Error::to_compile_error);
    let expand = quote! {
        #(#errors)*
        impl #impl_generics ::core::fmt::Debug for #struct_name #ty_generics
            #where_clause_debug
        {
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                fmt.debug_struct(stringify!(#struct_name))
                    #(#fields_debug)*
                    .finish()
            }
        }
    };

    proc_macro::TokenStream::from(expand)
}
