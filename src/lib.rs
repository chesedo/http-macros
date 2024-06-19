use std::collections::HashMap;

mod request;
mod request_builder;

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
