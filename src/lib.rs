#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions)]
//! Proc **m**acro **anyhow**, a combination of ideas from
//! [`anyhow`](docs.rs/anyhow) and
//! [`proc-macro-error`](docs.rs/proc-macro-error) to improve proc macro
//! development, especially focused on the error handling.
//!
//! # Motivation
//! Error handling in proc-macros is unideal, as the top level functions of proc
//! macros can only return `TokenStreams` both in success and failure case. This
//! means that I often write code like this, moving the actual impelemtation in
//! a seperate function to be able to use the ergonomic rust error handling with
//! e.g. `?`.
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
//! proc macro implementation, reducing the above example to:
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
//! fn my_macro(input: TokenStream2) -> syn::Result<TokenStream2> {
//!     // ..
//! #   Ok(quote!())
//! }
//! ```
//!
//! See [Without macros](#without-macros) to see what this expands to under the
//! hood.
//!
//! A proc macro function marked as `#[manyhow]` can take and return any
//! [`TokenStream`](AnyTokenStream), and can also return `Result<TokenStream,
//! E>` where `E` implments [`ToTokensError`]. As additional paramters a
//! [dummy](#dummy-mut-tokenstream) and/or [emitter](#emitter-mut-emitter) can
//! be specified.
//!
//! The `manyhow` attribute takes one optional flag when used for `proc_macro`
//! and `proc_macro_attribute`. `#[manyhow(input_as_dummy)]` will take the input
//! of a function like `proc_macro` to initialize the [dummy `&mut
//! TokenStream`](#dummy-mut-tokenstream) while `#[manyhow(item_as_dummy)]` on
//! `proc_macro_attribute` will initialize the dummy with the annotated item.
//!
//! # Without macros
//! `manyhow` can be used without proc macros, and they can be disabled by
//! adding `manyhow` with `default-features=false`.
//!
//! The usage is more or less the same, though with some added boilerplate from
//! needing to invoke one of [`function`], [`attribute`] or [`derive`](derive())
//! directly.
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
//!     manyhow::function(
//!         input,
//!         false,
//!         |input: TokenStream2| -> syn::Result<TokenStream2> {
//!             // ..
//! #           Ok(quote!())
//!         },
//!     )
//! }
//! ```
//! [`Emitter`](#emitter-mut-emitter) and [dummy
//! `TokenStream`](#dummy-mut-tokenstream) can also be used. [`function`] and
//! [`attribute`] take an additional boolean parameter controlling whether the
//! input/item will be used as initial dummy.
//!
//! # `emitter: &mut Emitter`
//! [`MacroHandler`]s (the trait defining what closures/functions can be used
//! with `manyhow`) can take a mutable reference to an [`Emitter`]. This
//! allows to collect errors, but not fail imidiatly.
//!
//! [`Emitter::fail_if_dirty`] can be used to return if an [`Emitter`] contains
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
//!     emitter.fail_if_dirty()?;
//!     // ..
//! #   Ok(quote!())
//! }
//! ```
//!
//! # `dummy: &mut TokenStream`
//! [`MacroHandler`]s also take a mutable reference to a `TokenStream`, to
//! enable emitting some dummy code to be used in case the macro errors.
//!
//! This allows either appending tokens e.g. with [`ToTokens::to_tokens`] or
//! directly setting the dummy code e.g. `*dummy = quote!{some tokens}`.

use std::convert::Infallible;

pub use macros::manyhow;
use proc_macro2::TokenStream;
#[cfg(doc)]
use quote::ToTokens;

extern crate proc_macro;

#[macro_use]
mod span_ranged;
pub use span_ranged::{to_tokens_span_range, SpanRanged};
#[macro_use]
mod macro_rules;
mod error;
pub use error::*;

#[doc(hidden)]
pub mod __private {
    pub use crate::span_ranged::{SpanRangedToSpanRange, ToTokensToSpanRange};
}

/// Marker trait for [`proc_macro::TokenStream`] and
/// [`proc_macro2::TokenStream`]
pub trait AnyTokenStream: Clone + From<TokenStream> + Into<TokenStream> {}
impl AnyTokenStream for TokenStream {}
impl AnyTokenStream for proc_macro::TokenStream {}

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
pub fn attribute<
    Input: AnyTokenStream,
    Item: AnyTokenStream,
    Output: AnyTokenStream,
    Return: AnyTokenStream,
    Error: ToTokensError,
    Function,
>(
    input: impl AnyTokenStream,
    item: impl AnyTokenStream,
    item_as_dummy: bool,
    body: impl MacroHandler<(Input, Item), Output, Function, Error>,
) -> Return {
    let mut tokens: Output = if item_as_dummy {
        item.clone().into().into()
    } else {
        TokenStream::default().into()
    };
    let mut emitter = Emitter::new();
    let output = body.call(
        (input.into().into(), item.into().into()),
        &mut tokens,
        &mut emitter,
    );
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
/// implementation
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
    body: impl MacroHandler<(Item,), Output, Function, Error>,
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

/// Handles function like [`proc_macro`](https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros)
/// implementation
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
    body: impl MacroHandler<(Input,), Output, Function, Error>,
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
pub trait MacroHandler<Input, Output, Function, Error = Infallible> {
    #[allow(clippy::missing_errors_doc, missing_docs)]
    fn call(self, input: Input, dummy: &mut Output, emitter: &mut Emitter)
    -> Result<Output, Error>;
}

macro_rules! impl_attribute_macro {
    ($dummy:ident, $emitter:ident =>
        $(<$($Inputs:ident),+>(($($in_id:ident),+):($($in_ty:ty),+)$(, $ident:ident:$ty:ty)*);)+) => {
        $(
        impl<$($Inputs,)+ Output: AnyTokenStream, F> MacroHandler<($($Inputs,)+), Output, ($($in_ty,)* $($ty,)* Output)> for F
        where
            F: FnOnce($($in_ty,)+ $($ty,)*) -> Output
        {
            #[allow(unused)]
            fn call(self, ($($in_id,)+): ($($in_ty,)+), $dummy: &mut Output, $emitter: &mut Emitter) -> Result<Output, Infallible> {
                Ok(self($($in_id,)+ $($ident),*))
            }
        }
        impl<$($Inputs,)+ Output: AnyTokenStream, F, Error> MacroHandler<($($Inputs,)+), Output, ($($in_ty,)* $($ty,)* Result<Output, Error>), Error> for F
        where
            F: FnOnce($($in_ty,)+ $($ty,)*) -> Result<Output, Error>
        {
            #[allow(unused)]
            fn call(self, ($($in_id,)+): ($($in_ty,)+), $dummy: &mut Output, $emitter: &mut Emitter) -> Result<Output, Error> {
                self($($in_id,)+ $($ident),*)
            }
        }
        )*
    };
}

impl_attribute_macro! {
    dummy, emitter =>
    <Input, Item> ((input, item): (Input, Item), dummy: &mut Output);
    <Input, Item> ((input, item): (Input, Item));
    <Input, Item> ((input, item): (Input, Item), dummy: &mut Output, emitter: &mut Emitter);
    <Input, Item> ((input, item): (Input, Item), emitter: &mut Emitter);
    <Input, Item> ((input, item): (Input, Item), emitter: &mut Emitter, dummy: &mut Output);
    <Input> ((input): (Input), dummy: &mut Output);
    <Input> ((input): (Input));
    <Input> ((input): (Input), dummy: &mut Output, emitter: &mut Emitter);
    <Input> ((input): (Input), emitter: &mut Emitter);
    <Input> ((input): (Input), emitter: &mut Emitter, dummy: &mut Output);
}
