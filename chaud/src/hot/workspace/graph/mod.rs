//! Data structures modeling the worspace's crate graph and configuration.
//!
//! Contains both immutable "info" and mutable "data".
//!
//! # Code Style
//!
//! * Methods should be `pub(super)` until they are needed outside this module.

pub use self::def::*;
pub use self::index::*;

mod data;
mod def;
mod env;
mod index;
mod info;
mod paths;
