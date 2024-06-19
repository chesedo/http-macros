use quote::{quote, ToTokens};

use crate::Parser;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Request {
    method: String,
    uri: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    pub fn new(input: &str) -> Self {
        let buf = input.as_bytes();
        let Parser {
            method,
            uri,
            headers,
            body,
        } = Parser::new(buf);

        Self {
            method,
            uri,
            headers,
            body: body.to_vec(),
        }
    }
}

impl ToTokens for Request {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let method = &self.method;
        let uri = &self.uri;
        let headers = self.headers.iter().map(|(n, v)| {
            quote! {
                .header(#n, #v)
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
            r#"POST /todo
Host: localhost:8000

{ "note": "Buy milk" }"#,
        );
        let expected = Request {
            method: "POST".to_string(),
            uri: "/todo".to_string(),
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
            headers: Vec::from([("Host".to_string(), "localhost:8000".to_string())]),
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
