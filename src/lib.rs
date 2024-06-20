use proc_macro::{Span, TokenStream};
use proc_macro_error::{abort, proc_macro_error};

mod parser;
mod request;
mod request_builder;
mod token_helpers;

/// Makes it easy to create a [http::request::Builder] from a request string that follows the RFC 7230 spec.
/// This allows you to manually set a body that is not supported by [request!].
///
/// # Simple Example
/// ```rust
/// use http_macros::request_builder;
///
/// let builder = request_builder!(GET /hello);
///
/// assert_eq!(builder.method_ref().unwrap(), http::Method::GET);
/// assert_eq!(builder.uri_ref().unwrap().path(), "/hello");
/// ```
///
/// # Example with headers and version
/// A request can also have headers and an optional version. Note, that whenever the request spans multiple lines, then it should be in double quotes.
///
/// ```rust
/// use http_macros::request_builder;
///
/// let builder = request_builder!(
///    "GET /hello HTTP/1.1
///     Host: example.com
///     Accept: */*
/// ");
///
/// assert_eq!(builder.method_ref().unwrap(), http::Method::GET);
/// assert_eq!(builder.uri_ref().unwrap().path(), "/hello");
/// assert_eq!(builder.version_ref().unwrap(), &http::Version::HTTP_11);
/// assert_eq!(builder.headers_ref().unwrap().get("Host").unwrap(), "example.com");
/// assert_eq!(builder.headers_ref().unwrap().get("Accept").unwrap(), "*/*");
/// ```
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

/// Creates a [http::Request] from a request string that follows the RFC 7230 spec.
/// This makes it easier to construct a request without having to use the builder API from [http::request::Builder].
///
/// # Simple Example
/// ```rust
/// use http_macros::request;
///
/// let request = request!(GET /hello);
///
/// assert_eq!(request.method(), http::Method::GET);
/// assert_eq!(request.uri().path(), "/hello");
/// ```
///
/// # Example with headers, version and body
/// A request can also have headers and an optional version like [request_builder!].
///
/// However, this macro can also take in a body for the request. A body is a string and is separated by an empty line from the headers (if there are any headers).
/// The body is optional and can be omitted if not needed. Note, since the request spans multiple lines, it should be in double quotes.
/// ```rust
/// use http_macros::request;
///
/// let request = request!(
///    r#"POST /hello HTTP/3.0
///       Host: example.com
///       Content-Type: application/json
///
///       { "name": "John Doe" }
/// "#);
///
/// assert_eq!(request.method(), http::Method::POST);
/// assert_eq!(request.uri().path(), "/hello");
/// assert_eq!(request.version(), http::Version::HTTP_3);
/// assert_eq!(request.headers().get("Host").unwrap(), "example.com");
/// assert_eq!(request.headers().get("Content-Type").unwrap(), "application/json");
/// assert_eq!(request.body(), &r#"{ "name": "John Doe" }"#);
/// ```
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
                .trim_end_matches('#')
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
