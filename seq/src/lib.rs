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
    let from = from.base10_parse::<i32>().unwrap();
    let to = to.base10_parse::<i32>().unwrap();
    let mut outputs = vec![];
    for toreplace in from..to {
        let toreplace = LitInt::new(&toreplace.to_string(), Span::call_site());
        let output = replace_ident(body.stream(), &name, &toreplace.to_token_stream());
        outputs.push(output);
    }
    TokenStream::from_iter(outputs).into()
}

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