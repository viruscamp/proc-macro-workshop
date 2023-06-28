#![feature(let_chains)]

use proc_macro::{TokenStream};
use proc_macro2::{TokenTree};
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

            // find `#[builder(each = "arg")] args: Vec<String>`, `#[builder(each = arg)]`
            if let Some(each_method_name) = attrs.iter().find_map(|attr| {
                if let Attribute {
                    meta: Meta::List(MetaList {
                        path,
                        tokens,
                        ..
                    }),
                    ..
                } = attr
                    && let Some(id_builder) = path.get_ident()
                    && id_builder.to_string() == "builder"
                {
                    let mut tokens = tokens.to_token_stream().into_iter();
                    if let Some(TokenTree::Ident(id_each)) = tokens.next()
                        && id_each.to_string() == "each"
                        && let Some(TokenTree::Punct(punct_eq)) = tokens.next()
                        && punct_eq.as_char() == '='
                        && let Some(method_name) = tokens.next()
                    {
                        if let TokenTree::Literal(method_name) = method_name {
                            if let Ok(Lit::Str(s)) = syn::parse_str::<Lit>(&method_name.to_string()) {
                                return Some(Ident::new(&s.value(), method_name.span()));
                            } else {
                                // TODO report Error
                            }
                        } else if let TokenTree::Ident(method_name) = method_name {
                            return Some(method_name);
                        } else {
                            // TODO report Error
                        }
                    }
                }
                return None;
            }) {
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
                    // TODO report Error
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

        let expanded = quote! {
            impl #struct_name {
                pub fn builder() -> #struct_builder_name {
                    <#struct_builder_name as Default>::default()
                }
            }

            #[derive(Default)]
            pub struct #struct_builder_name {
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
        };

        TokenStream::from(expanded)
    } else {
        // TODO error
        TokenStream::new()
    }
}
