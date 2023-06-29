use proc_macro2::*;
use syn::*;
use quote::*;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let mut errors = vec![];
    let mut fields_debug = vec![];

    if let Data::Struct(DataStruct {
        fields: Fields::Named(ref fields),
        ..
    }) = input.data
    {
        for f in &fields.named {
            let field_name = &f.ident;
            fields_debug.push(quote! {
                .field(stringify!(#field_name), &self.#field_name)
            });
        }
    } else {
        errors.push(Error::new_spanned(&input, "should be struct"));
    }

    let errors = errors.iter().map(Error::to_compile_error);
    let expand = quote! {
        #(#errors)*
        impl ::core::fmt::Debug for Field {
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                fmt.debug_struct(stringify!(#struct_name))
                    #(#fields_debug)*
                    .finish()
            }
        }
    };

    proc_macro::TokenStream::from(expand)
}
