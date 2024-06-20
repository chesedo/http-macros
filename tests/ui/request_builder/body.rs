use http_macros::request_builder;

fn main() {
    let _req = request_builder!(
        "GET /hello HTTP/1.1
         Host: example.com

         Hello, World!"
    );
}
