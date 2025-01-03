use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut output = input.clone();
    let _ = args;
    let item = parse_macro_input!(input as syn::Item);
    
    if let Err(e) = validate_enum(item) {
        output.extend(TokenStream::from(e.to_compile_error()));
    }

    output
}

fn validate_enum(item: syn::Item) -> Result<(), syn::Error> {
    if let syn::Item::Enum(item_enum) = item {
        let variants = &item_enum.variants;

        for i in 0..variants.len() {
            let current = &variants[i];
            for j in 0..i {
                let earlier = &variants[j];
        
                if current.ident.to_string() < earlier.ident.to_string() {
                    return Err(
                        syn::Error::new(
                            current.ident.span(),
                            format!(
                                "{} should sort before {}",
                                current.ident, earlier.ident
                            )
                        )
                    );
                }
            }
        }

        Ok(())
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ))
    }
}