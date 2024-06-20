use proc_macro::{Span, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};

mod request;
mod request_builder;

#[proc_macro_error]
#[proc_macro]
pub fn request_builder(input: TokenStream) -> TokenStream {
    let input = get_request(input);

    let builder = request_builder::RequestBuilder::new(&input);

    quote::quote! {
        #builder
    }
    .into()
}

#[proc_macro_error]
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

fn get_version(version: Option<&String>) -> Option<proc_macro2::TokenStream> {
    version
        .map(|v| match v.as_str() {
            "HTTP/0.9" => "HTTP_09",
            "HTTP/1.0" => "HTTP_10",
            "HTTP/1.1" => "HTTP_11",
            "HTTP/2.0" => "HTTP_2",
            "HTTP/3.0" => "HTTP_3",
            _ => abort!(
                Span::call_site(),
                "Invalid HTTP version";
                help = "Valid versions are: HTTP/0.9, HTTP/1.0, HTTP/1.1, HTTP/2.0, HTTP/3.0"
            ),
        })
        .map(|v| format_ident!("{}", v))
        .map(|v| quote! { .version(http::Version::#v) })
}

/// Get the actual request from the macro input
fn get_request(input: TokenStream) -> String {
    // `TokenStream` eats up the space characters. However, to match the RFC 7230 spec we need each header to be on a new line.
    // So to preserve the new lines, the input needs to be a string literal when the input is a multi-line string.
    // So check if this input is a string literal or not
    let Some(first_token) = input.clone().into_iter().next() else {
        abort!(
            Span::call_site(),
            "Missing request";
            help = "Try `request!(GET /hello)`"
        );
    };

    match first_token {
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
        proc_macro::TokenTree::Group(g) => abort!(
            g.span(),
            "Unexpected group";
            help = "Try `request!({})`", g.stream().to_string()
        ),
        proc_macro::TokenTree::Punct(p) => {
            abort!(
                p.span(),
                "Unexpected token";
                help = "Try `request!(GET /hello)`"
            );
        }
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
    version: Option<String>,
    headers: Vec<(String, String)>,
    body: &'a [u8],
}

impl<'a> Parser<'a> {
    fn new(buf: &'a [u8]) -> Parser<'a> {
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
