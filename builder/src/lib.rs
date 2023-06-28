use proc_macro::TokenStream;
use syn::*;
use quote::*;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(DataStruct { fields: Fields::Named(ref fields), .. }) = input.data {

        let struct_name = input.ident;
        let struct_builder_name = format_ident!("{struct_name}Builder");

        let mut fields_builder = vec![];
        let mut methods_builder = vec![];
        let mut build_internal = vec![];
        for f in &fields.named {
            let Field { ident, ty, ..} = f;
            fields_builder.push(quote! {
                #ident: Option<#ty>
            });
            methods_builder.push(quote! {
                pub fn #ident(&mut self, v: #ty){
                    self.#ident = Some(v);
                }
            });
            build_internal.push(quote!{
                #ident: self.#ident.take()?
            });
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
