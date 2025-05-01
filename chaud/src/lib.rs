#![doc = include_str!(env!("README"))]

#[doc(hidden)]
#[cfg(feature = "unsafe-hot-reload")]
pub use chaud_hot as __internal;

pub use chaud_macros::hot;
