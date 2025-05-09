use crate::factory::*;
use crate::input::{HotInput, PersistInput};
use proc_macro::TokenStream;

impl HotInput {
    fn inline(&self) -> TokenStream {
        if !self.common.hot {
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

    fn arg_tys(&self) -> impl Iterator<Item = TokenStream> + use<'_> {
        self.args.iter().map(|a| a.ty.clone())
    }

    fn arg_pats(&self) -> impl Iterator<Item = TokenStream> + use<'_> {
        self.args.iter().map(|a| a.pat.clone())
    }

    fn arg_idents_outer(&self) -> impl Iterator<Item = TokenStream> + use<'_> {
        self.args.iter().enumerate().map(|(i, _)| {
            if self.is_method && i == 0 {
                tokens![@self]
            } else {
                tokens![ident!("p{i}")]
            }
        })
    }

    fn args_outer(&self) -> impl Iterator<Item = TokenStream> + use<'_> {
        self.args.iter().enumerate().map(|(i, a)| {
            if self.is_method && i == 0 {
                tokens![@self, @:, &a.ty]
            } else if self.common.hot {
                tokens![ident!("p{i}"), @:, &a.ty]
            } else {
                tokens![&a.pat, @:, &a.ty]
            }
        })
    }

    fn self_fixup(&self) -> TokenStream {
        if !self.is_method {
            return tokens![];
        }

        tokens![
            @let,
            &self.args[0].pat,
            @=,
            @self,
            @;
        ]
    }

    pub fn output(&self) -> TokenStream {
        output(self)
    }
}

fn output(input: &HotInput) -> TokenStream {
    tokens![
        &input.attrs,
        input.inline(),
        &input.vis,
        @fn,
        &input.name,
        &input.life,
        paren![sep(',', input.args_outer())],
        &input.ret,
        brace![match input.common.hot {
            true => hot(input),
            false => tokens![input.self_fixup(), &input.body],
        }]
    ]
}

fn hot(input: &HotInput) -> TokenStream {
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

        storage(input),
        reload(input),

        @"__chaud_FUNC.get()",
        paren![sep(',', input.arg_idents_outer())]
    ]
}

fn storage(input: &HotInput) -> TokenStream {
    let persist = PersistInput {
        common: input.common,
        attrs: tokens![],
        vis: tokens![],
        name: tokens![@__chaud_FUNC],
        id: Some(input.name.clone()),
        ty: tokens![@"::chaud::__internal::FuncStorage<__chaud_func>"],
        init: tokens![@"::chaud::__internal::FuncStorage::new()"],
    };

    persist.output()
}

fn reload(input: &HotInput) -> TokenStream {
    if !input.common.reload {
        return tokens![];
    }

    tokens![
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
