use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let builder_name = syn::Ident::new(&format!("{}Builder", struct_name), struct_name.span());

    // Check the named fields is Struct
    let fields = if let Data::Struct(ref data_struct) = input.data {
        if let Fields::Named(ref fields) = data_struct.fields {
            fields
        } else {
            unimplemented!("Not support for unnamed fields");
        }
    } else {
        unimplemented!("Only support for Struct");
    };

    // Define each builder Struct fields: Option<T> for the original type T
    let builder_fields = fields.named.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            #name: std::option::Option<#ty>
        }
    });

    // Initialize each field with None in the builder() function
    let builder_inits = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: std::option::Option::None
        }
    });

    // Generate code
    let expanded = quote! {
        pub struct #builder_name {
            #(#builder_fields,)*
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_inits,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
