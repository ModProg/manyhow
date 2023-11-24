#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions)]
//! Proc **m**acro **anyhow**, a combination of ideas from
//! [`anyhow`](docs.rs/anyhow) and
//! [`proc-macro-error`](docs.rs/proc-macro-error) to improve proc macro
//! development, especially focused on the error handling.
//!
//! # Motivation
//! Error handling in proc-macros is unideal, as the top level functions of
//! proc-macros can only return `TokenStreams` both in success and failure case.
//! This means that I often write code like this, moving the actual
//! implementation in a separate function to be able to use the ergonomic rust
//! error handling with e.g., `?`.
//! ```
//! # use proc_macro2::TokenStream;
//! # use quote::quote;
//! # use syn2 as syn;
//! use proc_macro2::TokenStream as TokenStream2;
//!
//! # let _ = quote!{
//! #[proc_macro]
//! # };
//! pub fn my_macro(input: TokenStream) -> TokenStream {
//!     match actual_implementation(input.into()) {
//!         Ok(output) => output,
//!         Err(error) => error.into_compile_error(),
//!     }
//!     .into()
//! }
//!
//! fn actual_implementation(input: TokenStream2) -> syn::Result<TokenStream2> {
//!     // ..
//! #   Ok(quote!())
//! }
//! ```
//!
//! # Using the `#[manyhow]` macro
//! To activate the error hadling, just add [`#[manyhow]`](manyhow) above any
//! proc-macro implementation, reducing the above example to:
//!
//! ```
//! # use quote::quote;
//! # use syn2 as syn;
//! use manyhow::manyhow;
//! use proc_macro2::TokenStream as TokenStream2;
//!
//! # let _ = quote!{
//! #[manyhow]
//! #[proc_macro]
//! # };
//! // You can also merge the two attributes: #[manyhow(proc_macro)]
//! fn my_macro(input: TokenStream2) -> syn::Result<TokenStream2> {
//!     // ..
//! #   Ok(quote!())
//! }
//!
//! // On top of the TokenStreams any type that implements `syn::Parse` is supported
//! # let _ = quote!{
//! #[manyhow(proc_macro_derive(MyMacro))]
//! #[proc_macro]
//! # };
//! // The output can also be anything that implements `quote::ToTokens`
//! fn my_derive_macro(input: syn::DeriveInput) -> manyhow::Result<syn::ItemImpl> {
//!     // ..
//! #   manyhow::bail!("hello")
//! }
//! ```
//!
//! See [Without macros](#without-macros) to see what this expands to under the
//! hood.
//!
//! You can also use the `#[manyhow]` attrubutes on a use statement, useful when
//! moving your proc-macro implementations in seperate modules.
//!
//! ```
//! # use quote::quote;
//! use manyhow::manyhow;
//!
//! mod module {
//!     # use quote::quote;
//!     # use syn2 as syn;
//!     use proc_macro2::TokenStream as TokenStream2;
//!
//!     pub fn my_macro(input: TokenStream2) -> syn::Result<TokenStream2> {
//!         // ..
//!     #   Ok(quote!())
//!     }
//! }
//!
//! # let _ = quote!{
//! #[manyhow]
//! #[proc_macro]
//! # };
//! // You can also merge the two attributes: #[manyhow(proc_macro)]
//! pub use module::my_macro;
//! ```
//!
//! A proc macro function marked as `#[manyhow]` can take and return any
//! [`TokenStream`](AnyTokenStream), and can also return `Result<TokenStream,
//! E>` where `E` implments [`ToTokensError`]. As additional parameters a
//! [dummy](#dummy-mut-tokenstream) and/or [emitter](#emitter-mut-emitter) can
//! be specified.
//!
//! The `manyhow` attribute takes optional flags to configure its behavior.
//!
//! When used for `proc_macro` and `proc_macro_attribute`,
//! `#[manyhow(input_as_dummy, ...)]` will take the input of a function like
//! `proc_macro` to initialize the [dummy `&mut TokenStream`](#
//! dummy-mut-tokenstream) while `#[manyhow(item_as_dummy, ...)]` on
//! `proc_macro_attribute` will initialize the dummy with the annotated item.
//!
//! You can merge the `#[proc_macro*]` attribute inside the manyhow flags e.g.,
//! `#[manyhow(proc_macro)]` or `#[manyhow(proc_macro_derive(SomeTrait, ...))]`.
//!
//! The `#[manyhow(impl_fn, ...)]` flag will put the actual macro implementation
//! in a separate function. Making it available for e.g., unit testing with
//! [`proc_macro_utils::assert_expansion!`](https://docs.rs/proc-macro-utils/latest/proc_macro_utils/macro.assert_expansion.html).
//!
//! ```ignore
//! #[manyhow(impl_fn)]
//! #[proc_macro]
//! pub fn actual_macro(input: TokenStream2) -> TokenStream2 {
//!     // ...
//! }
//! // would roughly expand to
//! #[proc_macro]
//! pub fn actual_macro(input: TokenStream) -> TokenStream {
//!     actual_macro_impl(input.into()).into()
//! }
//! fn actual_macro_impl(input: TokenStream2) -> TokenStream2 {
//!     // ...
//! }
//! ```
//!
//! # Without macros
//! `manyhow` can be used without proc macros, and they can be disabled by
//! adding `manyhow` with `default-features=false`.
//!
//! The usage is more or less the same, though with some added boilerplate from
//! needing to invoke one of [`function()`] ([`function!`]), [`attribute()`]
//! ([`attribute!`]) or [`derive()`] ([`derive!`]) directly. For each version
//! there exists a function and a `macro_rules` macro, while the function only
//! supports [`proc_macro::TokenStream`] and [`proc_macro2::TokenStream`], the
//! macro versions also support any type that implements [`Parse`]
//! and [`ToTokens`] respectively.
//!
//! While the examples use closures, functions can be passed in as well. The
//! above example would then change to:
//! ```
//! # use proc_macro2::TokenStream;
//! # use quote::quote;
//! # use syn2 as syn;
//! use proc_macro2::TokenStream as TokenStream2;
//!
//! # let _ = quote!{
//! #[proc_macro]
//! # };
//! pub fn my_macro(input: TokenStream) -> TokenStream {
//! # let tmp = input.clone();
//! # let output: TokenStream =
//!     manyhow::function(
//!         input,
//!         false,
//!         |input: TokenStream2| -> syn::Result<TokenStream2> {
//!             // ..
//! #           Ok(quote!())
//!         },
//!     )
//! # ;
//! # let input = tmp;
//!     // Or
//!     manyhow::function!(
//!         input,
//!         |input: syn::DeriveInput| -> manyhow::Result<syn::ItemImpl> {
//!             // ..
//! #           manyhow::bail!("error")
//!         },
//!     )
//! }
//! ```
//! [`Emitter`](#emitter-mut-emitter) and [dummy
//! `TokenStream`](#dummy-mut-tokenstream) can also be used. [`function()`]
//! ([`function!`]) and [`attribute()`] ([`attribute!`]) take an additional
//! boolean parameter controlling whether the input/item will be used as initial
//! dummy.
//!
//! # `emitter: &mut Emitter`
//! [`MacroHandler`]s (the trait defining what closures/functions can be used
//! with `manyhow`) can take a mutable reference to an [`Emitter`]. This
//! allows collecting errors, but not fail immediately.
//!
//! [`Emitter::into_result`] can be used to return if an [`Emitter`] contains
//! any values.
//!
//! ```
//! # use quote::quote;
//! # use syn2 as syn;
//! use manyhow::{manyhow, Emitter, ErrorMessage};
//! use proc_macro2::TokenStream as TokenStream2;
//!
//! # let _ = quote!{
//! #[manyhow]
//! #[proc_macro]
//! # };
//! fn my_macro(input: TokenStream2, emitter: &mut Emitter) -> manyhow::Result<TokenStream2> {
//!     // ..
//!     emitter.emit(ErrorMessage::call_site("A fun error!"));
//!     emitter.into_result()?;
//!     // ..
//! #   Ok(quote!())
//! }
//! ```
//!
//! # `dummy: &mut TokenStream`
//! [`MacroHandler`]s also take a mutable reference to a `TokenStream`, to
//! enable emitting some dummy code to be used in case the macro errors.
//!
//! This allows either appending tokens e.g., with [`ToTokens::to_tokens`] or
//! directly setting the dummy code e.g., `*dummy = quote!{some tokens}`.
//!
//! # Crate features
//!
//! - `syn`/`syn2` **default** Enables errors for [`syn` 2.x](https://docs.rs/syn/latest/syn/).
//! - `syn1` Enables errors for [`syn` 1.x](https://docs.rs/syn/1.0.109/syn/index.html).
//! - `darling` Enables erros for [`darling`](https://docs.rs/darling/latest/index.html).

