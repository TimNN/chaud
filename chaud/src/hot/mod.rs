//! Hot-reloading functionality.
#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    reason = "don't want this to trigger for private items"
)]
#![allow(dead_code, reason = "TODO: remove once module is fully in use")]

pub use self::handle::TypedHandle;
use self::handle::{ErasedFnPtr, ErasedHandle};

mod cargo;
mod handle;
mod util;

/// # Safety
///
/// `F` must be a function pointer.
#[inline]
pub unsafe fn create_handle<F: crate::FnPtrBounds>(f: F) -> TypedHandle<F> {
    // TODO: When implementing proper handle creation, skip registration for
    // #[cfg(test)].

    // SAFETY: `f` is a function pointer.
    let erased = unsafe { ErasedFnPtr::erase(f) };

    let handle = ErasedHandle::new(erased);

    // SAFETY: `handle` was created from `F`.
    unsafe { TypedHandle::<F>::new(handle) }
}
