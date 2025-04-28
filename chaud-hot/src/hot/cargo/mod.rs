pub use self::build::*;

// FIXME(https://github.com/rust-lang/rust-clippy/issues/14697): False-positive
// that cannot even be `#[allow]`ed.
#[path = "build_.rs"]
mod build;
pub mod metadata;
