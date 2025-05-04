use crate::factory::*;
use crate::input::Input;
use proc_macro::TokenStream;

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

    fn name(&self) -> TokenStream {
        tokens![
            @"concat!",
            paren![
                @r#"module_path!(), "::", stringify!"#,
                paren![&self.name]
            ]
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

            @"const NAME: &'static str =",
            input.name(),
            @;,

            @"const actual: Self::Ptr = ",
            @|,
            sep(',', input.arg_pats()),
            @|,
            brace![&input.body],
            @;
        ],

        if input.reload { storage_ref(input) } else { storage_def(input) },

        @"__chaud_FUNC.get()",
        paren![sep(',', input.arg_idents_outer())]
    ]
}

fn storage_def(input: &Input) -> TokenStream {
    tokens![
        // SAFETY: The user must ensure that no conflicting `export_name`s are
        // being generated. This is covered under the `unsafe-hot-reload`
        // feature opt-in.
        attr![@unsafe, paren![
            @export_name,
            @=,
            input.id()
        ]],
        @"static __chaud_FUNC: ::chaud::__internal::FuncStorage<__chaud_func>",
        @" = ::chaud::__internal::FuncStorage::new();"
    ]
}

fn storage_ref(input: &Input) -> TokenStream {
    tokens![
        // SAFETY: The user must ensure that the type of the function is the
        // same as during the original compilation. This is covered under the
        // `unsafe-hot-reload` feature opt-in.
        @r#"unsafe extern "Rust""#,
        brace![
            attr![@link_name, @=, input.id()],
            @"safe static __chaud_FUNC: ::chaud::__internal::FuncStorage<__chaud_func>;"
        ],
        @"
            ::chaud::__internal::ctor! {
                #[ctor]
                fn __chaud__reload() {
                    __chaud_FUNC.update();
                }
            }
        "
    ]
}
