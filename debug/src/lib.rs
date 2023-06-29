use proc_macro2::*;
use syn::*;
use quote::*;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let vis = input.vis;
    let ident = &input.ident;
    let expand = quote!{
        // #vis #ident
    };

    proc_macro::TokenStream::from(expand)
}
