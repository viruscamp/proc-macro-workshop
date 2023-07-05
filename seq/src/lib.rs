#![feature(let_chains)]

use proc_macro2::*;
use syn::*;
use syn::parse::*;
use quote::*;

struct Seq {
    name: Ident,
    from: LitInt,
    to: LitInt,
    body: Group,
}
impl Parse for Seq {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let from: LitInt = input.parse()?;
        input.parse::<Token![..]>()?;
        let to: LitInt = input.parse()?;
        let body: Group = input.parse()?;
        Ok(Seq {
            name,
            from,
            to,
            body,
        })
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Seq { name, from, to, body } = parse_macro_input!(input as Seq);
    quote! {
        
    }.into()
}
