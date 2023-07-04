#![feature(let_chains)]

use syn::*;
use quote::*;

// all segments must be ident, ignore all <>
pub fn is_path(p: &Path, tocmp: &Path) -> bool {
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

pub fn is_types_one_param<'a, 'b>(ty: &'a Type, mut types: impl Iterator<Item = &'b Path>) -> Option<&'a Type>
{
    if let Type::Path(TypePath { path, .. }) = ty
        && let Some(last) = path.segments.last()
        && let PathArguments::AngleBracketed(ref tps) = last.arguments
        && tps.args.len() == 1
        && let Some(GenericArgument::Type(ty)) = tps.args.first()
        && types
            .find(|phantom_type| is_path(path, phantom_type))
            .is_some()
    {
        return Some(ty);
    }
    return None;
}

pub fn types_phantom() -> Vec<Path> {
    vec![
        syn::parse2::<Path>(quote!(PhantomData)).unwrap(),
        syn::parse2::<Path>(quote!(::core::marker::PhantomData)).unwrap(),
        syn::parse2::<Path>(quote!(core::marker::PhantomData)).unwrap(),
        syn::parse2::<Path>(quote!(::std::marker::PhantomData)).unwrap(),
        syn::parse2::<Path>(quote!(std::marker::PhantomData)).unwrap(),
    ]
}

pub fn is_phantom(ty: &Type) -> Option<&Type> {
    is_types_one_param(ty, types_phantom().iter())
}

pub fn types_option() -> Vec<Path> {
    vec![
        syn::parse2::<Path>(quote!(Option)).unwrap(),
        syn::parse2::<Path>(quote!(::core::option::Option)).unwrap(),
        syn::parse2::<Path>(quote!(core::option::Option)).unwrap(),
        syn::parse2::<Path>(quote!(::std::option::Option)).unwrap(),
        syn::parse2::<Path>(quote!(std::option::Option)).unwrap(),
    ]
}

pub fn is_option(ty: &Type) -> Option<&Type> {
    is_types_one_param(ty, types_option().iter())
}

pub fn types_vec() -> Vec<Path> {
    vec![
        syn::parse2::<Path>(quote!(Vec)).unwrap(),
        syn::parse2::<Path>(quote!(::std::vec::Vec)).unwrap(),
        syn::parse2::<Path>(quote!(std::vec::Vec)).unwrap(),
    ]
}

pub fn is_vec(ty: &Type) -> Option<&Type> {
    is_types_one_param(ty, types_vec().iter())
}

// will ignore PhantomData<T>
pub fn contains_generic_param(ty: &Type, gpid: &Ident) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if Some(gpid) == path.get_ident() {
                true
            } else if is_phantom(ty).is_some() {
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
