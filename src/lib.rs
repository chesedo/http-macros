use proc_macro::{Span, TokenStream};
use proc_macro_error::{abort, proc_macro_error};

mod parser;
mod request;
mod request_builder;
mod token_helpers;

#[proc_macro_error]
#[proc_macro]
pub fn request_builder(input: TokenStream) -> TokenStream {
    let input = get_request(input);

    let builder = request_builder::RequestBuilder::new(&input);

    quote::quote! {
        #builder
    }
    .into()
}

#[proc_macro_error]
#[proc_macro]
pub fn request(input: TokenStream) -> TokenStream {
    let input = get_request(input);

    let request = request::Request::new(&input);

    quote::quote! {
        #request
        .unwrap()
    }
    .into()
}

/// Get the actual request from the macro input
fn get_request(input: TokenStream) -> String {
    // `TokenStream` eats up the space characters. However, to match the RFC 7230 spec we need each header to be on a new line.
    // So to preserve the new lines, the input needs to be a string literal when the input is a multi-line string.
    // So check if this input is a string literal or not
    let Some(first_token) = input.clone().into_iter().next() else {
        abort!(
            Span::call_site(),
            "Missing request";
            help = "Try `request!(GET /hello)`"
        );
    };

    match first_token {
        proc_macro::TokenTree::Literal(lit) => {
            // Remove the quotes from the string literal
            // And trim the leading and trailing whitespaces
            lit.to_string()
                .trim_start_matches("r#")
                .trim_end_matches("#")
                .trim_matches('"')
                .lines()
                .map(|line| line.trim())
                .collect::<Vec<_>>()
                .join("\n")
        }
        proc_macro::TokenTree::Ident(_) => input.to_string(),
        proc_macro::TokenTree::Group(g) => abort!(
            g.span(),
            "Unexpected group";
            help = "Try `request!({})`", g.stream().to_string()
        ),
        proc_macro::TokenTree::Punct(p) => {
            abort!(
                p.span(),
                "Unexpected token";
                help = "Try `request!(GET /hello)`"
            );
        }
    }
}
