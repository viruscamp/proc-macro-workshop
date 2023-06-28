use proc_macro::TokenStream;
use syn::*;
use quote::*;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let struct_builder_name = format_ident!("{struct_name}Builder");

    let expanded = quote! {
        impl #struct_name {
            pub fn builder() -> #struct_builder_name {
                #struct_builder_name::default()
            }
        }

        #[derive(Default)]
        pub struct #struct_builder_name {
        }
    };

    TokenStream::from(expanded)
}
