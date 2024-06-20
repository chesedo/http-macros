use proc_macro::Span;
use proc_macro_error::abort;

struct Tokenizer<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl Tokenizer<'_> {
    fn new(buf: &[u8]) -> Tokenizer {
        Tokenizer { buf, pos: 0 }
    }

    fn next(&mut self) -> Option<String> {
        let start = self.pos;
        let mut end = self.pos;

        while end < self.buf.len() {
            if self.buf[end] == b' ' || self.buf[end] == b'\n' {
                break;
            }

            end += 1;
        }

        if start == end {
            return None;
        }

        self.pos = end + 1;

        Some(
            std::str::from_utf8(&self.buf[start..end])
                .unwrap()
                .to_string(),
        )
    }

    fn is_end(&self) -> bool {
        self.pos >= self.buf.len()
    }

    fn was_newline(&self) -> bool {
        self.buf[self.pos - 1] == b'\n'
    }

    fn is_newline(&self) -> bool {
        self.buf[self.pos] == b'\n'
    }

    fn skip_newline(&mut self) {
        self.pos += 1;
    }
}

impl<'a> Tokenizer<'a> {
    fn rest(self) -> &'a [u8] {
        let Self { buf, pos } = self;

        if pos >= buf.len() {
            &[]
        } else {
            &buf[pos..]
        }
    }
}

pub struct Parser<'a> {
    pub method: String,
    pub uri: String,
    pub version: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: &'a [u8],
}

impl<'a> Parser<'a> {
    pub fn new(buf: &'a [u8]) -> Parser<'a> {
        let mut tokenizer = Tokenizer::new(buf);

        let Some(method) = tokenizer.next() else {
            unreachable!("already checked in `get_request` that at least something exists");
        };

        let Some(uri) = tokenizer.next() else {
            abort!(
                Span::call_site(),
                "Missing URI";
                help = "Try `request!({} /)`", method
            );
        };

        let mut version = None;

        if !tokenizer.is_end() && !tokenizer.was_newline() {
            version = tokenizer.next();
        }

        if tokenizer.is_end() {
            return Self {
                method,
                uri,
                version,
                headers: vec![],
                body: tokenizer.rest(),
            };
        }

        if !tokenizer.was_newline() {
            abort!(
                Span::call_site(),
                "unexpected extra request line item";
                help = "Try `request!({} {} {})`", method, uri, version.unwrap_or_default()
            );
        }

        let mut headers = Vec::new();

        while !tokenizer.is_end() {
            // Double new line means end of headers and start of body
            if tokenizer.is_newline() {
                tokenizer.skip_newline();
                break;
            }

            let Some(name) = tokenizer.next() else {
                unreachable!(
                    "this is not the end of the buffer, nor a new line, so there should be a name"
                );
            };

            let name = name.trim_end_matches(':').to_string();

            let mut value = Vec::new();

            // An empty value is valid - meaning we just saw a new line
            // A value can also consist of multiple tokens (seperated by spaces) so the end of a line means the end of a value
            // Or the end of the buffer also means the end of a value
            while !tokenizer.is_end() && !tokenizer.was_newline() {
                if let Some(part) = tokenizer.next() {
                    value.push(part);
                } else {
                    break;
                }
            }

            headers.push((name, value.join(" ")));
        }

        Self {
            method,
            uri,
            version,
            headers,
            body: tokenizer.rest(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer_next() {
        let buf = b"GET /hello HTTP/1.1\nHost: example.com\nUser-Agent: rust-test";
        let mut tokenizer = Tokenizer::new(buf);

        assert_eq!(tokenizer.next(), Some("GET".to_string()));
        assert_eq!(tokenizer.next(), Some("/hello".to_string()));
        assert_eq!(tokenizer.next(), Some("HTTP/1.1".to_string()));
        assert!(tokenizer.was_newline());
        assert_eq!(tokenizer.next(), Some("Host:".to_string()));
        assert_eq!(tokenizer.next(), Some("example.com".to_string()));
        assert!(tokenizer.was_newline());
        assert_eq!(tokenizer.next(), Some("User-Agent:".to_string()));
        assert_eq!(tokenizer.next(), Some("rust-test".to_string()));
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn parser_simple() {
        let buf = b"GET /hello";
        let parser = Parser::new(buf);

        assert_eq!(parser.method, "GET");
        assert_eq!(parser.uri, "/hello");
        assert_eq!(parser.headers, vec![]);
        assert_eq!(parser.body, b"");
    }

    #[test]
    fn parser_with_headers() {
        let buf = b"GET /hello\nHost: example.com\nUser-Agent: rust-test";
        let parser = Parser::new(buf);

        assert_eq!(parser.method, "GET");
        assert_eq!(parser.uri, "/hello");
        assert_eq!(
            parser.headers,
            Vec::from([
                ("Host".to_string(), "example.com".to_string()),
                ("User-Agent".to_string(), "rust-test".to_string())
            ])
        );
        assert_eq!(parser.body, b"");
    }

    #[test]
    fn parser_with_complex_headers() {
        let buf = b"GET /hello\nEmpty-Value:\nAccept: application/json; application/xml";
        let parser = Parser::new(buf);

        assert_eq!(parser.method, "GET");
        assert_eq!(parser.uri, "/hello");
        assert_eq!(
            parser.headers,
            Vec::from([
                ("Empty-Value".to_string(), "".to_string()),
                (
                    "Accept".to_string(),
                    "application/json; application/xml".to_string()
                )
            ])
        );
        assert_eq!(parser.body, b"");
    }

    #[test]
    fn parser_with_headers_and_body() {
        let buf =
            b"GET /hello\nHost: example.com\nUser-Agent: rust-test\n\n{ \"note\": \"Buy milk\" }";
        let parser = Parser::new(buf);

        assert_eq!(parser.method, "GET");
        assert_eq!(parser.uri, "/hello");
        assert_eq!(
            parser.headers,
            Vec::from([
                ("Host".to_string(), "example.com".to_string()),
                ("User-Agent".to_string(), "rust-test".to_string())
            ])
        );
        assert_eq!(parser.body, b"{ \"note\": \"Buy milk\" }");
    }
}
