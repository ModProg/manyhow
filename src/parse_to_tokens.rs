use std::marker::PhantomData;

use proc_macro2::TokenStream;

use crate::{AnyTokenStream, Emitter, MacroHandler, ToTokensError};
pub trait ParseToTokens<T> {
    fn manyhow_parse(&self, input: impl AnyTokenStream, attr: bool) -> Result<T, TokenStream>;
    #[allow(clippy::wrong_self_convention)]
    fn manyhow_into_token_stream(&self, input: T, tokens: TokenStream) -> TokenStream;
}

pub struct WhatType<T>(PhantomData<T>);

impl<T> WhatType<T> {
    /// Always panics
    pub fn identify(&self) -> Result<T, TokenStream> {
        panic!("DON'T YOU DARE CALL ME")
    }

    pub fn from(_ty: &T) -> Self {
        Self(Default::default())
    }

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl<T> Clone for WhatType<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WhatType<T> {}

impl<T: Into<TokenStream> + From<TokenStream>> ParseToTokens<T> for WhatType<T> {
    fn manyhow_parse(&self, input: impl AnyTokenStream, _attr: bool) -> Result<T, TokenStream> {
        Ok(input.into().into())
    }

    fn manyhow_into_token_stream(&self, input: T, mut tokens: TokenStream) -> TokenStream {
        tokens.extend(input.into());
        tokens
    }
}

#[cfg(feature = "syn2")]
impl<T: syn2::parse::Parse + quote::ToTokens> ParseToTokens<T> for &WhatType<T> {
    fn manyhow_parse(&self, input: impl AnyTokenStream, attr: bool) -> Result<T, TokenStream> {
        let input = input.into();
        let empty = input.is_empty();
        syn2::parse2(input).map_err(|e| {
            let mut e = e.into_compile_error();
            if attr && empty {
                error_message!("while parsing attribute argument (`#[... (...)]`)")
                    .to_tokens(&mut e)
            }
            e
        })
    }

    fn manyhow_into_token_stream(&self, input: T, mut tokens: TokenStream) -> TokenStream {
        input.to_tokens(&mut tokens);
        tokens
    }
}

#[cfg(feature = "syn2")]
#[test]
#[allow(unused)]
fn test_inference() {
    if false {
        let wt = &WhatType::new();
        let ts: proc_macro::TokenStream = wt.manyhow_parse(quote::quote!(test), false).unwrap();
        let wt = &WhatType::new();
        if false {
            let wt: Result<syn2::Ident, _> = wt.identify();
        }
        let ts: syn2::Ident = wt.manyhow_parse(quote::quote!(test), false).unwrap();
    }
}

macro_rules! transparent_handlers {
    ($name:ident; $($input:ident: $Input:ident $($context:expr)?),*; $($dummy:ident)?) => {
        /// Internal implementation for macro.
        pub fn $name<$($Input,)* Dummy: AnyTokenStream, Output, Error: ToTokensError, Function,>(
            $($input: Result<$Input, TokenStream>,)*
            $($dummy: Option<impl AnyTokenStream>,)?
            body: impl MacroHandler<($($Input,)*), Dummy, Output, Function, Error>,
        ) -> Result<(Output, TokenStream), TokenStream> {
            #[allow(unused)]
            let mut dummy = TokenStream::new();
            $(let mut dummy = $dummy.unwrap_or_default().into();)?
            $(let $input = match $input {
                Ok($input) => $input,
                Err(tokens) => {
                    dummy.extend(tokens);
                    $($crate::error_message!($context).to_tokens(&mut dummy);)?
                    return Err(dummy);
                }
            };)*
            let mut tokens = dummy.into();
            let mut emitter = Emitter::new();
            let output = body.call(($($input,)*), &mut tokens, &mut emitter);
            match output {
                Ok(tokens) => Ok((tokens, emitter.into_token_stream())),
                Err(error) => {
                    let mut tokens = tokens.into();
                    error.to_tokens(&mut tokens);
                    emitter.to_tokens(&mut tokens);
                    Err(tokens)
                }
            }
        }
    };
}

transparent_handlers! { function_transparent; input: Input; dummy }
transparent_handlers! { derive_transparent; item: Item; }
transparent_handlers! { attribute_transparent; input: Input, item: Item; dummy }
