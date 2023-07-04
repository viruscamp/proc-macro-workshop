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

    // 方法4 检查每个泛型参数是否在field中使用，Phantom<T> 不算使用, Box<Option<T>> 算
    let mut generics_params_used = input.generics.params.iter().map(|gp| {
        if let GenericParam::Type(TypeParam { ident, ..   }) = gp {
            (Some(ident.clone()), false)
        } else {
            (None, false)
        }
    }).collect::<Vec<_>>();

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
            let fty = &f.ty;
            generics_params_used.iter_mut()
            .filter(|(_, used)| !used)
            .for_each(|(id, used)| {
                if let Some(id) = id
                    && contains_generic_param(fty, id) {
                    *used = true
                }
            });
        }
    } else {
        errors.push(Error::new_spanned(&input.ident, "should be struct"));
    }

    eprintln!("{struct_name} {generics_params_used:?}");
    let debug_trait_bound = syn::parse2::<TypeParamBound>(quote!{
        ::core::fmt::Debug
    }).unwrap();
    for (idx, gp) in input.generics.params.iter_mut().enumerate() {
        if let GenericParam::Type(tp) = gp 
            && generics_params_used[idx].1
        {
            tp.bounds.extend(Some(debug_trait_bound.clone()));
        }
    }

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
        }).unwrap();
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
            }).unwrap();
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
    }).unwrap();
    where_clause.predicates.extend(Some(field_debug_where));
}

// all segments must be ident, ignore all <>
fn is_path(p: &Path, tocmp: &Path) -> bool {
    if p.leading_colon != tocmp.leading_colon {
        return false;
    }
    if p.segments.len() != tocmp.segments.len() {
        return false;
    }
    for (idx, s) in p.segments.iter().enumerate() {
        let tocmps = &tocmp.segments[idx];
        if s.ident != tocmps.ident {
            return false;
        }
    }
    return true;
}

fn types_phantom() -> Vec<Path> {
    vec![
        syn::parse2::<Path>(quote!(PhantomData)).unwrap(),
        syn::parse2::<Path>(quote!(::core::marker::PhantomData)).unwrap(),
        syn::parse2::<Path>(quote!(core::marker::PhantomData)).unwrap(),
        syn::parse2::<Path>(quote!(::std::marker::PhantomData)).unwrap(),
        syn::parse2::<Path>(quote!(std::marker::PhantomData)).unwrap(),
    ]
}

fn is_phantom(path: &Path) -> Option<&Type> {
    if let Some(last) = path.segments.last()
        && let PathArguments::AngleBracketed(ref tps) = last.arguments
        && tps.args.len() == 1
        && let Some(GenericArgument::Type(ty)) = tps.args.first()
        && types_phantom().iter()
            .find(|phantom_type| is_path(path, phantom_type))
            .is_some()
    {
        return Some(ty);
    }
    return None;
}

// will ignore PhantomData<T>
fn contains_generic_param(ty: &Type, gpid: &Ident) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if Some(gpid) == path.get_ident() {
                true
            } else if is_phantom(path).is_some() {
                false
            } else if let Some(PathSegment {
                arguments: PathArguments::AngleBracketed(
                    AngleBracketedGenericArguments {
                        args,
                        .. 
                    }
                ),
                ..
            }) = path.segments.last() {
                args.iter().any(|arg| {
                    match arg {
                        GenericArgument::Type(ref ty)
                            | GenericArgument::AssocType(AssocType { ref ty, ..  })
                        => contains_generic_param(ty, gpid),
                        _ => false,
                    }
                })
            } else {
                false
            }
        },
        Type::Tuple(TypeTuple { elems, .. }) => {
            elems.iter().any(|ty| contains_generic_param(ty, gpid))
        },
        Type::Paren(TypeParen { elem, .. })
            | Type::Array(TypeArray { elem, .. }) 
            | Type::Slice(TypeSlice { elem, ..  })
            | Type::Reference(TypeReference { elem, ..  }) 
        => {
            contains_generic_param(elem.as_ref(), gpid)
        },
        _ => false,
    }
}
