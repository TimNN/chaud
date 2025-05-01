use crate::factory::*;
use crate::input::Input;
use proc_macro::{Ident, TokenStream};

impl Input {
    fn inline(&self) -> TokenStream {
        if !self.hot {
            return TokenStream::new();
        }

        attr![@inline]
    }

    fn higher(&self) -> TokenStream {
        if self.life.is_empty() {
            return TokenStream::new();
        }

        tokens![
            @for,
            &self.life
        ]
    }

    fn id(&self) -> TokenStream {
        tokens![
            @"concat!",
            paren![
                @r#""_CHAUD::", module_path!(), "::", stringify!"#,
                paren![&self.name]
            ]
        ]
    }

    fn arg_tys(&self) -> impl Iterator<Item = TokenStream> {
        self.args.iter().map(|a| a.ty.clone())
    }

    fn arg_pats(&self) -> impl Iterator<Item = TokenStream> {
        self.args.iter().map(|a| a.pat.clone())
    }

    fn arg_idents(&self) -> impl Iterator<Item = Ident> {
        self.args.iter().enumerate().map(|(i, _)| ident!("p{i}"))
    }

    fn arg_idents_outer(&self) -> impl Iterator<Item = TokenStream> {
        self.args.iter().enumerate().map(|(i, _)| {
            if self.is_method && i == 0 {
                tokens![@self]
            } else {
                tokens![ident!("p{i}")]
            }
        })
    }

    fn args_outer(&self) -> impl Iterator<Item = TokenStream> {
        self.args.iter().enumerate().map(|(i, a)| {
            if self.is_method && i == 0 {
                tokens![@self, @:, &a.ty]
            } else if self.hot {
                tokens![ident!("p{i}"), @:, &a.ty]
            } else {
                tokens![&a.pat, @:, &a.ty]
            }
        })
    }

    fn self_fixup(&self) -> TokenStream {
        if !self.is_method {
            return TokenStream::new();
        }

        tokens![
            @let,
            &self.args[0].pat,
            @=,
            @self,
            @;
        ]
    }
}

pub fn output(input: &Input) -> TokenStream {
    tokens![
        &input.attrs,
        input.inline(),
        &input.vis,
        @fn,
        &input.name,
        &input.life,
        paren![sep(',', input.args_outer())],
        &input.ret,
        brace![match input.hot {
            true => hot(input),
            false => tokens![input.self_fixup(), &input.body],
        }]
    ]
}

fn hot(input: &Input) -> TokenStream {
    tokens![
        @"struct __chaud_func;",
        // SAFETY: `Self::Ptr` is a function pointer.
        @"unsafe impl ::chaud::__internal::Func for __chaud_func",
        brace![
            @"type Ptr =",
            input.higher(),
            @fn,
            paren![sep(',', input.arg_tys())],
            &input.ret,
            @;,

            @"const ID: &'static str =",
            input.id(),
            @;,

            @"const actual: Self::Ptr = ",
            @|,
            sep(',', input.arg_pats()),
            @|,
            brace![&input.body],
            @;,

            @"const init: Self::Ptr = ",
            @|,
            sep(',', input.arg_idents()),
            @|,
            brace![
                @"__chaud_FUNC.init()",
                paren![sep(',', input.arg_idents())]
            ],
            @;,

            @"const jump: Self::Ptr = ",
            @|,
            sep(',', input.arg_idents()),
            @|,
            brace![
                @"__chaud_FUNC.get()",
                paren![sep(',', input.arg_idents())]
            ],
            @;
        ],

        // SAFETY: The user must ensure that no conflicting `export_name`s are
        // being generated. This is covered under the `unsafe-hot-reload`
        // feature opt-in.
        attr![@unsafe, paren![
            @export_name,
            @=,
            input.id()
        ]],
        @"static __chaud_FUNC: ::chaud::__internal::AtomicFunc<__chaud_func>",
        @" = ::chaud::__internal::AtomicFunc::new();",

        @"__chaud_FUNC.get()",
        paren![sep(',', input.arg_idents_outer())]
    ]
}
