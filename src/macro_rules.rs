#[cfg(doc)]
use proc_macro2::Span;
#[cfg(doc)]
use quote::ToTokens;

#[cfg(doc)]
use crate::{ErrorMessage, SpanRanged};

#[doc(hidden)]
#[macro_export]
macro_rules! __error_message_internal {
    ((cs($($fmt:tt)*)$(.$fn:ident($($fmt_fn:tt)*))*), (), ()) => {
        $crate::ErrorMessage::call_site($($fmt)*)
            $(.attachment(::core::stringify!($fn), $($fmt_fn)*))*
    };
    ((new($span:expr)($($fmt:tt)*)$(.$fn:ident($($fmt_fn:tt)*))*), (), ()) => {
        $crate::ErrorMessage::new(
            $crate::span_range!($span),
            $($fmt)*
        )
            $(.attachment(::core::stringify!($fn), $($fmt_fn)*))*
    };
    // ident = expr
    ($head:tt, ($($fmt:tt)*), (, $ident:ident = $expr:expr, $($tail:tt)*)) => {
        $crate::__error_message_internal!($head, ($($fmt)*, $ident = $expr), (, $($tail)*))
    };
    ($head:tt, ($($fmt:tt)*), (, $ident:ident = $expr:expr; $($tail:tt)*)) => {
        $crate::__error_message_internal!($head, ($($fmt)*, $ident = $expr), (; $($tail)*))
    };
    ($head:tt, ($($fmt:tt)*), (, $ident:ident = $expr:expr)) => {
        $crate::__error_message_internal!($head, ($($fmt)*, $ident = $expr), ())
    };
    // expr,
    ($head:tt, ($($fmt:tt)*), (, $expr:expr, $($tail:tt)*)) => {
        $crate::__error_message_internal!($head, ($($fmt)*, $expr), (, $($tail)*))
    };
    ($head:tt, ($($fmt:tt)*), (, $expr:expr; $($tail:tt)*)) => {
        $crate::__error_message_internal!($head, ($($fmt)*, $expr), (; $($tail)*))
    };
    ($head:tt, ($($fmt:tt)*), (, $expr:expr)) => {
        $crate::__error_message_internal!($head, ($($fmt)*, $expr), ())
    };
    // ; ident = "format", arguments
    (($($head:tt)*), $fmt:tt, ($(,)?$(;)?)) => {
        $crate::__error_message_internal!(($($head)*(::core::format_args!$fmt)), (), ())
    };
    (($($head:tt)*), $fmt:tt, ($(,)?; $attachment:ident = $fmt_str:literal $($tail:tt)*)) => {
        $crate::__error_message_internal!(($($head)*(::core::format_args!$fmt).$attachment), ($fmt_str), ($($tail)*))
    };
}

/// Creates an [`ErrorMessage`], comparable to the [`anyhow!`](https://docs.rs/anyhow/latest/anyhow/macro.anyhow.html) macro
///
/// If the first argument is not a literal it is taken as the span of the error.
/// The span expression can **either** implement [`SpanRanged`] or implement
/// [`ToTokens`]. Otherwise [`Span::call_site`] is used.
///
/// ```
/// # use proc_macro2::Span;
/// # use quote::quote;
/// # use manyhow::error_message;
/// assert_eq!(
///     error_message!("format {} string{named}", "<3", named = "!").to_string(),
///     "format <3 string!"
/// );
/// // Span can either be `proc_macro::Span` or `proc_macro2::Span`
/// assert_eq!(
///     error_message!(Span::call_site(), "spanned error").to_string(),
///     "spanned error"
/// );
/// # if false {
/// // Or any expression implementing `quote::ToTokens`
/// assert_eq!(
///     error_message!(quote!(some tokens), "spanned error").to_string(),
///     "spanned error"
/// );
/// # }
/// ```
///
/// On top of the standard [`format_args!`] parameters additional attachments
/// can be specified delimited with `;`.
///
/// ```
/// # use proc_macro2::Span;
/// # use quote::quote;
/// # use manyhow::error_message;
/// assert_eq!(
///     error_message!(
///         "format {} string{named}", "<3", named = "!";
///         error = "some additional error";
///         info = "some info as well";
///         custom_attachment = "amazing"
///     ).to_string(),
///     "format <3 string!
///
///   = error: some additional error
///   = info: some info as well
///   = custom_attachment: amazing
/// "
/// );
/// ```
#[macro_export]
macro_rules! error_message {
    ($fmt:literal $($tt:tt)*) => {
        $crate::__error_message_internal!((cs), ($fmt), ($($tt)*))
    };
    ($span:expr, $fmt:literal $($tt:tt)*) => {
        $crate::__error_message_internal!((new($span)), ($fmt), ($($tt)*))
    };
}

