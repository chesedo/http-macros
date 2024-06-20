use http::Method;
use http_macros::request;

#[test]
fn test_method() {
    let request = request!(GET /hello);
    assert_eq!(request.method(), Method::GET);
}

#[test]
fn test_uri() {
    let request = request!("POST /api/user");
    assert_eq!(request.uri().path(), "/api/user");
}

#[test]
fn test_version() {
    let request = request!("PUT / HTTP/3.0");
    assert_eq!(request.version(), http::Version::HTTP_3);
}

#[test]
fn test_headers() {
    let request = request!(
        "GET /hello
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
        r#"POST /reminder
           Host: example.com
           User-Agent: rust-test

           { "note": "Buy milk" }
    "#
    );
    assert_eq!(request.headers().get("Host").unwrap(), "example.com");
    assert_eq!(request.headers().get("User-Agent").unwrap(), "rust-test");
    assert_eq!(*request.body(), "{ \"note\": \"Buy milk\" }\n");
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/request/*.rs");
}
