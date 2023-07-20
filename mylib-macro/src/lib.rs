#![feature(let_chains)]

use std::collections::HashSet;

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
    if let Type::Path(TypePath { path, qself: None }) = ty
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

// will ignore `PhantomData<T>`, `*mut T`, `*const T`
pub fn contains_generic_param(ty: &Type, gpid: &Ident) -> bool {
    match ty {
        Type::Path(TypePath { path, qself }) => {
            if Some(gpid) == path.get_ident() && qself.is_none() {
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
        Type::Ptr(TypePtr { elem: _, .. }) => false,
        _ => false,
    }
}

pub fn used_generic_param<'a, 'b>(ty: &'a Type, gpids: &'b [&'b Ident], path_with_param: &mut HashSet<&'a Path>)
    -> bool
{
    match ty {
        Type::Path(TypePath { path, qself }) => {
            if let Some(_qself) = qself {
                // <T::Value2 as Trait>::Value  qself.ty="T::Value2" position=1 path="Trait::Value"
                // <Vec<T>>::AssociatedItem<X>  qself.ty="Vec<T>" position=0 path="AssociatedItem<X>"
                // TODO 暂时处理不了
                false
            } else if let Some(gpid) = path.get_ident(){
                // T or u32
                if gpids.contains(&gpid) {
                    path_with_param.insert(path); // T
                    true
                } else {
                    false // u32
                }
            } else if is_phantom(ty).is_some() {
                // Phantom<T> Phantom<Box<T>>
                false
            } else if path.leading_colon.is_none()
                && let Some(PathSegment { ident, .. }) = path.segments.first()
                && gpids.contains(&ident)
            {
                // T::Value
                // T::Value3<i16>,
                // T::Value<X>
                path_with_param.insert(path);
                true
            } else if let Some(PathSegment {
                arguments: PathArguments::AngleBracketed(
                    AngleBracketedGenericArguments {
                        args,
                        .. 
                    }
                ),
                ..
            }) = path.segments.last() {
                let mut has = false;
                for arg in args {
                    has |= match arg {
                        GenericArgument::Type(ref ty)
                            | GenericArgument::AssocType(AssocType { ref ty, ..  })
                        => used_generic_param(ty, gpids, path_with_param),
                        _ => false,
                    };
                }
                has
            } else {
                false
            }
        },
        Type::Tuple(TypeTuple { elems, .. }) => {
            let mut has = false;
            for ty in elems {
                has |= used_generic_param(ty, gpids, path_with_param);
            }
            has
        },
        Type::Paren(TypeParen { elem, .. })
            | Type::Array(TypeArray { elem, .. })
            | Type::Slice(TypeSlice { elem, ..  })
            | Type::Reference(TypeReference { elem, ..  })
        => {
            used_generic_param(elem.as_ref(), gpids, path_with_param)
        },
        Type::Ptr(TypePtr { elem: _, .. }) => false,
        _ => false,
    }
}
