use std::collections::HashMap;

use quote::{quote, ToTokens};

use crate::parse_request;

#[derive(Debug, PartialEq, Eq, Default)]
struct RequestBuilder {
    method: String,
    uri: String,
    headers: HashMap<String, String>,
}

impl RequestBuilder {
    fn new(input: &str) -> Self {
        let buf = input.as_bytes();
        let (method, uri, headers, _) = parse_request(buf);

        Self {
            method,
            uri,
            headers,
        }
    }
}

impl ToTokens for RequestBuilder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let method = &self.method;
        let uri = &self.uri;

        let builder = quote! {
            http::Request::builder()
                .method(#method)
                .uri(#uri)
        };

        builder.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "httparse always requires the version. Thus, will have to make own parser first"]
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
    fn basic_with_version() {
        let actual = RequestBuilder::new("GET /health HTTP/1.1");
        let expected = RequestBuilder {
            method: "GET".to_string(),
            uri: "/health".to_string(),
            ..Default::default()
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn with_headers() {
        let actual = RequestBuilder::new(
            r#"GET /health HTTP/1.1
Host: localhost:8000
Accept: application/json
"#, // TODO: remove the new line
        );
        let expected = RequestBuilder {
            method: "GET".to_string(),
            uri: "/health".to_string(),
            headers: {
                let mut headers = HashMap::new();
                headers.insert("Host".to_string(), "localhost:8000".to_string());
                headers.insert("Accept".to_string(), "application/json".to_string());
                headers
            },
        };

        assert_eq!(actual, expected);
    }

    #[test]
    #[ignore = "httparse always requires the version. Thus, will have to make own parser first"]
    fn with_malformed_headers() {
        let actual = RequestBuilder::new(
            r#"GET /health
Host: localhost:8000
Accept
"#, // TODO: remove the new line
        );
        let expected = RequestBuilder {
            method: "GET".to_string(),
            uri: "/health".to_string(),
            headers: {
                let mut headers = HashMap::new();
                headers.insert("Host".to_string(), "localhost:8000".to_string());
                headers.insert("Accept".to_string(), "application/json".to_string());
                headers
            },
        };

        // TODO: Test for error somehow
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
}
