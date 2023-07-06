use proc_macro2::*;
use syn::*;
use quote::*;

#[proc_macro_attribute]
pub fn sorted(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    
    let mut errors = vec![];

    let t = if let Ok(enum_item) = parse::<ItemEnum>(input.clone()) {
        eprintln!("enum_item");
        enum_item.to_token_stream()
    } else if let Ok(match_expr) = parse::<ExprMatch>(input.clone()) {
        eprintln!("match_expr");
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
