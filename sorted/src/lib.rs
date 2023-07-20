#![feature(let_chains)]

use proc_macro2::*;
use syn::*;
use syn::parse::{ParseStream, Parse, discouraged::Speculative};
use quote::*;
use syn::spanned::Spanned;

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

fn enum_variant_from_arm(arm: &Arm) -> Option<(&Ident, Path)> {
    if let Pat::Ident(PatIdent { ident: cur, .. }) = &arm.pat
    {
        let path = Path::from(cur.clone());
        Some((cur, path))
    } else if let Pat::Path(PatPath { qself: None, path, .. })
        | Pat::TupleStruct(PatTupleStruct { qself: None, path, .. })
        = &arm.pat
        && let Some(ps) = path.segments.last()
    {
        Some((&ps.ident, path.clone()))
    } else {
        None
    }
}

fn check_sorted_expr_match(expr_match: &ExprMatch, errors: &mut Vec<Error>) {
    let mut idx_underscore = None;
    let mut last: Option<(&Ident, Path)> = None;
    for (idx, arm) in expr_match.arms.iter().enumerate() {
        if let Pat::Wild(_) = arm.pat {
            idx_underscore = Some(idx);
            continue;
        }
        if let Some(cur) = enum_variant_from_arm(&arm)
        {
            if let Some(ref last) = last
                && last.0.cmp(cur.0).is_ge()
            {
                let pos = expr_match.arms.iter()
                    .filter_map(enum_variant_from_arm)
                    .find(|vi| cur.0.cmp(vi.0).is_le())
                    .unwrap_or(last.clone());
                let cur_path = &cur.1;
                let pos_path = &pos.1;
                errors.push(Error::new(
                    cur_path.span(),
                    //& quote!(#cur_path should sort before #pos_path).to_string(),
                    &format!("{} should sort before {}", cur_path.span().source_text().unwrap(), pos_path.span().source_text().unwrap())
                ));
            }
            last = Some(cur);
        } else {
            errors.push(Error::new(
                arm.pat.span(),
                "unsupported by #[sorted]",
            ));
            break;
        }
    }
    if let Some(idx_underscore) = idx_underscore
        && idx_underscore < expr_match.arms.len() - 1
    {
        errors.push(Error::new(
            Span::call_site(),
            "_ should be last",
        ));
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