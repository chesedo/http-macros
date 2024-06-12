use syn::parse::{Parse, ParseStream, Result};

#[derive(Debug, PartialEq, Eq)]
struct RequestBuilder;

impl Parse for RequestBuilder {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(RequestBuilder)
    }
}
#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn basic() {
        let actual: RequestBuilder = parse_quote! {};
        let expected = RequestBuilder;
        assert_eq!(actual, expected);
    }
}
