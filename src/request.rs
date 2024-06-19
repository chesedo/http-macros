use std::collections::HashMap;

use crate::parse_request;

#[derive(Debug, PartialEq, Eq, Default)]
struct Request {
    method: String,
    uri: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    fn new(input: &str) -> Self {
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
}
