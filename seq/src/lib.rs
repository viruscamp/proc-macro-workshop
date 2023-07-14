#![feature(let_chains)]

use std::ops::RangeInclusive;

use proc_macro2::*;
use syn::*;
use syn::parse::*;
use quote::*;

use mylib_macro::push_back_iter::PushBackIterator;

struct Seq {
    name: Ident,
    from: LitInt,
    inclusive: bool,
    to: LitInt,
    body: Group,
}
impl Parse for Seq {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let from: LitInt = input.parse()?;
        
        let inclusive = if input.peek(Token![..=]) {
            input.parse::<Token![..=]>()?;
            true
        } else if input.peek(Token![..]) {
            input.parse::<Token![..]>()?;
            false
        } else {
            Err(Error::new(input.span(), "must be .. or ..="))?
        };

        let to: LitInt = input.parse()?;
        let body: Group = input.parse()?;
        Ok(Seq {
            name,
            from,
            inclusive,
            to,
            body,
        })
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Seq { name, from, inclusive, to, body } = parse_macro_input!(input as Seq);
    let from = from.base10_parse::<i32>().unwrap();
    let mut to = to.base10_parse::<i32>().unwrap();
    if !inclusive {
        to -= 1;
    }

    let (output, has_section) = repeat_section(body.stream(), &name, from..=to);
    if has_section {
        output.into()
    } else {
        let mut output = TokenStream::new();
        for replaceby in from..=to {
            let replaceby = LitInt::new(&replaceby.to_string(), Span::call_site());
            output.extend(replace_ident(body.stream(), &name, &replaceby.to_token_stream()));
        }
        output.into()
    }
}

fn replace_ident(
    input: TokenStream,
    toreplace: &Ident,
    replaceby: &TokenStream
) -> TokenStream {
    let mut output = TokenStream::new();
    let mut input = PushBackIterator::new(input.into_iter());
    while let Some(t) = input.next() {
        match t {
            TokenTree::Ident(id) => {
                if &id == toreplace {
                    //IN => 1
                    //eprintln!("IN => 1 {id:?}");
                    output.append_all(replaceby.clone().into_iter())
                } else {
                    let t1 = input.next();
                    let t2 = input.next();
                    let t3 = input.next();
                    let t4 = input.next();
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
                            let newid = format_ident!("{id}{replaceby}{postfix}", span = id.span());
                            output.append(newid);
                        } else {
                            //Id~IN => Id1
                            //eprintln!("IId~IN => Id1 {id:?}{replaceby:?}");
                            let newid = format_ident!("{id}{replaceby}", span = id.span());
                            output.append(newid);
                            input.extend(t3);
                            input.extend(t4);
                        }
                    } else {
                        // Id
                        //eprintln!("Id => Id {id:?} push_back 4 [{t1:?} {t2:?} {t3:?} {t4:?}]");
                        output.append(id);
                        input.extend(t1);
                        input.extend(t2);
                        input.extend(t3);
                        input.extend(t4);
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

fn repeat_section(
    input: TokenStream,
    toreplace: &Ident,
    range: RangeInclusive<i32>,
) -> (TokenStream, bool) {
    let mut has_section = false;
    let mut output = TokenStream::new();
    let mut input = PushBackIterator::new(input.into_iter());
    while let Some(t) = input.next() {
        match t {
            TokenTree::Punct(punct) => {
                if punct.as_char() == '#' {
                    let t1 = input.next();
                    let t2 = input.next();
                    if let Some(TokenTree::Group(ref g)) = t1
                        && g.delimiter() == Delimiter::Parenthesis
                        && let Some(TokenTree::Punct(ref punct)) = t2
                        && punct.as_char() == '*'
                    {
                        has_section = true;
                        for replaceby in range.clone().into_iter() {
                            let replaceby = LitInt::new(&replaceby.to_string(), Span::call_site());
                            output.extend(replace_ident(g.stream(), toreplace, &replaceby.to_token_stream()));
                        }
                    } else {
                        output.append(punct);
                        input.extend(t1);
                        input.extend(t2);
                    }
                } else {
                    output.append(punct);
                }
            },
            TokenTree::Group(g) => {
                let (group_inner, has_section_group) = repeat_section(g.stream(), toreplace, range.clone());
                has_section |= has_section_group;
                let mut g_new = Group::new(g.delimiter(), group_inner);
                g_new.set_span(g.span());
                output.append(g_new)
            },
            other => output.append(other),
        }
    }
    (output, has_section)
}
