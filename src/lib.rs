use proc_macro::TokenStream;

mod request;
mod request_builder;

#[proc_macro]
pub fn request_builder(input: TokenStream) -> TokenStream {
    let input = get_request(input);

    let builder = request_builder::RequestBuilder::new(&input);

    quote::quote! {
        #builder
    }
    .into()
}

#[proc_macro]
pub fn request(input: TokenStream) -> TokenStream {
    let input = get_request(input);

    let request = request::Request::new(&input);

    quote::quote! {
        #request
        .unwrap()
    }
    .into()
}

/// Get the actual request from the macro input
fn get_request(input: TokenStream) -> String {
    // `TokenStream` eats up the space characters. However, to match the RFC 7230 spec we need each header to be on a new line.
    // So to preserve the new lines, the input needs to be a string literal when the input is a multi-line string.
    match input.clone().into_iter().next().unwrap() {
        proc_macro::TokenTree::Literal(lit) => {
            // Remove the quotes from the string literal
            // And trim the leading and trailing whitespaces
            lit.to_string()
                .trim_start_matches("r#")
                .trim_end_matches("#")
                .trim_matches('"')
                .lines()
                .map(|line| line.trim())
                .collect::<Vec<_>>()
                .join("\n")
        }
        proc_macro::TokenTree::Ident(_) => input.to_string(),
        _ => panic!("Invalid input"), // TODO: Improve error message
    }
}

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

struct Parser<'a> {
    method: String,
    uri: String,
    headers: Vec<(String, String)>,
    body: &'a [u8],
}

impl<'a> Parser<'a> {
    fn new(buf: &'a [u8]) -> Parser<'a> {
        let mut tokenizer = Tokenizer::new(buf);

        let Some(method) = tokenizer.next() else {
            todo!("handle missing method");
        };

        let Some(uri) = tokenizer.next() else {
            todo!("handle missing uri");
        };

        if tokenizer.is_end() {
            return Self {
                method,
                uri,
                headers: vec![],
                body: tokenizer.rest(),
            };
        }

        if !tokenizer.was_newline() {
            todo!("unexpected extra request line item");
        }

        let mut headers = Vec::new();

        while !tokenizer.is_end() {
            // Double new line means end of headers and start of body
            if tokenizer.is_newline() {
                tokenizer.skip_newline();
                break;
            }

            let Some(name) = tokenizer.next() else {
                todo!("handle missing header name");
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

    #[test]
    #[should_panic = "unexpected extra request line item"]
    fn parser_extra_request_line_item() {
        let buf = b"GET /hello HTTP/1.1 extra";
        Parser::new(buf);
    }
}
