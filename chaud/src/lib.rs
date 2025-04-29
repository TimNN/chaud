#![doc = include_str!(env!("README"))]

#[doc(hidden)]
#[cfg(feature = "unsafe-hot-reload")]
pub use chaud_hot as __internal_hot;

pub use chaud_def::Handle;

/// FIXME
#[macro_export]
macro_rules! handle {
    ($($t:tt)*) => {
        $crate::__internal_handle!($($t)*);
    };
}

mod macros;
