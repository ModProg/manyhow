use manyhow::{attribute, bail, derive, function, Emitter, ErrorMessage, Result, SilentError};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;

type SilentResult = Result<TokenStream2, SilentError>;

#[proc_macro_attribute]
pub fn attr_item_as_dummy(input: TokenStream, item: TokenStream) -> TokenStream {
    attribute(
        input,
        item,
        true,
        |_: TokenStream2, _: TokenStream2| -> SilentResult { Err(SilentError) },
    )
}

#[proc_macro_attribute]
pub fn attr_no_dummy(input: TokenStream, item: TokenStream) -> TokenStream {
    attribute(
        input,
        item,
        false,
        |_: TokenStream2, _: TokenStream2| -> SilentResult { Err(SilentError) },
    )
}

#[proc_macro_attribute]
pub fn attr_custom_dummy(input: TokenStream, item: TokenStream) -> TokenStream {
    attribute(
        input,
        item,
        false,
        |_: TokenStream2, _: TokenStream2, dummy: &mut TokenStream2| -> SilentResult {
            *dummy = quote! {fn dummy(){}};
            Err(SilentError)
        },
    )
}

#[proc_macro_attribute]
pub fn attr_emit(input: TokenStream, item: TokenStream) -> TokenStream {
    attribute(
        input,
        item,
        false,
        |_: TokenStream2, _: TokenStream2, emitter: &mut Emitter| -> TokenStream2 {
            emitter.emit(ErrorMessage::new(Span::call_site(), "example error"));
            quote! {fn output(){}}
        },
    )
}

#[proc_macro]
pub fn input_as_dummy(input: TokenStream) -> TokenStream {
    function(input, true, |_: TokenStream2| -> SilentResult {
        Err(SilentError)
    })
}

#[proc_macro]
pub fn no_dummy(input: TokenStream) -> TokenStream {
    function(input, false, |_: TokenStream2| -> SilentResult {
        Err(SilentError)
    })
}

#[proc_macro]
pub fn custom_dummy(input: TokenStream) -> TokenStream {
    function(
        input,
        false,
        |_: TokenStream2, dummy: &mut TokenStream2| -> SilentResult {
            *dummy = quote! {fn dummy(){}};
            Err(SilentError)
        },
    )
}

#[proc_macro]
pub fn emit(input: TokenStream) -> TokenStream {
    function(
        input,
        false,
        |_: TokenStream2, emitter: &mut Emitter| -> TokenStream2 {
            emitter.emit(ErrorMessage::new(Span::call_site(), "example error"));
            quote! {fn output(){}}
        },
    )
}

#[proc_macro]
pub fn no_closure(input: TokenStream) -> TokenStream {
    function(input, true, no_closure_impl)
}

fn no_closure_impl(_: TokenStream2) -> SilentResult {
    Err(SilentError)
}

#[proc_macro_derive(NoDummy)]
pub fn derive_no_dummy(item: TokenStream) -> TokenStream {
    derive(item, |_: TokenStream2| -> SilentResult { Err(SilentError) })
}

#[proc_macro_derive(Dummy)]
pub fn derive_dummy(item: TokenStream) -> TokenStream {
    derive(
        item,
        |_: TokenStream2, dummy: &mut TokenStream2| -> SilentResult {
            *dummy = quote! {fn dummy(){}};
            Err(SilentError)
        },
    )
}

#[proc_macro_derive(Emit)]
pub fn derive_emit(item: TokenStream) -> TokenStream {
    derive(
        item,
        |_: TokenStream2, emitter: &mut Emitter| -> TokenStream2 {
            emitter.emit(ErrorMessage::new(Span::call_site(), "example error"));
            quote! {fn output(){}}
        },
    )
}

#[proc_macro]
pub fn parse_quote(input: TokenStream) -> TokenStream {
    function!(input, |lit: syn::LitStr| lit)
}

#[proc_macro]
pub fn parse_quote_dummy(input: TokenStream) -> TokenStream {
    function!(
        #[as_dummy]
        input,
        |input: syn::DeriveInput| input
    )
}

#[proc_macro]
pub fn parse_quote_dummy_error(input: TokenStream) -> TokenStream {
    function!(
        #[as_dummy]
        input,
        |_: TokenStream| -> Result<syn::Ident> { bail!("error message") }
    )
}

#[proc_macro]
pub fn parse_quote_dummy_error_syn_result(input: TokenStream) -> TokenStream {
    function!(
        #[as_dummy]
        input,
        |_: TokenStream| -> syn::Result<syn::Ident> { bail!("error message") }
    )
}

#[proc_macro_attribute]
pub fn parse_quote_attribute(input: TokenStream, item: TokenStream) -> TokenStream {
    attribute!(input, item, |_: syn::LitStr, item: syn::DeriveInput| item)
}

#[proc_macro_attribute]
pub fn parse_quote_dummy_attribute(input: TokenStream, item: TokenStream) -> TokenStream {
    attribute!(
        input,
        #[as_dummy]
        item,
        |_: TokenStream, item: syn::DeriveInput| item
    )
}

#[proc_macro_attribute]
pub fn parse_quote_dummy_error_attribute(input: TokenStream, item: TokenStream) -> TokenStream {
    attribute!(
        input,
        #[as_dummy]
        item,
        |_: TokenStream, _: TokenStream| -> Result<syn::Ident> { bail!("error message") }
    )
}

#[proc_macro_attribute]
pub fn parse_quote_dummy_error_attribute_syn_result(input: TokenStream, item: TokenStream) -> TokenStream {
    attribute!(
        input,
        #[as_dummy]
        item,
        |_: TokenStream, _: TokenStream| -> syn::Result<syn::Ident> { bail!("error message") }
    )
}

#[proc_macro_derive(ParseQuote)]
pub fn parse_quote_derive(item: TokenStream) -> TokenStream {
    derive!(item, |item: syn::ItemStruct| item)
}

#[proc_macro_derive(ParseQuoteSynResult)]
pub fn parse_quote_derive_syn_result(item: TokenStream) -> TokenStream {
    derive!(
        item,
        |item: syn::ItemStruct| -> syn::Result<syn::ItemStruct> { Ok(item) }
    )
}
