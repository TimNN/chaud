#![doc = include_str!("../README.md")]
#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    reason = "this crate is an implementation detail, don't be as-strict"
)]
#![allow(dead_code, reason = "TODO: remove once module is fully in use")]

#[doc(no_inline)]
pub use self::func::{Func, FuncStorage};
#[doc(no_inline)]
pub use self::workspace::worker::launch as init;
#[doc(no_inline)]
pub use ctor::declarative::ctor;

mod cargo;
mod dylib;
mod func;
mod util;
mod workspace;
