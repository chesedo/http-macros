# http-macros

[![crates.io](https://img.shields.io/crates/v/http-macros.svg)](https://crates.io/crates/http-macros)
[![docs.rs](https://docs.rs/http-macros/badge.svg)](https://docs.rs/http-macros)
[![build status](https://github.com/chesedo/http-macros/workflows/CI/badge.svg)](https://github.com/chesedo/http-macros/actions)

`http-macros` is a Rust library providing macros to simplify the creation of HTTP request types for testing purposes. 

## Features

- Simplified HTTP request type creation.
- Convenient for writing tests with minimal boilerplate.
- Supports common HTTP methods and versions.

## Installation

Add `http-macros` to your `Cargo.toml`:

```toml
[dependencies]
http-macros = "0.1.0"
```

## Usage

```rust
use http_macros::request;

let request = request!(
   r#"POST /hello HTTP/3.0
      Host: example.com
      Content-Type: application/json

      { "name": "John Doe" }
"#);

assert_eq!(request.method(), http::Method::POST);
assert_eq!(request.uri().path(), "/hello");
assert_eq!(request.version(), http::Version::HTTP_3);
assert_eq!(request.headers().get("Host").unwrap(), "example.com");
assert_eq!(request.headers().get("Content-Type").unwrap(), "application/json");
assert_eq!(request.body(), &r#"{ "name": "John Doe" }"#);
```

Here, `request` is an [http::Request](https://docs.rs/http/latest/http/request/struct.Request.html) which can be used to test an HTTP server.
The only required inputs are the method and the uri.
The version, headers and body are all optional.


Sometimes you might want to have a more complex body.
For these cases you can use `request_builder!` instead to get an [http::request::Builder](https://docs.rs/http/latest/http/request/struct.Builder.html) so that you can manually set the request body.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT License.

## Shoutouts
This macro is inspired by the [REST client VsCode extension](https://marketplace.visualstudio.com/items?itemName=humao.rest-client)
