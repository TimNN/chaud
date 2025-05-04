use crate::err::Result;
use crate::expect::Expect::{self, *};
use core::iter::Peekable;
use proc_macro::{Delimiter, Span, TokenStream, TokenTree, token_stream};

pub struct Parser {
    it: Peekable<token_stream::IntoIter>,
    prev: Span,
    buf: Option<TokenStream>,
}

impl Parser {
    pub fn new(ts: TokenStream) -> Self {
        Self {
            it: ts.into_iter().peekable(),
            prev: Span::call_site(),
            buf: None,
        }
    }

    fn peek(&mut self) -> Option<&TokenTree> {
        self.it.peek()
    }

    fn span(&mut self) -> Span {
        match self.peek() {
            Some(t) => t.span(),
            _ => self.prev,
        }
    }

    fn next(&mut self) {
        let t = self.it.next().expect("should have `peek`ed");
        self.prev = t.span();
        if let Some(buf) = &mut self.buf {
            buf.extend([t]);
        }
    }

    fn next_get(&mut self) -> Option<TokenTree> {
        assert!(self.buf.is_none(), "`next_get` called while collecting");
        self.it.next().inspect(|t| {
            self.prev = t.span();
        })
    }

    pub fn unexpected(&mut self, msg: &str) -> Result<()> {
        bail!(self.span(), "{}", msg)
    }

    pub fn remaining(&mut self) -> TokenStream {
        (&mut self.it).collect()
    }

    pub fn collect(&mut self, f: impl FnOnce(&mut Parser) -> Result<()>) -> Result<TokenStream> {
        assert!(self.buf.is_none(), "Nested `collect` is not supported");
        self.buf = Some(TokenStream::new());
        f(self)?;
        Ok(self.buf.take().expect("`buf` was stolen"))
    }

    pub fn enter<T>(
        &mut self,
        d: Delimiter,
        f: impl FnOnce(&mut Parser) -> Result<T>,
    ) -> Result<T> {
        assert!(
            self.buf.is_none(),
            "`enter` during `collect` is not supported"
        );

        match self.next_get() {
            Some(TokenTree::Group(g)) if g.delimiter() == d => f(&mut Parser {
                it: g.stream().into_iter().peekable(),
                prev: g.span_open(),
                buf: None,
            }),
            _ => bail!(self.prev, "Expected `{}`", ds(d)),
        }
    }

    pub fn is_eos(&mut self) -> bool {
        self.peek().is_none()
    }

    pub fn expect_eos(&mut self) -> Result<()> {
        if let Some(t) = self.peek() {
            bail!(t, "Expected end of stream");
        }
        Ok(())
    }

    pub fn maybe(&mut self, e: Expect) -> bool {
        match self.peek() {
            Some(t) if e.matches(t) => {
                self.next();
                true
            }
            _ => false,
        }
    }

    pub fn expect(&mut self, e: Expect) -> Result<()> {
        if !self.maybe(e) {
            if self.peek().is_none() {
                bail!(self.span(), "Expected {e}, got end of stream");
            }

            bail!(self.span(), "Expected {e}")
        }
        Ok(())
    }

    pub fn ident(&mut self) -> Result<()> {
        self.expect(ident)
    }

    pub fn lifetime(&mut self) -> Result<()> {
        self.expect(lifetime)?;
        self.expect(ident)
    }

    pub fn pat(&mut self) -> Result<()> {
        self.maybe(kw("mut"));
        self.expect(ident)
    }

    #[expect(clippy::unnecessary_wraps, reason = "better composition")]
    pub fn ty_until(&mut self, end: Expect) -> Result<()> {
        let mut until = end;
        let mut depth = 0;

        while let Some(t) = self.peek() {
            if until.matches(t) {
                match depth {
                    0 => {
                        break;
                    }
                    1 => {
                        depth = 0;
                        until = end;
                    }
                    _ => {
                        depth -= 1;
                    }
                }
            }

            if self.maybe(sym('<')) {
                depth += 1;
                until = sym('>');
            } else {
                self.next();
            }
        }

        Ok(())
    }

    #[expect(clippy::unnecessary_wraps, reason = "better composition")]
    pub fn expr(&mut self) -> Result<()> {
        while let Some(t) = self.peek() {
            if sym(';').matches(t) || sym(',').matches(t) {
                break;
            }
            self.next();
        }

        Ok(())
    }

    #[expect(clippy::unnecessary_wraps, reason = "better composition")]
    pub fn vis(&mut self) -> Result<()> {
        if self.maybe(kw("pub")) {
            self.maybe(paren_tree);
        }
        Ok(())
    }

    pub fn maybe_attrs(&mut self) -> Result<TokenStream> {
        self.collect(|p| {
            while p.maybe(sym('#')) {
                p.expect(bracket_tree)?;
            }
            Ok(())
        })
    }
}

fn ds(d: Delimiter) -> &'static str {
    match d {
        Delimiter::Parenthesis => "(",
        Delimiter::Brace => "{",
        Delimiter::Bracket => "[",
        Delimiter::None => panic!("invalid `None` delimiter"),
    }
}
