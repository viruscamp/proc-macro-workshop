#![feature(let_chains)]

use proc_macro2::*;
use syn::*;
use quote::*;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let mut errors = vec![];
    let mut fields_debug = vec![];

    let mut where_clause = input.generics.make_where_clause();

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
            where_bound_field(where_clause, f);
        }
    } else {
        errors.push(Error::new_spanned(&input.ident, "should be struct"));
    }

    //where_bound_generic(&mut input.generics);
    //params_bound_generic(&mut input.generics);

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

/// 方法1 `where T: Debug`<br/>
/// 可能做出 `where T: Copy, T: Debug`
fn where_bound_generic(generics: &mut Generics) {
    let tpidents: Vec<_> = generics.params.iter().filter_map(|p| {
        if let GenericParam::Type(tp) = p {
            Some(tp.ident.clone())
        } else {
            None
        }
    }).collect();
    let where_clause = generics.make_where_clause();
    for tpident in tpidents {
        let debug_trait_bound = syn::parse2::<WherePredicate>(quote!{
            #tpident: ::core::fmt::Debug
        }.to_token_stream()).unwrap();
        where_clause.predicates.extend(Some(debug_trait_bound))
    }
}

/// 方法2 给 `GenericParam` 加 `Debug` 限制<br/>
/// `struct Field042<T: Clone, X> where X: Sized {}`<br/>
/// `impl<T: Clone + Debug, X: Debug> Debug for Field042<T,X> where X: Sized {}`  
fn params_bound_generic(generics: &mut Generics) {
    for p in generics.params.iter_mut() {
        if let GenericParam::Type(tp) = p {
            let debug_trait_bound = syn::parse2::<TypeParamBound>(quote!{
                ::core::fmt::Debug
            }.to_token_stream()).unwrap();
            tp.bounds.extend(Some(debug_trait_bound));
        }
    }
}

/// 方法3 加 `FieldType: Debug` 到 where<br/>
/// `struct Field042<T: Clone, X> where X: Sized {}`<br/>
/// ```rust ignore
/// impl<T: Clone, X> Debug for Field042<T,X>
///     where X: Sized
///         Phantom<T>: Debug,
///         i32: Debug,
///         T: Debug,
///         T: Debug,
/// {}
/// ```
fn where_bound_field(where_clause: &mut WhereClause, f: &Field) {
    let field_type = &f.ty;
    let field_debug_where = syn::parse2::<WherePredicate>(quote!{
        #field_type: ::core::fmt::Debug
    }.to_token_stream()).unwrap();
    where_clause.predicates.extend(Some(field_debug_where));
}
