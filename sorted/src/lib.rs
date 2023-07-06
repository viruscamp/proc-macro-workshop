use proc_macro2::*;
use syn::*;
use quote::*;

#[proc_macro_attribute]
pub fn sorted(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    let enum_code = syn::parse_macro_input!(input as DeriveInput);
    let enum_name = enum_code.ident;

    let mut errors = vec![];
    if let Data::Enum(_) = enum_code.data {

    } else {
        errors.push(Error::new(enum_name.span(), "expected enum or match expression"));
    }


    let errors = errors.iter().map(Error::to_compile_error);
    quote! {
        #(#errors)*
    }.into()
}
