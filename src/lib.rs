use std::collections::HashMap;

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

fn parse_request(buf: &[u8]) -> (String, String, HashMap<String, String>, usize) {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let res = req.parse(buf).unwrap();

    let method = req.method.unwrap();
    let uri = req.path.unwrap();
    let mut headers = HashMap::new();

    for header in req.headers {
        if !header.name.is_empty() {
            headers.insert(
                header.name.to_string(),
                std::str::from_utf8(&header.value).unwrap().to_string(),
            );
        }
    }

    let offset = match res {
        httparse::Status::Complete(offset) => offset,
        _ => 0,
    };

    (method.to_string(), uri.to_string(), headers, offset)
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
    fn request() {
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
