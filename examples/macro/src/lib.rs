use manyhow::{manyhow, Emitter, ErrorMessage, Result, SilentError};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;

type SilentResult = Result<TokenStream2, SilentError>;

#[manyhow(item_as_dummy)]
#[proc_macro_attribute]
pub fn attr_item_as_dummy(_input: TokenStream, _item: TokenStream) -> SilentResult {
    Err(SilentError)
}

#[manyhow()]
#[proc_macro_attribute]
pub fn attr_no_dummy(_input: TokenStream, _item: TokenStream) -> SilentResult {
    Err(SilentError)
}

#[manyhow()]
#[proc_macro_attribute]
pub fn attr_custom_dummy(
    _input: TokenStream,
    _item: TokenStream,
    dummy: &mut TokenStream2,
) -> SilentResult {
    *dummy = quote! {fn dummy(){}};
    Err(SilentError)
}

#[manyhow]
#[proc_macro_attribute]
pub fn attr_emit(_: TokenStream, _: TokenStream, emitter: &mut Emitter) -> TokenStream2 {
    emitter.emit(ErrorMessage::new(Span::call_site(), "example error"));
    quote! {fn output(){}}
}

#[manyhow(proc_macro_attribute)]
pub fn attr_flag(_input: TokenStream, _item: TokenStream) -> SilentResult {
    Err(SilentError)
}

#[manyhow(proc_macro_attribute, item_as_dummy)]
pub fn attr_flag_dummy(_input: TokenStream, _item: TokenStream) -> SilentResult {
    Err(SilentError)
}

#[manyhow(input_as_dummy)]
#[proc_macro]
pub fn input_as_dummy(_: TokenStream) -> SilentResult {
    Err(SilentError)
}

#[manyhow]
#[proc_macro]
pub fn no_dummy(_: TokenStream) -> SilentResult {
    Err(SilentError)
}

#[manyhow]
#[proc_macro]
pub fn custom_dummy(_: TokenStream, dummy: &mut TokenStream2) -> SilentResult {
    *dummy = quote! {fn dummy(){}};
    Err(SilentError)
}

#[manyhow]
#[proc_macro]
pub fn emit(_t: TokenStream, emitter: &mut Emitter) -> TokenStream2 {
    emitter.emit(ErrorMessage::new(Span::call_site(), "example error"));
    quote! {fn output(){}}
}

#[manyhow(proc_macro)]
pub fn flag(_: TokenStream) -> SilentResult {
    Err(SilentError)
}

#[manyhow]
#[proc_macro_derive(NoDummy)]
pub fn derive_no_dummy(_: TokenStream) -> SilentResult {
    Err(SilentError)
}

#[manyhow]
#[proc_macro_derive(Dummy)]
pub fn derive_dummy(_: TokenStream, dummy: &mut TokenStream2) -> SilentResult {
    *dummy = quote! {fn dummy(){}};
    Err(SilentError)
}

#[manyhow]
#[proc_macro_derive(Emit)]
pub fn derive_emit(_: TokenStream, emitter: &mut Emitter) -> TokenStream2 {
    emitter.emit(ErrorMessage::new(Span::call_site(), "example error"));
    quote! {fn output(){}}
}

#[manyhow(proc_macro_derive(Flag))]
pub fn derive_flag(_: TokenStream) -> SilentResult {
    Err(SilentError)
}

#[manyhow(impl_fn)]
#[proc_macro]
pub fn impl_fn(input: TokenStream2) -> TokenStream2 {
    input
}

#[test]
fn unit_test() {
    assert_eq!(impl_fn_impl(quote!(Hello World)).to_string(), "Hello World");
}

#[manyhow(impl_fn, input_as_dummy)]
#[proc_macro]
pub fn impl_fn_with_dummy(input: TokenStream2) -> TokenStream2 {
    input
}

#[test]
fn unit_test_with_dummy() {
    assert_eq!(
        impl_fn_with_dummy_impl(quote!(Hello World)).to_string(),
        "Hello World"
    );
}

mod module {
    use manyhow::SilentError;
    use proc_macro2::TokenStream;

    use crate::SilentResult;

    pub fn attr_use(_input: TokenStream, _item: TokenStream) -> SilentResult {
        Err(SilentError)
    }
}

#[manyhow(proc_macro_attribute)]
pub use module::attr_use;
