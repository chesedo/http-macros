use quote::{quote, ToTokens};

use crate::{
    parser::Parser,
    token_helpers::{get_headers, get_version},
};

/// Represents a HTTP request (which has a body).
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Request {
    method: String,
    uri: String,
    version: Option<String>,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    pub fn new(input: &str) -> Self {
        let buf = input.as_bytes();
        let Parser {
            method,
            uri,
            version,
            headers,
            body,
        } = Parser::new(buf);

        Self {
            method,
            uri,
            version,
            headers,
            body: body.to_vec(),
        }
    }
}

impl ToTokens for Request {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let method = &self.method;
        let uri = &self.uri;
        let version = get_version(self.version.as_ref());
        let headers = get_headers(self.headers.iter());

        // Safe to unwrap since the TokenStream already makes sure it is a valid UTF-8 string
        let body = String::from_utf8(self.body.clone()).unwrap();

        let builder = quote! {
            http::Request::builder()
                .method(#method)
                .uri(#uri)
                #version
                #(#headers)*
                .body(#body.to_string())
        };

        builder.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let actual = Request::new(
            r#"POST /reminder
Host: localhost:8000

{ "note": "Buy milk" }"#,
        );
        let expected = Request {
            method: "POST".to_string(),
            uri: "/reminder".to_string(),
            version: None,
            headers: Vec::from([("Host".to_string(), "localhost:8000".to_string())]),
            body: "{ \"note\": \"Buy milk\" }".as_bytes().to_vec(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn output() {
        let input = Request {
            method: "GET".to_string(),
            uri: "/health".to_string(),
            version: Some("HTTP/2.0".to_string()),
            headers: Vec::from([("Host".to_string(), "localhost:8000".to_string())]),
            body: "{ \"note\": \"Buy milk\" }".as_bytes().to_vec(),
        };
        let expected = quote! {
            http::Request::builder()
                .method("GET")
                .uri("/health")
                .version(http::Version::HTTP_2)
                .header("Host", "localhost:8000")
                .body("{ \"note\": \"Buy milk\" }".to_string())
        };

        assert_eq!(input.to_token_stream().to_string(), expected.to_string());
    }
}
