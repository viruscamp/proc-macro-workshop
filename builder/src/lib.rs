use proc_macro::TokenStream;
use syn::*;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    dbg!(&derive_input);
    TokenStream::new()
}
