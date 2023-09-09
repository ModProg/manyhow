use std::fmt::{Display, Write};

use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use proc_macro_utils::{Delimited, TokenStream2Ext, TokenStreamExt};
use quote::{format_ident, quote, quote_spanned, ToTokens};

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

enum Param {
    Flag(Ident),
    Complex(Ident, Group),
}

impl Param {
    fn span(&self) -> Span {
        self.ident().span()
    }

    fn ident(&self) -> &Ident {
        let (Param::Flag(ident) | Param::Complex(ident, _)) = self;
        ident
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Param::Flag(ident) => ident.fmt(f),
            Param::Complex(ident, tokens) => ident.fmt(f).and(tokens.fmt(f)),
        }
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
    let mut kind = None;
    let mut kind_attribute = None;
    let mut set_kind = |ident: &Ident, create_attribute: bool| {
        let new_kind = match ident.to_string().as_str() {
            "proc_macro" => ProcMacroType::Function,
            "proc_macro_attribute" => ProcMacroType::Attribute,
            "proc_macro_derive" => ProcMacroType::Derive,
            _ => return Ok(()),
        };
        if let Some((_, span)) = kind {
            Err(with_helpful_error(
                with_helpful_error(
                    item.clone(),
                    span,
                    "proc_macro kind specified multiple times",
                    "try removing this",
                ),
                ident.span(),
                "proc_macro kind specified multiple times",
                "try removing this",
            ))
        } else {
            kind = Some((new_kind, ident.span()));
            if create_attribute {
                kind_attribute = Some(quote!(#[#ident]));
            }
            Ok(())
        }
    };
    while let Some(pound) = parser.next_tt_pound() {
        output.extend(pound);
        let attribute_content = parser
            .next_bracketed()
            .expect("rust should only allow valid attributes");
        let ident = attribute_content
            .stream()
            .parser()
            .next_ident()
            .expect("rust should only allow valid attributes");
        output.push(attribute_content.into());
        if let Err(err) = set_kind(&ident, false) {
            return err;
        }
    }

    let mut flags = Vec::new();
    let mut input = input.parser();
    while !input.is_empty() {
        let Some(ident) = input.next_ident() else {
            return with_helpful_error(
                item,
                input.next().unwrap().span(),
                "manyhow expects a comma seperated list of flags",
                format_args!("try `#[manyhow(impl_fn)]`"),
            );
        };
        if ident == "proc_macro_derive" {
            let Some(group) = input.next_group() else {
                return with_helpful_error(
                    item,
                    input.next().unwrap_or(ident.into()).span(),
                    "`proc_macro_derive` expects `(TraitName)`",
                    format_args!("try `#[manyhow(proc_macro_derive(YourTraitName))]`"),
                );
            };
            // We set it manually here
            if let Err(error) = set_kind(&ident, false) {
                return error;
            }
            quote!(#[#ident #group]).to_tokens(&mut output);
            flags.push(Param::Complex(ident, group));
        } else {
            if let Err(error) = set_kind(&ident, true) {
                return error;
            }
            flags.push(Param::Flag(ident));
        }
        // This technically allows `flag flag flag` but it's fine IMO
        _ = input.next_tt_comma();
    }

    output.extend(kind_attribute);

    let Some((kind, _)) = kind else {
        return with_helpful_error(
            item,
            Span::call_site(),
            "expected proc_macro* attribute below `#[manyhow]` or a flag as parameter of the \
             attribute",
            "try adding `#[proc_macro]`, `#[proc_macro_attribute]`, or `#[proc_macro_derive]` \
             below `#[manyhow]` or adding a flag to `#[manyhow]`, i.e., `#[manyhow(proc_macro)]`, \
             `#[manyhow(proc_macro_attribute)]` or `#[manyhow(proc_macro_derive)]` ",
        );
    };

    let flags_replace = |i: usize, replacement: Option<&str>| {
        let mut flags = flags.iter().map(ToString::to_string).collect::<Vec<_>>();
        if let Some(replacement) = replacement {
            flags[i] = replacement.to_owned();
        } else {
            flags.remove(i);
        }
        if flags.is_empty() {
            "".to_owned()
        } else {
            format!("({})", flags.join(", "))
        }
    };

    let mut as_dummy = false;
    let mut impl_fn = false;
    for (i, param) in flags.iter().enumerate() {
        match (param.ident().to_string().as_str(), kind) {
            ("impl_fn", _) => impl_fn = true,
            ("item_as_dummy", ProcMacroType::Attribute) => as_dummy = true,
            ("item_as_dummy", ProcMacroType::Function) => {
                return with_helpful_error(
                    item,
                    param.span(),
                    format_args!(
                        "`item_as_dummy` is only supported with `#[proc_macro_attribute]`"
                    ),
                    format_args!(
                        "try `#[manyhow{}]` instead",
                        flags_replace(i, Some("input_as_dummy"))
                    ),
                );
            }
            ("input_as_dummy", ProcMacroType::Function) => as_dummy = true,
            ("input_as_dummy", ProcMacroType::Attribute) => {
                return with_helpful_error(
                    item,
                    param.span(),
                    format_args!("`input_as_dummy` is only supported with `#[proc_macro]`"),
                    format_args!(
                        "try `#[manyhow{}]` instead",
                        flags_replace(i, Some("item_as_dummy"))
                    ),
                );
            }
            ("input_as_dummy" | "item_as_dummy", ProcMacroType::Derive) => {
                return with_helpful_error(
                    item,
                    param.span(),
                    format_args!(
                        "only `#[proc_macro]` and `#[proc_macro_attribute]` support `*_as_dummy` \
                         flags"
                    ),
                    format_args!("try `#[manyhow{}]` instead", flags_replace(i, None)),
                );
            }
            ("proc_macro" | "proc_macro_attribute" | "proc_macro_derive", _) => {}
            _ => {
                return with_helpful_error(
                    item,
                    param.span(),
                    format_args!(
                        "only `proc_macro`, `proc_macro_attribute`, `proc_macro_derive`, `{}`, \
                         and `impl_fn` are supported",
                        kind.dummy_flag(),
                    ),
                    format_args!("try `#[manyhow{}]", flags_replace(i, None)),
                );
            }
        }
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
    let Some(fn_name) = parser.next_ident() else {
        return with_error(
            item,
            parser
                .next()
                .as_ref()
                .map_or_else(Span::call_site, TokenTree::span),
            "expected function name",
        );
    };
    let impl_fn = impl_fn.then(|| format_ident!("{fn_name}_impl"));
    // function name
    output.push(fn_name.into());
    // there should not be any generics
    match parser.next_tt_lt() {
        None => {}
        Some(lt) => {
            return with_error(
                item,
                lt.into_iter().next().unwrap().span(),
                "proc macros cannot have generics",
            );
        }
    }
    kind.to_signature(&mut output);
    // (...)
    let params = parser.next_group().expect("params");
    // ->
    let Some(arrow) = parser.next_tt_r_arrow() else {
        return with_helpful_error(
            item,
            params.span_close(),
            "expected return type",
            "try adding either `-> TokenStream` or `-> manyhow::Result`",
        );
    };
    // return type
    let ret_ty = parser
        .next_until(|tt| tt.is_braced())
        .expect("return type after ->");
    // {...}
    let body = parser.next_group().expect("body");
    assert!(parser.is_empty(), "no tokens after function body");

    let inner_impl_fn = if let Some(impl_fn) = &impl_fn {
        quote!(let __implementation = #impl_fn;)
    } else {
        quote!(fn __implementation #params #arrow #ret_ty #body)
    };

    quote! {
        {
            #inner_impl_fn
            let __as_dummy = #as_dummy;
            #kind
        }
    }
    .to_tokens(&mut output);

    if let Some(impl_fn) = impl_fn {
        quote!(fn #impl_fn #params #arrow #ret_ty #body).to_tokens(&mut output);
    }
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
