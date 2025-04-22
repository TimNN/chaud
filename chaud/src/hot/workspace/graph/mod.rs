//! Data structures modeling the worspace's crate graph and configuration.
//!
//! Contains both immutable "info" and mutable "data".
//!
//! # Code Style
//!
//! * Methods should be `pub(super)` until they are needed outside this module.

pub use self::data::*;
pub use self::def::*;
pub use self::dylib::*;
pub use self::index::*;
pub use self::info::*;

mod data;
mod def;
mod dylib;
mod env;
mod flags;
mod index;
mod info;
mod paths;