#[macro_export]
/// Exit by returning error, matching [`anyhow::bail!`](https://docs.rs/anyhow/latest/anyhow/macro.bail.html)
///
/// The syntax is identical to [`error_message!`], the only difference is, that
/// a single expression with an error is supported as well.
/// ```should_panic
/// # use manyhow::bail;
/// # use proc_macro2::Span;
/// # use syn2 as syn;
/// bail!("an error message"; error = "with attachments");
/// let span = Span::call_site();
/// bail!(span, "error message");
/// let error = syn::Error::new(Span::call_site(), "an error");
/// # Ok::<_, manyhow::Error>(())
/// ```
macro_rules! bail {
    ($msg:literal) => {
        return ::core::result::Result::Err($crate::error_message!($msg).into());
    };
    ($error:expr) => {
        return ::core::result::Result::Err($error.into());
    };
    ($($tt:tt)*) => {
        return ::core::result::Result::Err($crate::error_message!($($tt)*).into());
    };
}

#[cfg(test)]
mod test {
    use proc_macro::Span;
    use quote::quote;

    use crate::ErrorMessage;

    macro_rules! returned {
        ($ty:ty, $expr:expr) => {
            #[allow(unreachable_code)]
            (|| -> $ty {
                $expr;
                unreachable!();
            })()
        };
    }
    #[test]
    fn bail() {
        assert_eq!(
            returned!(Result<(), ErrorMessage>, bail!("format"))
                .unwrap_err()
                .to_string(),
            "format"
        );
        assert_eq!(
            returned!(Result<(), ErrorMessage>, bail!("format {}", 1))
                .unwrap_err()
                .to_string(),
            "format 1"
        );
        let b = "ho";
        assert_eq!(
            returned!(Result<(), ErrorMessage>, bail!("format {} {a} {} {b}", 1, 2, a = 4))
                .unwrap_err()
                .to_string(),
            "format 1 4 2 ho"
        );
    }

    #[test]
    fn error_message() {
        assert_eq!(error_message!("test").to_string(), "test");
        assert_eq!(error_message!("test";).to_string(), "test");
        assert_eq!(
            error_message!(
                "test";
                error = "hello {} {a}", 1 + 4, a = ""
            )
            .to_string(),
            "test\n\n  = error: hello 5 \n"
        );
        assert_eq!(
            error_message!(
                "test";
                error = "hello {} {a}", 1 + 4, a = "";
            )
            .to_string(),
            "test\n\n  = error: hello 5 \n"
        );
        assert_eq!(
            error_message!(
                "test";
                error = "hello {} {a}", 1 + 4, a = "",;
                hint = "a hint"
            )
            .to_string(),
            "test\n\n  = error: hello 5 \n  = hint: a hint\n"
        );
    }

    // Only tests that it compiles
    fn _error_message_spanned() {
        let span = Span::call_site();
        _ = error_message!(span, "test");

        let span = proc_macro::Span::call_site();
        _ = error_message!(span, "test");

        let tokens = quote!(test);
        _ = error_message!(tokens, "an error message",);
        _ = error_message!(tokens, "an error message",;);
        _ = error_message!(tokens, "an error message",; warning="and a warning";);
    }
}
