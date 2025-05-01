#![doc = include_str!("../README.md")]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::wildcard_imports,
    clippy::indexing_slicing,
    clippy::enum_glob_use,
    clippy::expect_used,
    clippy::panic
)]

use self::err::Error;
use self::input::Input;
use self::output::output;
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

#[expect(clippy::needless_return)]
#[proc_macro_attribute]
pub fn hot(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut attr = Parser::new(attr);
    let mut p = Parser::new(item.clone());

    return Error::reporting(item.clone(), || {
        let input = Input::parse(&mut attr, &mut p)?;

        Ok(output(&input))
    });
}
