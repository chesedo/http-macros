use http::Method;
use http_macros::request_builder;

#[test]
fn test_method() {
    let request = request_builder!("GET /hello");
    assert_eq!(request.method_ref().unwrap(), Method::GET);
}

#[test]
fn test_uri() {
    let request = request_builder!("POST /api/user");
    assert_eq!(request.uri_ref().unwrap().path(), "/api/user");
}

#[test]
fn test_version() {
    let request = request_builder!("PUT / HTTP/3.0");
    assert_eq!(request.version_ref().unwrap(), &http::Version::HTTP_3);
}

#[test]
fn test_headers() {
    let request = request_builder!(
        "GET /hello
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

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/request_builder/*.rs");
}
