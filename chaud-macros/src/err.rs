use crate::factory::*;
use proc_macro::{Span, TokenStream, TokenTree};

pub type Result<T> = core::result::Result<T, Error>;

pub struct Error {
    pub span: Span,
    pub msg: String,
}

impl Error {
    pub fn reporting(
        original: TokenStream,
        f: impl FnOnce() -> Result<TokenStream>,
    ) -> TokenStream {
        let err = match f() {
            Ok(ts) => return ts,
            Err(e) => e,
        };

        tokens![
            original,
            ident("compile_error").sp(err.span),
            @!,
            paren![err.msg.lit()].sp(err.span),
            @;
        ]
    }
}

macro_rules! bail {
    ($span:expr, $($msg:tt)+) => {
        return $crate::err::Result::Err($crate::err::Error {
            span: $crate::err::HasSpan::span(&$span),
            msg: format!($($msg)*)
        })
    }
}

pub trait HasSpan {
    fn span(&self) -> Span;
}

impl<T: HasSpan> HasSpan for &T {
    fn span(&self) -> Span {
        (*self).span()
    }
}

impl HasSpan for Span {
    fn span(&self) -> Span {
        *self
    }
}

impl HasSpan for TokenTree {
    fn span(&self) -> Span {
        self.span()
    }
}
