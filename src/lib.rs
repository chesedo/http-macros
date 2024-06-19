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

fn parse_request(buf: &[u8]) -> (String, String, Vec<(String, String)>, usize) {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let res = req.parse(buf).unwrap();

    let method = req.method.unwrap();
    let uri = req.path.unwrap();
    let headers = req
        .headers
        .iter()
        .filter(|h| !h.name.is_empty())
        .map(|h| {
            (
                h.name.to_string(),
                std::str::from_utf8(h.value).unwrap().to_string(),
            )
        })
        .collect::<Vec<_>>();

    let offset = match res {
        httparse::Status::Complete(offset) => offset,
        _ => 0,
    };

    (method.to_string(), uri.to_string(), headers, offset)
}
