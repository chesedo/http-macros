use http::Method;
use http_macros::request;

#[test]
fn test_method() {
    let request = request!(GET /hello HTTP/1.1);
    assert_eq!(request.method(), Method::GET);
}

#[test]
fn test_uri() {
    let request = request!("POST /api/user HTTP/1.1");
    assert_eq!(request.uri().path(), "/api/user");
}

#[test]
fn test_headers() {
    let request = request!(
        "
       GET /hello HTTP/1.1
       Host: example.com
       User-Agent: rust-test
    "
    );
    assert_eq!(request.headers().get("Host").unwrap(), "example.com");
    assert_eq!(request.headers().get("User-Agent").unwrap(), "rust-test");
}

#[test]
fn test_body() {
    let request = request!(
        r#"POST /todo HTTP/1.1
           Host: example.com
           User-Agent: rust-test

           { "note": "Buy milk" }
    "#
    );
    assert_eq!(request.headers().get("Host").unwrap(), "example.com");
    assert_eq!(request.headers().get("User-Agent").unwrap(), "rust-test");
}

// TODO: Restore after writing own parser
// #[test]
// fn test_simple() {
//     request!(GET /hello);
// }