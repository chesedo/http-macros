use http_macros::request;

fn main() {
    let _req = request!(
        "GET /

         AB\xfc");
}
