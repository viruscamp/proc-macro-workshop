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
    //let range = [from..to];
    //let replaceby = from;
    let output = replace_ident(body.stream(), &name, &from.to_token_stream());
    quote! {
        #output
    }.into()
}

// wrong and useless
fn replace_ident(
    input: TokenStream,
    toreplace: &Ident,
    replaceby: &TokenStream
) -> TokenStream {
    let mut output = TokenStream::new();
    for t in input.into_iter() {
        match t {
            TokenTree::Ident(id) => {
                if &id == toreplace {
                    output.append_all(replaceby.clone().into_iter())
                } else {
                    output.append(id)
                }
            },
            TokenTree::Group(g) => {
                let group_inner = replace_ident(g.stream(), toreplace, replaceby);
                let g = Group::new(g.delimiter(), group_inner);
                output.append(g)
            },
            TokenTree::Punct(pt) => output.append(pt),
            TokenTree::Literal(lit) => output.append(lit)
        }
    }
    output
}