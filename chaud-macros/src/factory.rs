use proc_macro::{Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

pub fn sym(c: char) -> Punct {
    Punct::new(c, Spacing::Alone)
}

pub fn ident(s: &str) -> Ident {
    Ident::new(s, Span::mixed_site())
}

pub fn parse(s: &'static str) -> TokenStream {
    s.parse().expect("Invalid token string")
}

macro_rules! ident {
    ($($t:tt)+) => { $crate::factory::ident(&format!($($t)*)) }
}

macro_rules! paren {
    [$($t:tt)*] => {
        ::proc_macro::Group::new(
            ::proc_macro::Delimiter::Parenthesis, tokens![$($t)*])
    }
}

macro_rules! brace {
    [$($t:tt)*] => {
        ::proc_macro::Group::new(
            ::proc_macro::Delimiter::Brace, tokens![$($t)*])
    }
}

macro_rules! bracket {
    [$($t:tt)*] => {
        ::proc_macro::Group::new(
            ::proc_macro::Delimiter::Bracket, tokens![$($t)*])
    }
}

macro_rules! attr {
    [$($t:tt)*] => {
        tokens![
            sym('#'),
            bracket![$($t)*]
        ]
    }
}

pub fn sep(sep: char, parts: impl IntoIterator<Item = impl IntoTokenStream>) -> TokenStream {
    let mut buf = TokenStream::new();

    for (i, p) in parts.into_iter().enumerate() {
        if i != 0 {
            buf.extend([sym(sep).ts()]);
        }
        buf.extend([p.ts()]);
    }

    buf
}

pub trait IntoLit {
    fn lit(self) -> Literal;
}

impl IntoLit for &str {
    fn lit(self) -> Literal {
        Literal::string(self)
    }
}

pub trait WithSpan {
    fn sp(self, sp: Span) -> Self;
}

impl WithSpan for TokenTree {
    fn sp(mut self, sp: Span) -> Self {
        self.set_span(sp);
        self
    }
}

impl WithSpan for Group {
    fn sp(mut self, sp: Span) -> Self {
        self.set_span(sp);
        self
    }
}

impl WithSpan for Ident {
    fn sp(mut self, sp: Span) -> Self {
        self.set_span(sp);
        self
    }
}

impl WithSpan for Punct {
    fn sp(mut self, sp: Span) -> Self {
        self.set_span(sp);
        self
    }
}

impl WithSpan for Literal {
    fn sp(mut self, sp: Span) -> Self {
        self.set_span(sp);
        self
    }
}

pub trait IntoTokenStream {
    fn ts(self) -> TokenStream;
}

impl IntoTokenStream for TokenStream {
    fn ts(self) -> TokenStream {
        self
    }
}

impl IntoTokenStream for TokenTree {
    fn ts(self) -> TokenStream {
        self.into()
    }
}

impl IntoTokenStream for Group {
    fn ts(self) -> TokenStream {
        TokenTree::from(self).into()
    }
}

impl IntoTokenStream for Ident {
    fn ts(self) -> TokenStream {
        TokenTree::from(self).into()
    }
}

impl IntoTokenStream for Punct {
    fn ts(self) -> TokenStream {
        TokenTree::from(self).into()
    }
}

impl IntoTokenStream for Literal {
    fn ts(self) -> TokenStream {
        TokenTree::from(self).into()
    }
}

impl<T: IntoTokenStream, const N: usize> IntoTokenStream for [T; N] {
    fn ts(self) -> TokenStream {
        self.into_iter().map(|t| t.ts()).collect()
    }
}

impl<T: Clone + IntoTokenStream> IntoTokenStream for &T {
    fn ts(self) -> TokenStream {
        self.clone().ts()
    }
}

impl<T: IntoTokenStream> IntoTokenStream for Option<T> {
    fn ts(self) -> TokenStream {
        match self {
            Some(t) => t.ts(),
            None => TokenStream::new(),
        }
    }
}

macro_rules! token {
    (@$i:ident) => {
        $crate::factory::ident(stringify!($i))
    };
    (@$l:literal) => {
        $crate::factory::parse($l)
    };
    (@$s:tt) => {
        $crate::factory::sym(const { $crate::factory::only_char(stringify!($s)) })
    };
    ($e:expr) => {
        $e
    };
}

macro_rules! tokens {
    [] => { TokenStream::new() };
    [$($(@$t:tt)?$($e:expr)?$(;;;$count:tt)?),*] => {{
        <
            // `stringify` allows us to reference `$t` (to get the correct
            // repetition), so we can generate a `1` for each expression.
            [::proc_macro::TokenStream; 0 $(+ 1 $($count)?)*]
            as $crate::factory::IntoTokenStream
        >::ts([
            $($crate::factory::IntoTokenStream::ts(
                token!($(@$t)* $($e)*)),)*
        ])
    }}
}

pub const fn only_char(s: &str) -> char {
    let [b] = s.as_bytes() else {
        panic!("too long");
    };
    assert!(b.is_ascii());
    *b as char
}
