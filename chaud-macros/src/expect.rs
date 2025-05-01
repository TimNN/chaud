use core::fmt;
use proc_macro::{Delimiter, Spacing, TokenTree};

#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum Expect<'a> {
    brace_tree,
    bracket_tree,
    ident,
    kw(&'a str),
    lifetime,
    paren_tree,
    sym(char),
}

impl fmt::Display for Expect<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expect::brace_tree => write!(f, "`{{`"),
            Expect::bracket_tree => write!(f, "`[`"),
            Expect::ident => write!(f, "an identifier"),
            Expect::kw(k) => write!(f, "`{k}`"),
            Expect::lifetime => write!(f, "lifetime"),
            Expect::paren_tree => write!(f, "`(`"),
            Expect::sym(c) => write!(f, "`{c}`"),
        }
    }
}

impl Expect<'_> {
    pub fn matches(&self, t: &TokenTree) -> bool {
        match self {
            Expect::brace_tree => {
                matches!(t, TokenTree::Group(g) if g.delimiter() == Delimiter::Brace)
            }
            Expect::bracket_tree => {
                matches!(t, TokenTree::Group(g) if g.delimiter() == Delimiter::Bracket)
            }
            Expect::ident => matches!(t, TokenTree::Ident(_)),
            Expect::kw(k) => matches!(t, TokenTree::Ident(i) if i.to_string() == *k),
            Expect::lifetime => {
                matches!(t, TokenTree::Punct(p) if p.as_char() == '\'' && p.spacing() == Spacing::Joint)
            }
            Expect::paren_tree => {
                matches!(t, TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis)
            }
            Expect::sym(c) => matches!(t, TokenTree::Punct(p) if p.as_char() == *c),
        }
    }
}
