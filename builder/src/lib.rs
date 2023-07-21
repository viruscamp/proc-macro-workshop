#![feature(let_chains)]

use mylib_macro::*;
use proc_macro2::*;
use quote::*;
use syn::*;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let vis = &input.vis;
    let struct_name = &input.ident;
    let struct_builder_name = format_ident!("{struct_name}Builder");

    let mut errors = vec![];
    let mut fields_builder = vec![];
    let mut methods_builder = vec![];
    let mut build_internal = vec![];

    let attr_id_builder = format_ident!("builder");

    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { ref named, .. }),
        ..
    }) = input.data
    {
        for Field {
            ident, ty, attrs, ..
        } in named
        {
            // find `#[builder(..)]`
            if let Some((attr, tokens)) = attrs.iter().find_map(|attr| {
                if let Meta::List(MetaList {
                        path,
                        tokens,
                        ..
                    }) = &attr.meta
                    && path.is_ident(&attr_id_builder)
                {
                    return Some((attr, tokens));
                }
                return None;
            }) {
                match get_each_method_name(attr, tokens) {
                    Ok(each_method_name) => {
                        if let Some(ty_inner) = is_vec(ty) {
                            fields_builder.push(quote! {
                                #ident: ::std::vec::Vec<#ty_inner>
                            });
                            methods_builder.push(quote! {
                                pub fn #each_method_name(&mut self, v: #ty_inner) -> &mut Self {
                                    self.#ident.push(v);
                                    self
                                }
                            });
                            build_internal.push(quote! {
                                #ident: ::core::mem::take(&mut self.#ident)
                            });
                        } else {
                            errors
                                .push(Error::new_spanned(ty, "builder attr each without Vec type"))
                        }
                    }
                    Err(err) => {
                        errors.push(err);
                    }
                }
            } else if let Some(ty_inner) = is_option(ty) {
                fields_builder.push(quote! {
                    #ident: ::core::option::Option<#ty_inner>
                });
                methods_builder.push(quote! {
                    pub fn #ident(&mut self, v: #ty_inner) -> &mut Self {
                        self.#ident = ::core::option::Option::Some(v);
                        self
                    }
                });
                build_internal.push(quote! {
                    #ident: ::core::mem::take(&mut self.#ident)
                });
            } else {
                fields_builder.push(quote! {
                    #ident: ::core::option::Option<#ty>
                });
                methods_builder.push(quote! {
                    pub fn #ident(&mut self, v: #ty) -> &mut Self {
                        self.#ident = ::core::option::Option::Some(v);
                        self
                    }
                });
                build_internal.push(quote! {
                    #ident: self.#ident.take()?
                });
            }
        }
    } else {
        errors.push(Error::new_spanned(&input, "should be struct"));
    }

    let expanded = if errors.is_empty() {
        quote! {
            impl #struct_name {
                pub fn builder() -> #struct_builder_name {
                    <#struct_builder_name as ::core::default::Default>::default()
                }
            }

            #[derive(Default)]
            #vis struct #struct_builder_name {
                #(#fields_builder),*
            }

            impl #struct_builder_name {
                pub fn build(&mut self) -> ::core::option::Option<#struct_name> {
                    Some(#struct_name {
                        #(#build_internal),*
                    })
                }

                #(#methods_builder)*
            }
        }
    } else {
        // 一般直接把 errors 放前面就可以
        // 此处测试 08-unrecognized-attribute.rs 要求报错必须一模一样，所以有错误时不输出 impl
        let errors = errors.iter().map(Error::to_compile_error);
        quote! {
            #(#errors)*
        }
    };
    proc_macro::TokenStream::from(expanded)
}

// find `#[builder(each = "arg")] args: Vec<String>`, `#[builder(each = arg)]`
fn get_each_method_name(attr: &Attribute, tokens: &TokenStream) -> Result<Ident> {
    let attr_id_each = format_ident!("each");
    let mut tokens_iter = tokens.to_token_stream().into_iter();
    if let Some(TokenTree::Ident(id_each)) = tokens_iter.next()
        && id_each == attr_id_each
        && let Some(TokenTree::Punct(punct_eq)) = tokens_iter.next()
        && punct_eq.as_char() == '='
        && let Some(method_name) = tokens_iter.next()
    {
        if let TokenTree::Literal(method_name) = method_name {
            if let Ok(Lit::Str(s)) = syn::parse_str::<Lit>(&method_name.to_string()) {
                if let Ok(mut id) = syn::parse_str::<Ident>(&s.value()) {
                    // `#[builder(each = "arg")]` 判断了 "arg" 是否有效标识符
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
            // `#[builder(each = arg)]`
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

#[allow(dead_code)]
mod showcase {
    //#[derive(Builder)]
    pub struct Command {
        executable: String,
        vec_option: Vec<i32>,
        current_dir: Option<String>,
        //#[builder(each = "arg")]
        args: Vec<String>,
        //#[builder(each = env)]
        env: Vec<String>,
    }

    impl Command {
        pub fn builder() -> CommandBuilder {
            <CommandBuilder as ::core::default::Default>::default()
        }
    }
    #[derive(Default)]
    pub struct CommandBuilder {
        executable: ::core::option::Option<String>,
        vec_option: ::core::option::Option<Vec<i32>>,
        current_dir: ::core::option::Option<String>,
        args: ::std::vec::Vec<String>,
        env: ::std::vec::Vec<String>,
    }
    impl CommandBuilder {
        pub fn build(&mut self) -> ::core::option::Option<Command> {
            Some(Command {
                executable: self.executable.take()?,
                vec_option: self.vec_option.take()?,
                current_dir: ::core::mem::take(&mut self.current_dir),
                args: ::core::mem::take(&mut self.args),
                env: ::core::mem::take(&mut self.env),
            })
        }
        pub fn executable(&mut self, v: String) -> &mut Self {
            self.executable = ::core::option::Option::Some(v);
            self
        }
        pub fn vec_option(&mut self, v: Vec<i32>) -> &mut Self {
            self.vec_option = ::core::option::Option::Some(v);
            self
        }
        pub fn current_dir(&mut self, v: String) -> &mut Self {
            self.current_dir = ::core::option::Option::Some(v);
            self
        }
        pub fn arg(&mut self, v: String) -> &mut Self {
            self.args.push(v);
            self
        }
        pub fn env(&mut self, v: String) -> &mut Self {
            self.env.push(v);
            self
        }
    }
}
