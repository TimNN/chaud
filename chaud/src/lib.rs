#![forbid(unsafe_code)]
#![doc = include_str!(env!("README"))]

#[doc(hidden)]
#[cfg(feature = "unsafe-hot-reload")]
pub use chaud_hot as __internal;

pub use chaud_macros::hot;

/// Initialize Chaud.
///
/// When hot-reloading is **disabled**, this is a no-op.
///
/// When hot-reloading is **enabled**, this starts the worker thread and returns
/// afterwards.
///
/// As described in the [Logging][crate#logging] section, enabling logging is
/// essential to receive notifications about any failures that may occur while
/// using this crate.
///
/// To ensure basic visibility into such failures, this function will
/// automatically configure a minimal logger if the [`log`](https://docs.rs/log)
/// crate has not already been initialized at the time it is called.
pub fn init() {
    #[cfg(feature = "unsafe-hot-reload")]
    __internal::init();
}
