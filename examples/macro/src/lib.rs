use manyhow::{bail, manyhow, Emitter, ErrorMessage, Result, SilentError};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;

type SilentResult = Result<TokenStream2, SilentError>;

#[manyhow(item_as_dummy)]
#[proc_macro_attribute]
pub fn attr_item_as_dummy(_input: TokenStream, _item: TokenStream) -> SilentResult {
    Err(SilentError)
}

#[manyhow(item_as_dummy)]
#[proc_macro_attribute]
pub fn attr_item_as_dummy_ok(_input: TokenStream, _item: TokenStream) -> TokenStream2 {
    quote!()
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

#[manyhow]
#[proc_macro]
pub fn parse_quote(input: syn::LitStr) -> syn::LitStr {
    input
}

#[manyhow(input_as_dummy)]
#[proc_macro]
pub fn parse_quote_dummy(input: syn::DeriveInput) -> syn::DeriveInput {
    input
}

#[manyhow(input_as_dummy)]
#[proc_macro]
pub fn parse_quote_dummy_error(_: TokenStream) -> Result<syn::Ident> {
    bail!("error message")
}

#[manyhow(input_as_dummy)]
#[proc_macro]
pub fn parse_quote_dummy_error_syn_result(_: TokenStream) -> syn::Result<syn::Ident> {
    bail!("error message")
}

#[manyhow]
#[proc_macro_attribute]
pub fn parse_quote_attribute(_: syn::LitStr, item: syn::DeriveInput) -> syn::DeriveInput {
    item
}

#[manyhow(item_as_dummy)]
#[proc_macro_attribute]
pub fn parse_quote_dummy_attribute(_: syn::LitStr, item: syn::DeriveInput) -> syn::DeriveInput {
    item
}

#[manyhow(item_as_dummy)]
#[proc_macro_attribute]
pub fn parse_quote_dummy_error_attribute(_: TokenStream, _: TokenStream) -> Result<syn::Ident> {
    bail!("error message")
}

#[manyhow(item_as_dummy)]
#[proc_macro_attribute]
pub fn parse_quote_dummy_error_attribute_syn_result(
    _: TokenStream,
    _: TokenStream,
) -> syn::Result<syn::Ident> {
    bail!("error message")
}

#[manyhow]
#[proc_macro_derive(ParseQuote)]
pub fn parse_quote_derive(item: syn::ItemStruct) -> syn::ItemStruct {
    item
}

#[manyhow]
#[proc_macro_derive(ParseQuoteSynResult)]
pub fn parse_quote_derive_syn_result(item: syn::ItemStruct) -> syn::Result<syn::ItemStruct> {
    Ok(item)
}
