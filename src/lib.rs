use syn::parse::{Parse, ParseStream, Result};

#[derive(Debug, PartialEq, Eq)]
struct RequestBuilder {
    method: String,
    uri: String,
}

impl RequestBuilder {
    fn new(input: &str) -> Self {
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        let res = req.parse(input.as_bytes()).unwrap();

        let method = req.method.unwrap();
        let uri = req.path.unwrap();

        RequestBuilder {
            method: method.to_string(),
            uri: uri.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    #[ignore = "httparse always requires the version. Thus, will have to make own parser first"]
    fn basic() {
        let actual: RequestBuilder = RequestBuilder::new(r#"GET /health"#);
        let expected = RequestBuilder {
            method: "GET".to_string(),
            uri: "/health".to_string(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn basic_with_version() {
        let actual: RequestBuilder = RequestBuilder::new(r#"GET /health HTTP/1.1"#);
        let expected = RequestBuilder {
            method: "GET".to_string(),
            uri: "/health".to_string(),
        };

        assert_eq!(actual, expected);
    }
}
