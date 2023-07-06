#![feature(let_chains)]

use proc_macro2::*;
use syn::*;
use quote::*;

#[proc_macro_attribute]
pub fn sorted(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    
    let mut errors = vec![];

    let t = if let Ok(enum_item) = parse::<ItemEnum>(input.clone()) {
        let mut last: Option<&Ident> = None;
        for v in enum_item.variants.iter() {
            let cur = &v.ident;
            if let Some(last) = last
                && last.cmp(cur).is_ge()
            {
                let pos = enum_item.variants.iter()
                    .map(|v| &v.ident)
                    .find(|vi| cur.cmp(vi).is_le())
                    .unwrap_or(last); 
                errors.push(Error::new(cur.span(), format!("{} should sort before {}", cur, pos)));
            }
            last = Some(cur);
        }
        enum_item.to_token_stream()
    } else if let Ok(match_expr) = parse::<ExprMatch>(input.clone()) {
        match_expr.to_token_stream()
    } else {
        errors.push(Error::new(Span::call_site(), "expected enum or match expression"));
        TokenStream::from(input)
    };

    let errors = errors.iter().map(Error::to_compile_error);
    quote! {
        #(#errors)*
        #t
    }.into()
}
