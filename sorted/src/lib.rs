use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let item = parse_macro_input!(input as syn::Item);
    
    match validate_enum(&item) {
        // Same as proc_macro::TokenStream::from(i.to_token_stream())
        Ok(i) => i.to_token_stream().into(),
        Err(e) => e.to_compile_error().into(), 
    }
}

fn validate_enum(item: &syn::Item) -> Result<&syn::ItemEnum, syn::Error> {
    if let syn::Item::Enum(item_enum) = item {
        Ok(item_enum)
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ))
    }
}