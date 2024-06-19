use std::collections::HashMap;

use quote::{quote, ToTokens};

use crate::parse_request;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Request {
    method: String,
    uri: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    pub fn new(input: &str) -> Self {
        let buf = input.as_bytes();
        let (method, uri, headers, offset) = parse_request(buf);

        Self {
            method,
            uri,
            headers,
            body: buf[offset..].to_vec(),
        }
    }
}

impl ToTokens for Request {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let method = &self.method;
        let uri = &self.uri;
        let headers = self.headers.iter().map(|(k, v)| {
            quote! {
                .header(#k, #v)
            }
        });
        let body = String::from_utf8(self.body.clone()).unwrap();

        let builder = quote! {
            http::Request::builder()
                .method(#method)
                .uri(#uri)
                #(#headers)*
                .body(#body)
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
            r#"POST /todo HTTP/1.1
Host: localhost:8000

{ "note": "Buy milk" }"#,
        );
        let expected = Request {
            method: "POST".to_string(),
            uri: "/todo".to_string(),
            headers: {
                let mut headers = HashMap::new();
                headers.insert("Host".to_string(), "localhost:8000".to_string());
                headers
            },
            body: "{ \"note\": \"Buy milk\" }".as_bytes().to_vec(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn output() {
        let input = Request {
            method: "GET".to_string(),
            uri: "/health".to_string(),
            headers: {
                let mut headers = HashMap::new();
                headers.insert("Host".to_string(), "localhost:8000".to_string());
                headers
            },
            body: "{ \"note\": \"Buy milk\" }".as_bytes().to_vec(),
        };
        let expected = quote! {
            http::Request::builder()
                .method("GET")
                .uri("/health")
                .header("Host", "localhost:8000")
                .body("{ \"note\": \"Buy milk\" }")
        };

        assert_eq!(input.to_token_stream().to_string(), expected.to_string());
    }
}
