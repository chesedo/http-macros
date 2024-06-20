use proc_macro::Span;
use proc_macro_error::abort;
use quote::{format_ident, quote};

/// Get the correct [http::Version] from a string.
pub fn get_version(version: Option<&String>) -> Option<proc_macro2::TokenStream> {
    version
        .map(|v| match v.as_str() {
            "HTTP/0.9" => "HTTP_09",
            "HTTP/1.0" => "HTTP_10",
            "HTTP/1.1" => "HTTP_11",
            "HTTP/2.0" => "HTTP_2",
            "HTTP/3.0" => "HTTP_3",
            _ => abort!(
                Span::call_site(),
                "Invalid HTTP version";
                help = "Valid versions are: HTTP/0.9, HTTP/1.0, HTTP/1.1, HTTP/2.0, HTTP/3.0"
            ),
        })
        .map(|v| format_ident!("{}", v))
        .map(|v| quote! { .version(http::Version::#v) })
}

/// Get the headers from a list of key-value pairs.
pub fn get_headers<'a>(
    headers: impl Iterator<Item = &'a (String, String)> + 'a,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    headers.map(|(name, value)| {
        quote! {
            .header(#name, #value)
        }
    })
}
