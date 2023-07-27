#![feature(let_chains)]

use proc_macro2::*;
use quote::*;
use syn::buffer::TokenBuffer;
use syn::parse::*;
use syn::*;

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
    let Seq {
        name,
        from,
        inclusive,
        to,
        body,
    } = parse_macro_input!(input as Seq);
    let from = from.base10_parse::<i64>().unwrap();
    let to = to.base10_parse::<i64>().unwrap();
    let nums = if inclusive {
        from..=to
    } else {
        from..=(to - 1)
    };

    let (output, has_section) = repeat_section(body.stream(), &name, nums.clone());
    if has_section {
        output.into()
    } else {
        let mut output = TokenStream::new();
        for replaceby in nums.clone() {
            replace_ident(body.stream(), &name, replaceby).to_tokens(&mut output);
        }
        output.into()
    }
}

fn replace_ident(input: TokenStream, toreplace: &Ident, replaceby: i64) -> TokenStream {
    let mut output = TokenStream::new();
    let input = TokenBuffer::new2(input);
    let mut cur = input.begin();
    loop {
        cur = if true
            && let Some((id ,cur)) = cur.ident()
            && &id == toreplace
        {
            //IN => 1
            //eprintln!("IN => 1 {id:?}");
            let replaceby = LitInt::new(&replaceby.to_string(), id.span());
            replaceby.to_tokens(&mut output);
            cur
        } else if true
            && let Some((prefix, cur)) = cur.ident()
            && let Some((p, cur)) = cur.punct()
            && p.as_char() == '~'
            && let Some((id, cur)) = cur.ident()
            && &id == toreplace
        {
            let replaceby = if replaceby < 0 {
                format!("_{}", -replaceby)
            } else {
                format!("{}", replaceby)
            };
            if true
                && let Some((p, cur)) = cur.punct()
                && p.as_char() == '~'
                && let Some((postfix, cur)) = cur.ident()
            {
                //pre~IN~post => pre1post
                //eprintln!("pre~IN~post => pre1post {prefix:?}{replaceby:?}{postfix:?}");
                let newid = format_ident!("{prefix}{replaceby}{postfix}", span = prefix.span());
                output.append(newid);
                cur
            } else {
                //pre~IN => pre1
                //eprintln!("pre~IN => pre1 {prefix:?}{replaceby:?}");
                let newid = format_ident!("{prefix}{replaceby}", span = prefix.span());
                output.append(newid);
                cur
            }
        } else if let Some((tt, cur)) = cur.token_tree() {
            if let TokenTree::Group(g) = tt {
                let group_inner = replace_ident(g.stream(), toreplace, replaceby);
                let mut g_new = Group::new(g.delimiter(), group_inner);
                g_new.set_span(g.span());
                output.append(g_new)
            } else {
                output.append(tt)
            }
            cur
        } else {
            break;
        }
    }
    output
}

fn repeat_section(
    input: TokenStream,
    toreplace: &Ident,
    range: impl Iterator<Item = i64> + Clone,
) -> (TokenStream, bool) {
    let mut has_section = false;
    let mut output = TokenStream::new();
    let input = TokenBuffer::new2(input);
    let mut cur = input.begin();

    loop {
        cur = if true
        	&& let Some((p_begin, cur)) = cur.punct()
            && p_begin.as_char() == '#'

            && let Some((cur_g, _span, cur)) = cur.group(Delimiter::Parenthesis)

            && let Some((p_end, cur)) = cur.punct()
            && p_end.as_char() == '*'
        {
            has_section = true;
            for replaceby in range.clone() {
                replace_ident(cur_g.token_stream(), toreplace, replaceby)
                    .to_tokens(&mut output);
            }
            cur
        } else if let Some((tt, cur)) = cur.token_tree() {
            if let TokenTree::Group(g) = tt {
                let (group_inner, has_section_group) = repeat_section(g.stream(), toreplace, range.clone());
                has_section |= has_section_group;
                let mut g_new = Group::new(g.delimiter(), group_inner);
                g_new.set_span(g.span());
                output.append(g_new);
            } else {
                output.append(tt);
            }
            cur
        } else {
            // actual eof()
            break;
        }
    }
    (output, has_section)
}
