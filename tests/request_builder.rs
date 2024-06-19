use http::Method;
use http_macros::request_builder;

#[test]
fn test_method() {
    let request = request_builder!(GET /hello HTTP/1.1);
    assert_eq!(request.method_ref().unwrap(), Method::GET);
}

#[test]
fn test_uri() {
    let request = request_builder!("POST /api/user HTTP/1.1");
    assert_eq!(request.uri_ref().unwrap().path(), "/api/user");
}

#[test]
fn test_headers() {
    let request = request_builder!(
        "GET /hello HTTP/1.1
         Host: example.com
         User-Agent: rust-test
    "
    );
    assert_eq!(
        request.headers_ref().unwrap().get("Host").unwrap(),
        "example.com"
    );
    assert_eq!(
        request.headers_ref().unwrap().get("User-Agent").unwrap(),
        "rust-test"
    );
}

// TODO: Restore after writing own parser
// #[test]
// fn test_simple() {
//     request_builder!(GET /hello);
// }
