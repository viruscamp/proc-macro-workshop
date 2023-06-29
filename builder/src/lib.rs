#![feature(let_chains)]

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::*;
use syn::*;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(DataStruct {
        fields: Fields::Named(ref fields),
        ..
    }) = input.data
    {
        let struct_name = input.ident;
        let struct_builder_name = format_ident!("{struct_name}Builder");

        let mut errors_builder = vec![];
        let mut fields_builder = vec![];
        let mut methods_builder = vec![];
        let mut build_internal = vec![];
        for f in &fields.named {
            let Field {
                ident, ty, attrs, ..
            } = f;

            // find `current_dir: Option<String>` or `args: Vec<String>`
            // failed for `current_dir: core::option::Option<String>`
            fn generic_inner(ty: &Type) -> Option<(&Ident, &Type)> {
                if let Type::Path(TypePath {
                    path: Path { segments, .. },
                    ..
                }) = ty
                {
                    if let Some(PathSegment {
                        ident,
                        arguments:
                            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                args, ..
                            }),
                    }) = segments.first()
                    {
                        if let Some(GenericArgument::Type(t)) = args.first() {
                            return Some((ident, t));
                        }
                    }
                }
                return None;
            }

            let generic_inner_a = generic_inner(ty);

            // find `#[builder(..)]`
            if let Some((attr, tokens)) = attrs.iter().find_map(|attr| {
                if let Meta::List(MetaList {
                        path,
                        tokens,
                        ..
                    }) = &attr.meta
                    && let Some(id_builder) = path.get_ident()
                    && id_builder.to_string() == "builder"
                {
                    return Some((attr, tokens));
                }
                return None;
            }) {
                // find `#[builder(each = "arg")] args: Vec<String>`, `#[builder(each = arg)]`
                fn get_each_method_name(attr: &Attribute, tokens: &TokenStream2) -> Result<Ident> {
                    let mut tokens_iter = tokens.to_token_stream().into_iter();
                    if let Some(TokenTree::Ident(id_each)) = tokens_iter.next()
                        && id_each.to_string() == "each"
                        && let Some(TokenTree::Punct(punct_eq)) = tokens_iter.next()
                        && punct_eq.as_char() == '='
                        && let Some(method_name) = tokens_iter.next()
                    {
                        if let TokenTree::Literal(method_name) = method_name {
                            if let Ok(Lit::Str(s)) = syn::parse_str::<Lit>(&method_name.to_string()) {
                                if let Ok(mut id) = syn::parse_str::<Ident>(&s.value()) {
                                    id.set_span(method_name.span());
                                    return Ok(id);
                                } else {
                                    return Err(
                                        Error::new_spanned(method_name, "not a valid ident")
                                    );
                                }
                            } else {
                                return Err(
                                    Error::new_spanned(method_name, "lit is not str")
                                );
                            }
                        } else if let TokenTree::Ident(method_name) = method_name {
                            return Ok(method_name);
                        } else {
                            return Err(
                                Error::new_spanned(method_name, "not lit str nor ident")
                            )
                        }
                    } else {
                        return Err(
                            Error::new_spanned(&attr.meta, "expected `builder(each = \"...\")`")
                        )
                    }
                }

                match get_each_method_name(attr, tokens) {
                    Ok(each_method_name) => {
                        if let Some((wrapper, ty_inner)) = generic_inner_a
                        && wrapper.to_string() == "Vec"
                    {
                        fields_builder.push(quote! {
                            #ident: #ty
                        });
                        methods_builder.push(quote! {
                            pub fn #each_method_name(&mut self, v: #ty_inner) -> &mut Self {
                                self.#ident.push(v);
                                self
                            }
                        });
                        build_internal.push(quote! {
                            #ident: core::mem::take(&mut self.#ident)
                        });
                    } else {
                        errors_builder.push(
                            Error::new_spanned(ty, "builder attr without Vec type").to_compile_error()
                        )
                    }
                    },
                    Err(err) => {
                        errors_builder.push(err.to_compile_error());
                    },
                }
            } else if let Some((wrapper, ty_inner)) = generic_inner_a
                && wrapper.to_string() == "Option"
            {
                fields_builder.push(quote! {
                    #ident: #ty
                });
                methods_builder.push(quote! {
                    pub fn #ident(&mut self, v: #ty_inner) -> &mut Self {
                        self.#ident = Some(v);
                        self
                    }
                });
                build_internal.push(quote!{
                    #ident: core::mem::take(&mut self.#ident)
                });
            } else {
                fields_builder.push(quote! {
                    #ident: Option<#ty>
                });
                methods_builder.push(quote! {
                    pub fn #ident(&mut self, v: #ty) -> &mut Self {
                        self.#ident = Some(v);
                        self
                    }
                });
                build_internal.push(quote!{
                    #ident: self.#ident.take()?
                });
            }
        }
        let vis = input.vis;
        let expanded = if errors_builder.is_empty() {
            quote! {
                impl #struct_name {
                    pub fn builder() -> #struct_builder_name {
                        <#struct_builder_name as Default>::default()
                    }
                }

                #[derive(Default)]
                #vis struct #struct_builder_name {
                    #(#fields_builder),*
                }

                impl #struct_builder_name {
                    pub fn build(&mut self) -> Option<#struct_name> {
                        Some(#struct_name {
                            #(#build_internal),*
                        })
                    }

                    #(#methods_builder)*
                }
            }
        } else {
            quote! {
                #(#errors_builder)*
            }
        };

        TokenStream::from(expanded)
    } else {
        let expanded = Error::new_spanned(&input, "should be struct").to_compile_error();
        TokenStream::from(expanded)
    }
}
