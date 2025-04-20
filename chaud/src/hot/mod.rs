//! Hot-reloading functionality.
#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    reason = "don't want this to trigger for private items"
)]
#![allow(dead_code, reason = "TODO: remove once module is fully in use")]

pub use self::handle::TypedHandle;
pub use self::handles::create_handle;

mod cargo;
mod dylib;
mod handle;
mod handles;
mod util;
