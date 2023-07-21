#![feature(let_chains)]

use std::collections::HashSet;

use mylib_macro::*;
use proc_macro2::*;
use quote::*;
use syn::*;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let mut errors = vec![];
    let mut fields_debug = vec![];

    let mut attr_debug_bounds = vec![];
    fn parse_attr_debug_bounds(
        attrs: &Vec<Attribute>,
        debug_bounds: &mut Vec<LitStr>,
        errors: &mut Vec<Error>,
    ) -> usize {
        let mut found = 0;
        for debug_bound in extract_attr_debug_bounds(attrs) {
            match debug_bound {
                Ok(s) => {
                    found += 1;
                    debug_bounds.push(s)
                }
                Err(err) => errors.push(err),
            }
        }
        found
    }
    let disable_inference =
        parse_attr_debug_bounds(&input.attrs, &mut attr_debug_bounds, &mut errors) > 0;

    // 方法5 加入 T  X  T::Target T::Target<X>
    let mut path_with_params = HashSet::new();
    let gpids = input
        .generics
        .params
        .iter()
        .filter_map(|gp| {
            if let GenericParam::Type(TypeParam { ident, .. }) = gp {
                Some(ident)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { ref named, .. }),
        ..
    }) = input.data
    {
        for Field {
            ident: field_name,
            ty,
            attrs,
            ..
        } in named
        {
            match make_field_debug_from_attr_debug_fmt(attrs, field_name) {
                Ok(ts) => fields_debug.push(ts),
                Err(err) => errors.push(err),
            }
            let disable_inference_field =
                parse_attr_debug_bounds(attrs, &mut attr_debug_bounds, &mut errors) > 0;
            if !disable_inference && !disable_inference_field {
                used_generic_param(ty, gpids.as_slice(), &mut path_with_params);
            }
        }
    } else {
        errors.push(Error::new_spanned(&input.ident, "should be struct"));
    }

    let where_clause = input.generics.make_where_clause();
    where_clause
        .predicates
        .extend(path_with_params.iter().filter_map(|p| {
            match syn::parse2::<WherePredicate>(quote! {
                #p: ::core::fmt::Debug
            }) {
                Ok(wp) => Some(wp),
                Err(err) => {
                    errors.push(err);
                    None
                }
            }
        }));
    where_clause
        .predicates
        .extend(attr_debug_bounds.iter().filter_map(|s| {
            match syn::parse_str::<WherePredicate>(&s.value()) {
                Ok(wp) => Some(wp),
                Err(err) => {
                    errors.push(err);
                    None
                }
            }
        }));

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let errors = errors.iter().map(Error::to_compile_error);
    let expand = quote! {
        #(#errors)*
        impl #impl_generics ::core::fmt::Debug for #struct_name #ty_generics
            #where_clause
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

// find `#[debug = "0b{:08b}"]`
// 没有，找到但错误，找到合法的
fn make_field_debug_from_attr_debug_fmt(
    attrs: &Vec<Attribute>,
    field_name: &Option<Ident>,
) -> Result<TokenStream> {
    let attr_id_debug = format_ident!("debug");
    if let Some(attr_value) = attrs.iter().find_map(|attr| {
        if let Meta::NameValue(MetaNameValue {
                path,
                value,
                ..
            }) = &attr.meta
            && path.is_ident(&attr_id_debug)
        {
            return Some(value);
        }
        return None;
    }) {
        if let Expr::Lit(ExprLit {
            lit: Lit::Str(fmt_str),
            ..
        }) = attr_value
        {
            Ok(quote! {
                .field(stringify!(#field_name), &format_args!(#fmt_str, &self.#field_name))
            })
        } else {
            Err(Error::new_spanned(
                &attr_value,
                "must be valid format string",
            ))
        }
    } else {
        Ok(quote! {
            .field(stringify!(#field_name), &self.#field_name)
        })
    }
}

fn extract_attr_debug_bounds(attrs: &Vec<Attribute>) -> Vec<Result<LitStr>> {
    let attr_id_debug = format_ident!("debug");
    let attr_id_bound = format_ident!("bound");

    let mut bounds = vec![];
    //Meta::Path: `#[abc::def]`
    //Meta::List: `#[derive(Copy, Clone)]` `#[debug(bound = "T::Value: Debug")]`
    //Meta::NameValue: `#[path = "sys/windows.rs"]`
    for Attribute { meta, .. } in attrs {
        if let Meta::List(MetaList { path, tokens, .. }) = meta
            && path.is_ident(&attr_id_debug)
        {
            let mut tokens_iter = tokens.to_token_stream().into_iter();
            if let Some(TokenTree::Ident(id)) = tokens_iter.next()
                && id == attr_id_bound
                && let Some(TokenTree::Punct(punct_eq)) = tokens_iter.next()
                && punct_eq.as_char() == '='
                && let Some(bound_val) = tokens_iter.next()
            {
                // must be a str "abc" ""
                if let TokenTree::Literal(ref bound_val) = bound_val
                    && let bound_val = Lit::new(bound_val.clone())
                    && let Lit::Str(s) = bound_val
                {
                    bounds.push(Ok(s));
                } else {
                    bounds.push(Err(
                        Error::new_spanned(bound_val, "must be a string")
                    ));
                }
            }
        }
    }
    return bounds;
}
