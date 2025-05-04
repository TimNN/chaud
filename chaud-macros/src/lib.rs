#![doc = include_str!("../README.md")]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::wildcard_imports,
    clippy::indexing_slicing,
    clippy::enum_glob_use,
    clippy::expect_used,
    clippy::panic,
    reason = "less restrictions on internal build-time dependencies"
)]

use self::err::Error;
use self::input::{HotInput, PersistInput};
use self::parse::Parser;
use proc_macro::TokenStream;

#[macro_use]
mod factory;

#[macro_use]
mod err;

mod expect;
mod input;
mod output;
mod parse;

#[proc_macro_attribute]
pub fn hot(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut attr = Parser::new(attr);
    let mut p = Parser::new(item.clone());

    Error::reporting(item.clone(), || {
        let input = HotInput::parse(&mut attr, &mut p)?;

        Ok(input.output())
    })
}

#[proc_macro_attribute]
pub fn persist(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut attr = Parser::new(attr);
    let mut p = Parser::new(item.clone());

    Error::reporting(item.clone(), || {
        let input = PersistInput::parse(&mut attr, &mut p)?;

        Ok(input.output())
    })
}
