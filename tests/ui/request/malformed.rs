use http_macros::request;

fn main() {
    // Having an extra group
    let _req = request!({GET /});

    // Starting with a punctiation
    let _req = request!(!GET /);

    // Missing URI
    let _req = request!(POST);

    // Extra item
    let _req = request!(POST /reminder HTTP/1.1);
}
