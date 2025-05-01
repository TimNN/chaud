use crate::err::Result;
use crate::expect::Expect::*;
use crate::parse::Parser;
use proc_macro::{Delimiter, TokenStream};
use std::env;

#[derive(Debug, Default)]
pub struct Arg {
    pub pat: TokenStream,
    pub ty: TokenStream,
}

#[derive(Debug, Default)]
pub struct Input {
    pub hot: bool,
    pub reload: bool,
    pub is_method: bool,
    pub attrs: TokenStream,
    pub vis: TokenStream,
    pub name: TokenStream,
    pub life: TokenStream,
    pub args: Vec<Arg>,
    pub ret: TokenStream,
    pub body: TokenStream,
}

impl Input {
    pub fn parse(attr: &mut Parser, p: &mut Parser) -> Result<Self> {
        let mut this = Self::default();
        this.hot = cfg!(feature = "unsafe-hot-reload");
        this.reload = env::var_os("__CHAUD_RELOAD").is_some();

        while !attr.is_eos() {
            match () {
                _ if attr.maybe(kw("self")) => this.is_method = true,
                _ => attr.unexpected("Unsupported option")?,
            }
        }

        this.attrs = p.maybe_attrs()?;
        this.vis = p.collect(Parser::vis)?;
        p.expect(kw("fn"))?;
        this.name = p.collect(Parser::ident)?;

        this.life = p.collect(|p| {
            if p.maybe(sym('<')) {
                while !p.maybe(sym('>')) {
                    p.lifetime()?;
                    if p.maybe(sym('>')) {
                        break;
                    }
                    p.expect(sym(','))?;
                }
            }
            Ok(())
        })?;

        p.enter(Delimiter::Parenthesis, |p| {
            while !p.is_eos() {
                let mut arg = Arg::default();
                arg.pat = p.collect(Parser::pat)?;
                p.expect(sym(':'))?;
                arg.ty = p.collect(|p| p.ty_until(sym(',')))?;
                this.args.push(arg);
                if !p.maybe(sym(',')) {
                    break;
                }
            }
            if this.is_method && this.args.is_empty() {
                p.unexpected("Expected at least `self` parameter for methods")?;
            }
            p.expect_eos()
        })?;

        this.ret = p.collect(|p| {
            if p.maybe(sym('-')) {
                p.expect(sym('>'))?;
                p.ty_until(brace_tree)?;
            }
            Ok(())
        })?;

        p.enter(Delimiter::Brace, |p| {
            this.body = p.remaining();
            Ok(())
        })?;

        p.expect_eos()?;

        Ok(this)
    }
}
