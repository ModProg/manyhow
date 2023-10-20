use example_macro::*;

#[parse_quote_attribute("hello")]
fn test() {}

#[parse_quote_attribute(1)]
fn test() {}

#[parse_quote_attribute]
fn test() {}

#[parse_quote_dummy_attribute]
fn test_dummy() {}

#[parse_quote_dummy_error_attribute]
fn test_dummy2() {}

parse_quote_dummy!(
    fn test_dummy3() {}
);
parse_quote_dummy_error!(
    fn test_dummy4() {}
);

#[derive(ParseQuote)]
enum NoStruct{}

fn main() {
    // can be resolved through dummy
    test_dummy();
    test_dummy2();

    test_dummy3();
    test_dummy4();
}
