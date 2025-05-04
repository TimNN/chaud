use crate::factory::*;
use crate::input::PersistInput;
use proc_macro::TokenStream;

impl PersistInput {
    fn id(&self) -> TokenStream {
        tokens![
            @"concat!",
            paren![
                @r#""_CHAUD::", module_path!(), "::", stringify!"#,
                paren![self.id.as_ref().unwrap_or(&self.name)]
            ]
        ]
    }

    fn export_attr(&self) -> TokenStream {
        if !self.common.hot {
            return tokens![];
        }

        // SAFETY: The user must ensure that no conflicting `export_name`s are
        // being generated. This is covered under the `unsafe-hot-reload`
        // feature opt-in.
        attr![@unsafe, paren![
            @export_name,
            @=,
            self.id()
        ]]
    }

    fn link_attr(&self) -> TokenStream {
        attr![@link_name, @=, self.id()]
    }

    pub fn output(&self) -> TokenStream {
        output(self)
    }
}

fn output(input: &PersistInput) -> TokenStream {
    if input.common.reload {
        storage_ref(input)
    } else {
        storage_def(input)
    }
}

fn storage_def(input: &PersistInput) -> TokenStream {
    tokens![
        &input.attrs,
        input.export_attr(),
        @static,
        &input.name,
        @:,
        &input.ty,
        @=,
        &input.init,
        @;
    ]
}

fn storage_ref(input: &PersistInput) -> TokenStream {
    tokens![
        // SAFETY: The user must ensure that the type of this static is the
        // same as during the original compilation. This is covered under the
        // `unsafe-hot-reload` feature opt-in.
        @r#"unsafe extern "Rust""#,
        brace![
            input.link_attr(),
            @safe,
            @static,
            &input.name,
            @:,
            &input.ty,
            @;
        ]
    ]
}
