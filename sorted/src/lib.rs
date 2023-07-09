#![feature(let_chains)]

use proc_macro2::*;
use syn::*;
use syn::parse::{ParseStream, Parse, discouraged::Speculative};
use quote::*;

#[proc_macro_attribute]
pub fn sorted(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    
    let mut errors = vec![];

    let t = if let Ok(enum_item) = parse::<ItemEnum>(input.clone()) {
        check_sorted_item_enum(&enum_item, &mut errors);
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

#[proc_macro_attribute]
pub fn check(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    let mut output = TokenStream::new();
    let mut errors = vec![];

    if let Err(err) = process_tokens(TokenStream::from(input).into_iter(), &mut output, &mut errors) {
        errors.push(err);
    }

    let errors = errors.iter().map(Error::to_compile_error);
    quote! {
        #(#errors)*
        #output
    }.into()
}

#[derive(Debug)]
struct TokenSearch<T: Parse> {
    found: Vec<(Vec<TokenTree>, T)>,
    tail: Vec<TokenTree>,
}
impl<T: Parse> Parse for TokenSearch<T>  {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut found = vec![];
        let mut tokens = vec![];

        while !input.is_empty() {
            let fork = input.fork();
            if let Ok(t) = input.parse::<T>() {
                found.push((tokens, t));
                tokens = vec![];
            } else {
                input.advance_to(&fork);
                let tt = input.parse::<TokenTree>()?;
                tokens.push(tt);
            }
        }
        Ok(Self {
            found,
            tail: tokens,
        })
    }
}

#[derive(Debug)]
struct MatchWithSorted {
    expr_match: ExprMatch,
}
impl Parse for MatchWithSorted {
    fn parse(input: ParseStream) -> Result<Self> {
        let id_sorted = format_ident!("sorted");
        let mut expr_match: ExprMatch = input.parse()?;
        let len_old = expr_match.attrs.len();
        expr_match.attrs.retain(|attr| {
            ! match &attr.meta {
                Meta::Path(path) => path,
                Meta::List(MetaList { path, .. }) => path,
                Meta::NameValue(MetaNameValue { path, ..}) => path,
            }.is_ident(&id_sorted)
        });

        if len_old == expr_match.attrs.len() {
            Err(Error::new(Span::call_site(), "no attr sorted"))
        } else {
            Ok(Self { expr_match })
        }
    }
}

fn process_sorted_match(input: TokenStream, errors: &mut Vec<Error>) -> Result<TokenStream> {
    let mut output = TokenStream::new();
    let search = parse2::<TokenSearch<MatchWithSorted>>(input)?;
    //eprintln!("search: {search:?}");
    for (tokens, expr_match) in search.found {
        process_tokens(tokens.into_iter(), &mut output, errors)?;
        check_sorted_expr_match(&expr_match.expr_match, errors);
        output.append_all(expr_match.expr_match.into_token_stream());
        // TODO 处理嵌套 match
    }
    process_tokens(search.tail.into_iter(), &mut output, errors)?;
    Ok(output)
}

fn process_tokens(tokens: impl Iterator<Item = TokenTree>, output: &mut TokenStream, errors: &mut Vec<Error>) -> Result<()> {
    for tt in tokens {
        match tt {
            TokenTree::Group(g) => {
                let new_stream = process_sorted_match(g.stream(), errors)?;
                let mut new_group = Group::new(g.delimiter(), new_stream);
                new_group.set_span(g.span());
                output.append(new_group)
            },
            _ => output.append(tt),
        }
    }
    Ok(())
}

fn enum_variant_from_arm(arm: &Arm) -> Option<&Ident> {
    if let Pat::Ident(PatIdent { ident: cur, .. }) = &arm.pat
    {
        Some(cur)
    } else if let Pat::Path(PatPath { qself: None, path, .. })
        | Pat::TupleStruct(PatTupleStruct { qself: None, path, .. })
        = &arm.pat
        && let Some(ps) = path.segments.last()
    {
        Some(&ps.ident)
    } else {
        None
    }
}

fn check_sorted_expr_match(expr_match: &ExprMatch, errors: &mut Vec<Error>) {
    let mut last: Option<&Ident> = None;
    for arm in expr_match.arms.iter() {
        if let Some(cur) = enum_variant_from_arm(&arm)
        {
            if let Some(last) = last
                && last.cmp(cur).is_ge()
            {
                let pos = expr_match.arms.iter()
                    .filter_map(|v| {
                        if let Pat::Path(PatPath { qself: None, path, .. }) = &v.pat
                            && let Some(cur) = path.get_ident()
                        {
                            Some(cur)
                        } else {
                            None
                        }
                    })
                    .find(|vi| cur.cmp(vi).is_le())
                    .unwrap_or(last); 
                errors.push(Error::new(cur.span(), format!("{} should sort before {}", cur, pos)));
            }
            last = Some(cur);
        }
    }
    // TODO check match expr in arms, bodies 
}

fn check_sorted_item_enum(enum_item: &ItemEnum, errors: &mut Vec<Error>) {
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
}