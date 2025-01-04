use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse_macro_input;
use syn::visit_mut::VisitMut;

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
                    return Err(syn::Error::new(
                        current.ident.span(),
                        format!("{} should sort before {}", current.ident, earlier.ident),
                    ));
                }
            }
        }

        Ok(())
    } else {
        Err(syn::Error::new(Span::call_site(), "expected enum or match expression"))
    }
}

// #[sorted::check] macro: check if the arms of a match expression with
// #[sorted] attribute in the function are sorted, and remove #[sorted]
// attribute.
#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_fn = parse_macro_input!(input as syn::ItemFn);

    // Create a visitor to find and rewrite match expressions.
    let mut visitor = MatchVisitor { errors: Vec::new() };
    visitor.visit_item_fn_mut(&mut item_fn);

    // Converted function to TokenStream.
    let mut output = quote! {
        #item_fn
    };

    // Add any erros that have accumulated within the visitor.
    for e in visitor.errors {
        output.extend(e.to_compile_error());
    }

    TokenStream::from(output)
}

// Visitor to look for and check match expressions in functions.
struct MatchVisitor {
    pub errors: Vec<syn::Error>,
}

impl VisitMut for MatchVisitor {
    // find match and expr
    fn visit_expr_match_mut(&mut self, expr_match: &mut syn::ExprMatch) {
        if let Some(pos) = expr_match.attrs.iter().position(|attr| attr.path().is_ident("sorted")) {
            if let Err(e) = is_sorted_match_arms(expr_match) {
                self.errors.push(e);
            }

            // Remove #[sorted] attribute to prevent stable compiler from
            // generating erros.
            expr_match.attrs.remove(pos);
        }

        // If you want to process other child elements recursively, call the
        // superclass method.
        syn::visit_mut::visit_expr_match_mut(self, expr_match);
    }
}

fn is_sorted_match_arms(expr_match: &syn::ExprMatch) -> Result<(), syn::Error> {
    let mut arm_names: Vec<(String, &syn::Path)> = Vec::new();

    for arm in &expr_match.arms {
        if let Some((full_path_str, path)) = extract_variant_from_pat(&arm.pat) {
            arm_names.push((full_path_str, path));
        }
    }

    // Check if the retrieved variant names are sorted
    for i in 0..arm_names.len() {
        for j in 0..i {
            if arm_names[i].0 < arm_names[j].0 {
                return Err(syn::Error::new_spanned(
                    arm_names[i].1,
                    format!("{} should sort before {}", arm_names[i].0, arm_names[j].0),
                ));
            }
        }
    }

    Ok(())
}

fn extract_variant_from_pat(pat: &syn::Pat) -> Option<(String, &syn::Path)> {
    match pat {
        syn::Pat::Path(pat_path) => {
            let segments = &pat_path.path.segments;
            let full_path_str =
                segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");
            Some((full_path_str, &pat_path.path))
        }
        syn::Pat::TupleStruct(pat_ts) => {
            let segments = &pat_ts.path.segments;
            let full_path_str =
                segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");
            Some((full_path_str, &pat_ts.path))
        }
        syn::Pat::Struct(pat_struct) => {
            let segments = &pat_struct.path.segments;
            let full_path_str =
                segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");
            Some((full_path_str, &pat_struct.path))
        }
        _ => None,
    }
}
