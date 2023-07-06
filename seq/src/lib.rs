#![feature(let_chains)]

use std::collections::VecDeque;

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
    let mut input = input.into_iter();
    let mut unprocessed = VecDeque::new();
    fn read_one(vec: &mut VecDeque<TokenTree>, iter: &mut impl Iterator<Item = TokenTree>)
        -> Option<TokenTree> {
            if let Some(tt) = vec.pop_front() {
                Some(tt)
            } else if let Some(tt) = iter.next() {
                Some(tt)
            } else {
                None
            }
    }
    while let Some(t) = read_one(&mut unprocessed, &mut input) {
        match t {
            TokenTree::Ident(id) => {
                if &id == toreplace {
                    //IN => 1
                    //eprintln!("IN => 1 {id:?}");
                    output.append_all(replaceby.clone().into_iter())
                } else {
                    let t1 = read_one(&mut unprocessed, &mut input);
                    let t2 = read_one(&mut unprocessed, &mut input);
                    let t3 = read_one(&mut unprocessed, &mut input);
                    let t4 = read_one(&mut unprocessed, &mut input);
                    if let Some(TokenTree::Punct(ref punct)) = t1
                        && punct.as_char() == '~'
                        && let Some(TokenTree::Ident(ref idn)) = t2
                        && idn == toreplace
                    {
                        if let Some(TokenTree::Punct(ref punct)) = t3
                        && punct.as_char() == '~'
                        && let Some(TokenTree::Ident(ref postfix)) = t4
                        {
                            //Id~IN~id => Id1id
                            //eprintln!("Id~IN~id => Id1id {id:?}{replaceby:?}{postfix:?}");
                            let id = format_ident!("{id}{replaceby}{postfix}");
                            output.append(id);
                        } else {
                            //Id~IN => Id1
                            //eprintln!("IId~IN => Id1 {id:?}{replaceby:?}");
                            let id = format_ident!("{id}{replaceby}");
                            output.append(id);
                            unprocessed.extend(t3);
                            unprocessed.extend(t4);
                        }
                    } else {
                        // Id
                        //eprintln!("Id => Id {id:?} push_back 4 [{t1:?} {t2:?} {t3:?} {t4:?}]");
                        output.append(id);
                        unprocessed.extend(t1);
                        unprocessed.extend(t2);
                        unprocessed.extend(t3);
                        unprocessed.extend(t4);
                    }
                }
            },
            TokenTree::Group(g) => {
                let group_inner = replace_ident(g.stream(), toreplace, replaceby);
                let mut g_new = Group::new(g.delimiter(), group_inner);
                g_new.set_span(g.span());
                output.append(g_new)
            },
            other => output.append(other),
        }
    }
    output
}
