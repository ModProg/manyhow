use std::fmt::{Display, Write};

use proc_macro2::{Span, TokenStream, TokenTree};
use proc_macro_utils::{Delimited, TokenStream2Ext, TokenStreamExt};
use quote::{quote, quote_spanned, ToTokens};

#[derive(PartialEq, Eq, Clone, Copy)]
enum ProcMacroType {
    Function,
    Derive,
    Attribute,
}
impl ProcMacroType {
    fn to_signature(self, tokens: &mut TokenStream) {
        match self {
            ProcMacroType::Function | ProcMacroType::Derive => quote! {
                (__input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream
            },
            ProcMacroType::Attribute => quote! {
                (__input: ::proc_macro::TokenStream, __item: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream
            },
        }
        .to_tokens(tokens)
    }

    fn dummy_flag(self) -> &'static str {
        match self {
            ProcMacroType::Function => "input_as_dummy",
            ProcMacroType::Derive => "",
            ProcMacroType::Attribute => "item_as_dummy",
        }
    }
}
impl ToTokens for ProcMacroType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fn_name = match self {
            ProcMacroType::Function => quote!(function),
            ProcMacroType::Derive => quote!(derive),
            ProcMacroType::Attribute => quote!(attribute),
        };
        let item = if *self == ProcMacroType::Attribute {
            quote!(, __item)
        } else {
            quote!()
        };
        let as_dummy = if matches!(self, ProcMacroType::Attribute | ProcMacroType::Function) {
            quote!(, __as_dummy)
        } else {
            quote!()
        };
        quote! {
            ::manyhow::#fn_name(__input #item #as_dummy, __implementation)
        }
        .to_tokens(tokens)
    }
}

/// Attribute macro to remove boiler plate from proc macro entry points.
///
/// See [the documentation at the crate root for more
/// details](https://docs.rs/manyhow#using-the-manyhow-macro).
#[proc_macro_attribute]
pub fn manyhow(
    input: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut parser = item.clone().parser();
    let mut output = TokenStream::default();

    // For now, we will keep all attributes on the outer function
    let mut typ = None;
    while let Some(pound) = parser.next_pound() {
        output.extend(pound);
        let attribute_content = parser
            .next_bracketed()
            .expect("rust should only allow valid attributes");
        output.extend(quote!( [#attribute_content] ));
        let ident = attribute_content
            .parser()
            .next_ident()
            .expect("rust should only allow valid attributes");
        match ident.to_string().as_str() {
            "proc_macro" => {
                typ = Some(ProcMacroType::Function);
            }
            "proc_macro_attribute" => {
                typ = Some(ProcMacroType::Attribute);
            }
            "proc_macro_derive" => {
                typ = Some(ProcMacroType::Derive);
            }
            _ => {}
        }
    }
    let Some(typ) = typ else {
        return with_helpful_error(item, Span::call_site(), "expected proc_macro* attribute below `#[manyhow]`", "try adding `#[proc_macro]`, `#[proc_macro_attribute]` or `#[proc_macro_derive]` below `#[manyhow]`");
    };

    let mut as_dummy = false;
    let mut input = input.parser();
    match input.next_ident() {
        Some(ident) => match (ident.to_string().as_str(), typ) {
            ("item_as_dummy", ProcMacroType::Attribute) => {
                as_dummy = true;
            }
            ("item_as_dummy", ProcMacroType::Function) => {
                return with_helpful_error(
                    item,
                    ident.span(),
                    format_args!(
                        "`item_as_dummy` is only supported with `#[proc_macro_attribute]`"
                    ),
                    format_args!("try `#[manyhow(input_as_dummy)]` instead"),
                );
            }
            ("input_as_dummy", ProcMacroType::Function) => {
                as_dummy = true;
            }
            ("input_as_dummy", ProcMacroType::Attribute) => {
                return with_helpful_error(
                    item,
                    ident.span(),
                    format_args!("`input_as_dummy` is only supported with `#[proc_macro]`"),
                    "try `#[manyhow(item_as_dummy)]` instead",
                );
            }
            ("input_as_dummy" | "item_as_dummy", ProcMacroType::Derive) => {
                return with_helpful_error(
                    item,
                    ident.span(),
                    format_args!(
                        "only `#[proc_macro]` and `#[proc_macro_attribute]` support `*_as_dummy` \
                         flags"
                    ),
                    "try `#[manyhow]` instead",
                );
            }
            _ => {
                return with_error(
                    item,
                    ident.span(),
                    format_args!("only `{}` is supported", typ.dummy_flag(),),
                );
            }
        },
        None if !input.is_empty() => {
            return with_helpful_error(
                item,
                input.next().unwrap().span(),
                "manyhow expects a comma seperated list of flags",
                format_args!("try `#[manyhow({})]`", typ.dummy_flag()),
            );
        }
        None => {}
    }
    // All attributes are parsed now there should only be a public function

    // vis
    output.extend(parser.next_if(|tt| matches!(tt, TokenTree::Ident(ident) if ident == "pub")));
    // fn
    output.push(match parser.next() {
        Some(TokenTree::Ident(ident)) if ident == "fn" => ident.into(),
        token => {
            return with_error(
                item,
                token.as_ref().map_or_else(Span::call_site, TokenTree::span),
                "expected function",
            );
        }
    });
    // function name
    output.push(parser.next().expect("function name"));
    // there should not be any generics
    match parser.next_lt() {
        None => {}
        Some(lt) => {
            return with_error(
                item,
                lt.into_iter().next().unwrap().span(),
                "proc macros cannot have generics",
            );
        }
    }
    typ.to_signature(&mut output);
    // (...)
    let params = parser.next_group().expect("params");
    // ->
    let Some(arrow) = parser.next_r_arrow() else {
        return with_helpful_error(item, params.span_close(), "expected return type", "try adding either `-> TokenStream` or `-> manyhow::Result`");
    };
    // return type
    let ret_ty = parser
        .next_until(|tt| tt.is_braced())
        .expect("return type after ->");
    // {...}
    let body = parser.next_group().expect("body");
    assert!(parser.is_empty(), "no tokens after function body");

    quote! {
        {
            fn __implementation #params #arrow #ret_ty #body
            let __as_dummy = #as_dummy;
            #typ
        }
    }
    .to_tokens(&mut output);

    output.into()
}

fn with_error(
    item: proc_macro::TokenStream,
    span: Span,
    error: impl Display,
) -> proc_macro::TokenStream {
    let mut item = item.into();
    self::error(span, error).to_tokens(&mut item);
    item.into()
}

fn with_helpful_error(
    item: proc_macro::TokenStream,
    span: Span,
    error: impl Display,
    help: impl Display,
) -> proc_macro::TokenStream {
    let mut item = item.into();
    self::error_help(span, error, help).to_tokens(&mut item);
    item.into()
}

fn error(span: Span, error: impl Display) -> TokenStream {
    let error = error.to_string();
    quote_spanned! {span=>
        ::core::compile_error!{ #error }
    }
}

fn error_help(span: Span, error: impl Display, help: impl Display) -> TokenStream {
    let mut error = error.to_string();
    write!(error, "\n\n  = help: {help}").unwrap();
    quote_spanned! {span=>
        ::core::compile_error!{ #error }
    }
}
