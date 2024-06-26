use proc_macro::Span;
use proc_macro_error::abort;
use quote::{quote, ToTokens};

use crate::{
    parser::Parser,
    token_helpers::{get_headers, get_version},
};

/// Represents a request builder (which does not have a body).
#[derive(Debug, PartialEq, Eq, Default)]
pub struct RequestBuilder {
    method: String,
    uri: String,
    version: Option<String>,
    headers: Vec<(String, String)>,
}

impl RequestBuilder {
    pub fn new(input: &str) -> Self {
        let buf = input.as_bytes();
        let Parser {
            method,
            uri,
            version,
            headers,
            body,
        } = Parser::new(buf);

        if !body.is_empty() {
            abort!(
                Span::call_site(),
                "The body of the request is not supported by ``request_builder!` Use `request!` instead."
            );
        }

        Self {
            method,
            uri,
            version,
            headers,
        }
    }
}

impl ToTokens for RequestBuilder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let method = &self.method;
        let uri = &self.uri;
        let version = get_version(self.version.as_ref());
        let headers = get_headers(self.headers.iter());

        let builder = quote! {
            http::Request::builder()
                .method(#method)
                .uri(#uri)
                #version
                #(#headers)*
        };

        builder.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let actual = RequestBuilder::new("GET /health");
        let expected = RequestBuilder {
            method: "GET".to_string(),
            uri: "/health".to_string(),
            ..Default::default()
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn with_version() {
        let actual = RequestBuilder::new("GET /health HTTP/1.1");
        let expected = RequestBuilder {
            method: "GET".to_string(),
            uri: "/health".to_string(),
            version: Some("HTTP/1.1".to_string()),
            ..Default::default()
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn with_headers() {
        let actual = RequestBuilder::new(
            r#"GET /health
Host: localhost:8000
Accept: application/json"#,
        );
        let expected = RequestBuilder {
            method: "GET".to_string(),
            uri: "/health".to_string(),
            version: None,
            headers: Vec::from([
                ("Host".to_string(), "localhost:8000".to_string()),
                ("Accept".to_string(), "application/json".to_string()),
            ]),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn basic_output() {
        let input = RequestBuilder {
            method: "GET".to_string(),
            uri: "/health".to_string(),
            ..Default::default()
        };
        let expected = quote! {
            http::Request::builder()
                .method("GET")
                .uri("/health")
        };

        assert_eq!(input.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn version_output() {
        let input = RequestBuilder {
            method: "GET".to_string(),
            uri: "/health".to_string(),
            version: Some("HTTP/1.0".to_string()),
            ..Default::default()
        };
        let expected = quote! {
            http::Request::builder()
                .method("GET")
                .uri("/health")
                .version(http::Version::HTTP_10)
        };

        assert_eq!(input.to_token_stream().to_string(), expected.to_string());
    }

    #[test]
    fn header_output() {
        let input = RequestBuilder {
            method: "PUT".to_string(),
            uri: "/hello".to_string(),
            version: None,
            headers: Vec::from([
                ("Host".to_string(), "localhost:8000".to_string()),
                ("Accept".to_string(), "application/json".to_string()),
            ]),
        };
        let expected = quote! {
            http::Request::builder()
                .method("PUT")
                .uri("/hello")
                .header("Host", "localhost:8000")
                .header("Accept", "application/json")
        };

        assert_eq!(input.to_token_stream().to_string(), expected.to_string());
    }
}