use std::convert::Infallible;

#[cfg(feature = "macros")]
pub use macros::manyhow;
use proc_macro2::TokenStream;
#[cfg(doc)]
use {quote::ToTokens, syn2::parse::Parse};

extern crate proc_macro;

#[macro_use]
mod span_ranged;
pub use span_ranged::{to_tokens_span_range, SpanRanged};
#[macro_use]
mod macro_rules;
mod error;
pub use error::*;

mod parse_to_tokens;

#[doc(hidden)]
pub mod __private {
    pub use std::prelude::rust_2021::*;

    use proc_macro2::TokenStream;
    pub use quote;

    pub use crate::span_ranged::*;
    pub type Dummy = Option<TokenStream>;

    pub use crate::parse_to_tokens::*;
}

/// Marker trait for [`proc_macro::TokenStream`] and
/// [`proc_macro2::TokenStream`]
pub trait AnyTokenStream: Clone + From<TokenStream> + Into<TokenStream> + Default {}
impl AnyTokenStream for TokenStream {}
impl AnyTokenStream for proc_macro::TokenStream {}

macro_rules! handler {
    ($(#$doc:tt)*$name:ident; $($input:ident: $Input:ident),*; $($dummy:ident, $dummy_value:ident)?) => {
        $(#$doc)*
        pub fn $name<
            $($Input: AnyTokenStream,)*
            Output: AnyTokenStream,
            Return: AnyTokenStream,
            Error: ToTokensError,
            Function,
        >(
            $($input: impl AnyTokenStream,)*
            $($dummy: bool,)?
            body: impl MacroHandler<($($Input,)*), Output, Output, Function, Error>,
        ) -> Return {
            #[allow(unused_mut)]
            let mut tokens = Output::default();
            $(let mut tokens = if $dummy {
                $dummy_value.clone().into().into()
            } else {
                tokens
            };)?
            let mut emitter = Emitter::new();
            let output = body.call(($($input.into().into(),)*), &mut tokens, &mut emitter);
            let mut tokens = tokens.into();
            let mut tokens = match output {
                Ok(tokens) => tokens.into(),
                Err(error) => {
                    error.to_tokens(&mut tokens);
                    tokens
                }
            };
            emitter.to_tokens(&mut tokens);
            tokens.into()
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __macro_handler {
    ($name:ident; $($(#attr=$attr:tt)? $n:ident: $input:expr),+; $impl:expr$(; dummy:$dummy:expr)?) => {
        $crate::__macro_handler! {! $name; $($(#attr=$attr)? $n: $input.clone()),+; $impl $(; $crate::__private::Some($dummy))?}
    };
    ($name:ident; $($(#attr=$attr:tt)? $n:ident: $input:expr),+; $impl:expr; dummy) => {
        $crate::__macro_handler! {! $name; $($(#attr=$attr)? $n: $input),+; $impl; $crate::__private::Dummy::None}
    };
    (! $name:ident; $($(#attr=$attr:tt)? $n:ident: $input:expr),+; $impl:expr $(; $dummy:expr)?) => {{
        use $crate::__private::{ManyhowParse, ManyhowToTokens};
        let implementation = $impl;
        $(let $n = &$crate::__private::WhatType::new();)+
        if false {
            _ = $crate::__private::$name($($n.identify(),)+ $($dummy,)? implementation);
            unreachable!();
        } else {
            match $crate::__private::$name($(
                {#[allow(unused)]
                let attr = false;
                $(let attr = $attr;)?
                $n.manyhow_parse($input, attr)},
            )+ $($dummy,)? implementation)
            {
                Err(tokens) => tokens.into(),
                Ok((output, tokens)) => (&$crate::__private::WhatType::from(&output))
                    .manyhow_into_token_stream(output, tokens)
                    .into(),
            }
        }
    }};
}

handler! {
/// Handles [`proc_macro_attribute`](https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros)
/// implementation
///
/// Takes any `TokenStream` for `input` and `item` and returns any
/// `TokenStream`. If `item_as_dummy = true` the item input will be used as
/// default dummy code on error. `body` takes a [`MacroHandler`] with two
/// `TokenStream` parameters. And an optional [`&mut Emitter`](Emitter) and a
/// `&mut TokenStream` for storing a dummy output.
///
/// ```
/// # use proc_macro_utils::assert_tokens;
/// # use quote::{quote, ToTokens};
/// use manyhow::{attribute, Emitter, Result};
/// use proc_macro2::TokenStream;
/// # let input = quote!();
/// # let item = quote!();
/// # let output: TokenStream =
/// attribute(
///     input,
///     item,
///     false,
///     |input: TokenStream,
///      item: TokenStream,
///      dummy: &mut TokenStream,
///      emitter: &mut Emitter|
///      -> Result {
///         // ..
///         # Ok(quote!())
///     },
/// );
/// ```
///
/// *Note:* When `item_as_dummy = true` the `dummy: &mut TokenStream` will be
/// initialized with `item`. To override assign a new `TokenStream`:
/// ```
/// # use proc_macro_utils::assert_tokens;
/// use manyhow::{attribute, Result, SilentError};
/// use proc_macro2::TokenStream;
/// use quote::{quote, ToTokens};
/// # let input = quote!(input);
/// let item = quote!(
///     struct Struct;
/// );
/// let output: TokenStream = attribute(
///     input,
///     item,
///     true,
///     |input: TokenStream,
///      item: TokenStream,
///      dummy: &mut TokenStream|
///      -> Result<TokenStream, SilentError> {
///         assert_tokens!(dummy.to_token_stream(), {
///             struct Struct;
///         });
///         *dummy = quote! {
///             struct Struct(HelloWorld);
///         };
///         // ..
///         Err(SilentError)
///     },
/// );
///
/// assert_tokens! {output, {struct Struct(HelloWorld);}};
/// ```
attribute; input: Input, item: Item; item_as_dummy, item
}

/// Handles [`proc_macro_attribute`](https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros)
/// implementation
///
/// Takes any `TokenStream` for `input` and `item` and its return value. If
/// `#[as_dummy]` is specified on item, it will be used as default
/// dummy code on error. `body` takes a [`MacroHandler`] with two `TokenStream`
/// or type implementing [`Parse`] parameters and returning a `TokenStream` or
/// type implementing [`ToTokens`]. And an optional [`&mut Emitter`](Emitter)
/// and a `&mut TokenStream` for storing a dummy output.
///
///
/// ```
/// # use proc_macro_utils::assert_tokens;
/// # use quote::{quote, ToTokens};
/// use manyhow::{attribute, Emitter, Result};
/// use proc_macro2::TokenStream;
/// # let input = quote!();
/// # let item = quote!();
/// # let output: TokenStream =
/// attribute!(input, item, |input: TokenStream,
///                          item: TokenStream,
///                          dummy: &mut TokenStream,
///                          emitter: &mut Emitter|
///  -> Result {
///     // ..
///         # Ok(quote!())
/// });
/// ```
///
/// *Note:* When `#[as_dummy]` is specified the `dummy: &mut TokenStream` will
/// be initialized with `item`. To override assign a new `TokenStream`:
/// ```
/// # use proc_macro_utils::assert_tokens;
/// # use syn2 as syn;
/// use manyhow::{attribute, Result, SilentError};
/// use proc_macro2::TokenStream;
/// use quote::{quote, ToTokens};
/// # let input = quote!(input);
/// let item = quote!(
///     struct Struct;
/// );
/// let output: TokenStream = attribute!(
///     input,
///     #[as_dummy]
///     item,
///     |input: TokenStream,
///      item: syn::ItemStruct,
///      dummy: &mut TokenStream|
///      -> Result<syn::ItemStruct, SilentError> {
///         assert_tokens!(dummy.to_token_stream(), {
///             struct Struct;
///         });
///         *dummy = quote! {
///             struct Struct(HelloWorld);
///         };
///         // ..
///         Err(SilentError)
///     },
/// );
///
/// assert_tokens! {output, {struct Struct(HelloWorld);}};
/// ```
#[macro_export]
macro_rules! attribute {
    ($input:expr, #[as_dummy] $item:expr, $impl:expr $(,)?) => {
        $crate::__macro_handler!{attribute_transparent; #attr=true input: $input, item: $item.clone(); $impl; dummy: $item}
    };
    ($input:expr, $item:expr, $impl:expr $(,)?) => {
        $crate::__macro_handler!{attribute_transparent; #attr=true input: $input, item: $item; $impl; dummy}
    };
}

/// Handles [`proc_macro_derive`](https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros)
/// implementation.
///
/// Use [`derive!`] to support [`Parse`] and [`ToTokens`] as well.
///
/// Takes any `TokenStream` for `item` and returns any `TokenStream`. `body`
/// takes a [`MacroHandler`] with one `TokenStream` parameter. And an optional
/// [`&mut Emitter`](Emitter) and `&mut TokenStream` for storing a dummy
/// output.
///
/// ```
/// # use proc_macro_utils::assert_tokens;
/// # use quote::{quote, ToTokens};
/// use manyhow::{derive, Emitter, Result};
/// use proc_macro2::TokenStream;
/// # let item = quote!();
/// # let output: TokenStream =
/// derive(
///     item,
///     |item: TokenStream, dummy: &mut TokenStream, emitter: &mut Emitter| -> Result {
///         // ..
///         # Ok(quote!())
///     },
/// );
/// ```
pub fn derive<
    Item: AnyTokenStream,
    Output: AnyTokenStream,
    Return: AnyTokenStream,
    Error: ToTokensError,
    Function,
>(
    item: impl AnyTokenStream,
    body: impl MacroHandler<(Item,), Output, Output, Function, Error>,
) -> Return {
    let mut tokens = TokenStream::default().into();
    let mut emitter = Emitter::new();
    let output = body.call((item.into().into(),), &mut tokens, &mut emitter);
    let mut tokens = tokens.into();
    let mut tokens = match output {
        Ok(tokens) => tokens.into(),
        Err(error) => {
            error.to_tokens(&mut tokens);
            tokens
        }
    };
    emitter.to_tokens(&mut tokens);
    tokens.into()
}

/// Handles [`proc_macro_derive`](https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros)
/// implementation.
///
/// Takes any `TokenStream` for `item` and returns any `TokenStream`. `body`
/// takes a [`MacroHandler`] with one `TokenStream` or type implementing
/// [`Parse`] parameter and returns a `TokenStream` or type implementing
/// [`ToTokens`]. And an optional [`&mut Emitter`](Emitter) and `&mut
/// TokenStream` for storing a dummy output.
///
/// ```
/// # use proc_macro_utils::assert_tokens;
/// # use quote::{quote, ToTokens};
/// # use syn2 as syn;
/// use manyhow::{derive, Emitter, Result};
/// use proc_macro2::TokenStream;
/// # let item = quote!();
/// # let output: TokenStream =
/// derive!(item, |item: syn::DeriveInput,
///                dummy: &mut TokenStream,
///                emitter: &mut Emitter|
///  -> Result {
///     // ..
///         # Ok(quote!())
/// });
/// ```
#[macro_export]
macro_rules! derive {
    ($item:expr, $impl:expr $(,)?) => {
        $crate::__macro_handler! {derive_transparent; item: $item; $impl}
    };
}

/// Handles function like [`proc_macro`](https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros)
/// implementation
///
/// Use [`function!`] to support [`Parse`] and [`ToTokens`] as well.
///
/// Takes any `TokenStream` for `input` and returns any
/// `TokenStream`. If `input_as_dummy = true` the item input will be used as
/// default dummy code on error. `body` takes a [`MacroHandler`] with one
/// `TokenStream` parameter. And an optional [`&mut Emitter`](Emitter) and a
/// `&mut TokenStream` for storing a dummy output.
///
/// ```
/// # use proc_macro_utils::assert_tokens;
/// # use quote::{quote, ToTokens};
/// use manyhow::{function, Emitter, Result};
/// use proc_macro2::TokenStream;
/// # let input = quote!();
/// # let output: TokenStream =
/// function(
///     input,
///     false,
///     |input: TokenStream, dummy: &mut TokenStream, emitter: &mut Emitter| -> Result {
///         // ..
///         # Ok(quote!())
///     },
/// );
/// ```
///
/// *Note:* When `input_as_dummy = true` the `dummy: &mut TokenStream` will be
/// initialized with `input`. To override assign a new `TokenStream`:
/// ```
/// # use proc_macro_utils::assert_tokens;
/// use manyhow::{function, Result, SilentError};
/// use proc_macro2::TokenStream;
/// use quote::{quote, ToTokens};
/// let input = quote!(some input);
/// let output: TokenStream = function(
///     input,
///     true,
///     |input: TokenStream,
///      dummy: &mut TokenStream|
///      -> Result<TokenStream, SilentError> {
///         assert_tokens!(dummy.to_token_stream(), {
///             some input
///         });
///         *dummy = quote! {
///             another input
///         };
///         // ..
///         Err(SilentError)
///     },
/// );
///
/// assert_tokens! {output, {another input}};
/// ```
pub fn function<
    Input: AnyTokenStream,
    Output: AnyTokenStream,
    Return: AnyTokenStream,
    Error: ToTokensError,
    Function,
>(
    input: impl AnyTokenStream,
    input_as_dummy: bool,
    body: impl MacroHandler<(Input,), Output, Output, Function, Error>,
) -> Return {
    let mut tokens: Output = if input_as_dummy {
        input.clone().into().into()
    } else {
        TokenStream::default().into()
    };
    let mut emitter = Emitter::new();
    let output = body.call((input.into().into(),), &mut tokens, &mut emitter);
    let mut tokens = tokens.into();
    let mut tokens = match output {
        Ok(tokens) => tokens.into(),
        Err(error) => {
            error.to_tokens(&mut tokens);
            tokens
        }
    };
    emitter.to_tokens(&mut tokens);
    tokens.into()
}

/// Handles function like [`proc_macro`](https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros)
/// implementation
///
/// Takes any `TokenStream` for `input` and returns any `TokenStream`. If
/// `#[as_dummy]` is specified on input, it will be used as default
/// dummy code on error. `body` takes a [`MacroHandler`] with one `TokenStream`
/// or type implementing [`Parse`] parameter and returns a `TokenStream` or type
/// implementing [`ToTokens`]. And an optional [`&mut Emitter`](Emitter) and a
/// `&mut TokenStream` for storing a dummy output.
///
/// ```
/// # use proc_macro_utils::assert_tokens;
/// # use quote::{quote, ToTokens};
/// # use syn2 as syn;
/// use manyhow::{function, Emitter, Result};
/// use proc_macro2::TokenStream;
/// # let input = quote!();
/// # let output: TokenStream =
/// function!(input, |input: syn::Item,
///                   dummy: &mut TokenStream,
///                   emitter: &mut Emitter|
///  -> Result<syn::ItemImpl> {
///     // ..
///         # manyhow::bail!("unimplemented")
/// });
/// ```
///
/// *Note:* When `#[as_dummy]` is specified on the input, the `dummy: &mut
/// TokenStream` will be initialized with `input`. To override assign a new
/// `TokenStream`:
///
/// ```
/// use proc_macro_utils::assert_tokens;
/// use manyhow::{function, Result, SilentError};
/// use proc_macro2::TokenStream;
/// use quote::{quote, ToTokens};
///
/// let input = quote!(some input);
/// let output: TokenStream = function!(
///     #[as_dummy] input,
///     |input: TokenStream, dummy: &mut TokenStream|
///      -> Result<TokenStream, SilentError> {
///         assert_tokens!(dummy.to_token_stream(), {
///             some input
///         });
///         *dummy = quote! {
///             another input
///         };
///         // ..
///         Err(SilentError)
///     },
/// );
///
/// assert_tokens! {output, {another input}};
/// ```
#[macro_export]
macro_rules! function {
    (#[as_dummy] $input:expr, $impl:expr $(,)?) => {
        $crate::__macro_handler! {function_transparent; input: $input; $impl; dummy: $input}
    };
    ($input:expr, $impl:expr $(,)?) => {
        $crate::__macro_handler! {function_transparent; input: $input; $impl; dummy}
    };
}

#[test]
fn function_macro() {
    use proc_macro::TokenStream as TokenStream1;
    use quote::quote;
    // proc_macro2::TokenStream
    let output: TokenStream =
        function!(quote!(hello), |input: TokenStream| -> TokenStream { input });
    assert_eq!(output.to_string(), "hello");
    // proc_macro::TokenStream do not run :D
    if false {
        let _: TokenStream1 = function!(
            TokenStream1::from(quote!(hello)),
            |input: TokenStream1| -> TokenStream1 { input }
        );
    }

    #[cfg(feature = "syn2")]
    {
        use quote::ToTokens;
        let output: TokenStream = function!(
            #[as_dummy]
            quote!(hello;),
            |input: syn2::LitInt| -> TokenStream { input.into_token_stream() }
        );
        assert_eq!(
            output.to_string(),
            quote!(hello; ::core::compile_error! { "expected integer literal" }).to_string()
        );
        let output: TokenStream = function!(quote!(20), |_input: syn2::LitInt| -> syn2::Ident {
            syn2::parse_quote!(hello)
        });
        assert_eq!(output.to_string(), "hello");
    }
}

/// Implementation of a proc-macro
///
/// Note: for `TokenStream` either [`proc_macro::TokenStream`] or
/// [`proc_macro2::TokenStream`] can be used.
///
/// Trait is implemented for any [`function`](FnOnce), taking in either one (for
/// derive or function like macros) or two (for attribute macros) `TokenStream`s
/// and returns either a `TokenStream` or a [`Result<TokenStream, impl
/// ToTokensError>`](ToTokensError).
///
/// Additionally they can take optionally in any order a [`&mut
/// Emitter`](Emitter) which allows emitting errors without returning early. And
/// a `&mut TokenStream` to return a dummy `TokenStream` on failure, note that
/// this `TokenStream` must be the same type as the one returned.
pub trait MacroHandler<Input, Dummy, Output, Function, Error = Infallible> {
    #[allow(clippy::missing_errors_doc, missing_docs)]
    fn call(self, input: Input, dummy: &mut Dummy, emitter: &mut Emitter) -> Result<Output, Error>;
}

macro_rules! impl_attribute_macro {
    ($dummy:ident, $emitter:ident =>
        $(<$($Inputs:ident $(:$Bound:ident)?),+>(($($in_id:ident),+):($($in_ty:ty),+)$(, $ident:ident:$ty:ty)*), $Dummy:ident;)+) => {
        $(
        // NOTE: This `Clone` is just a marker to make this != Emitter
        impl<$($Inputs $(:$Bound)?,)+ Output: Clone, F> MacroHandler<($($in_ty,)+), $Dummy, Output, ($($in_ty,)* $($ty,)* Output)> for F
        where
            F: FnOnce($($in_ty,)+ $($ty,)*) -> Output
        {
            #[allow(unused)]
            fn call(self, ($($in_id,)+): ($($in_ty,)+), $dummy: &mut $Dummy, $emitter: &mut Emitter) -> Result<Output, Infallible> {
                Ok(self($($in_id,)+ $($ident),*))
            }
        }
        // NOTE: This `Clone` is just a marker to make this != Emitter
        impl<$($Inputs $(:$Bound)?,)+ Output: Clone, F, Error> MacroHandler<($($in_ty,)+), $Dummy, Output, ($($in_ty,)* $($ty,)* Result<Output, Error>), Error> for F
        where
            F: FnOnce($($in_ty,)+ $($ty,)*) -> Result<Output, Error>
        {
            #[allow(unused)]
            fn call(self, ($($in_id,)+): ($($in_ty,)+), $dummy: &mut $Dummy, $emitter: &mut Emitter) -> Result<Output, Error> {
                self($($in_id,)+ $($ident),*)
            }
        }
        )*
    };
}

impl_attribute_macro! {
    dummy, emitter =>
    <Input, Item, Dummy: Clone> ((input, item): (Input, Item), dummy: &mut Dummy), Dummy;
    <Input, Item> ((input, item): (Input, Item)), TokenStream;
    <Input, Item, Dummy: Clone> ((input, item): (Input, Item), dummy: &mut Dummy, emitter: &mut Emitter), Dummy;
    <Input, Item> ((input, item): (Input, Item), emitter: &mut Emitter), TokenStream;
    <Input, Item, Dummy: Clone> ((input, item): (Input, Item), emitter: &mut Emitter, dummy: &mut Dummy), Dummy;
    <Input, Dummy: Clone> ((input): (Input), dummy: &mut Dummy), Dummy;
    <Input> ((input): (Input)), TokenStream;
    <Input, Dummy: Clone> ((input): (Input), dummy: &mut Dummy, emitter: &mut Emitter), Dummy;
    <Input> ((input): (Input), emitter: &mut Emitter), TokenStream;
    <Input, Dummy: Clone> ((input): (Input), emitter: &mut Emitter, dummy: &mut Dummy), Dummy;
}
