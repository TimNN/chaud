#![cfg_attr(not(feature = "unsafe-hot-reload"), forbid(unsafe_code))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!(env!("README"))]

/// A trait representing the bounds expected to hold for a function pointer.
pub trait FnPtrBounds: Copy + Send + Sized + 'static {}

// NOTE: Ideally we'd enforce that this is _only_ implemented for function
// pointers, but that is not possible without the unstable `core::marker::FnPtr`
// trait.
//
// Manually implementing this trait for function pointers of different arities
// does not work due to lifetime problems.
impl<F: Copy + Send + Sized + 'static> FnPtrBounds for F {}

/// Provides access to a function pointer whose definition may change at runtime
/// if hot-reloading is enabled.
///
/// # Access
///
/// See [`Handle::get`] for how to access the function pointer represented by a
/// handle.
///
/// # Creation
///
/// A handle is created by calling one of the `Self::create<N><kw>` functions,
/// passing a pointer to the function you want the handle to represent as an
/// argument. The separate functions are necessary due to limitations of Rust's
/// type system.
///
/// The `<N>` specifies the number of arguments your function takes.
///
/// The `<kw>` part is optional and describes any additional keywords your
/// function has. Currently supported are "`u`" for `unsafe fn` and "`e`" for
/// `extern "C" fn` (as well as "`ue`" to combine the two). These are only
/// available when the `create-unsafe` and/or `create-extern` features are
/// enabled.
///
/// ```
/// # if cfg!(feature = "unsafe-hot-reload") { return }
/// #
/// # use chaud::Handle;
/// #
/// fn do_some_math(a: u32, b: u32) -> u32 { a + b }
///
/// let handle = Handle::create2(do_some_math);
/// ```
///
/// ## Failures
///
/// Handle creation always succeeds.
///
/// If hot-reloading is **disabled**, a handle simply wraps the provided
/// function pointer. Handle creation is essentiall a no-op.
///
/// If hot-reloading is **enabled**, it is possible that creating a _shared_
/// handle fails internally. In that case, a _non-shared_ handle is returned
/// instead, which will not be hot-reloaded.
///
/// Creating a _non-shared_ handle always succeeds[^1] and [`get`][Self::get]
/// will always return the function pointer provided when the handle was
/// created.
///
/// As explained in the [Logging][crate#logging] section, it is essential
/// that you enable logging for this crate to be informed about any failures
/// that do occur.
///
/// ## Performance
///
/// For maximum performance, create only one handle for each function you wish
/// to hot-reload and reuse it. Copying a handle does not affect performance.
///
/// Repeatedly calling `create` for the same function is usually fine as well,
/// since handles are cached internally.
///
/// [^1]: Unless there is a memory allocation error, in which case Rust's
/// [allocation error handling][std::alloc::handle_alloc_error] applies.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Handle<F: FnPtrBounds> {
    #[cfg(not(feature = "unsafe-hot-reload"))]
    repr: F,
    #[cfg(feature = "unsafe-hot-reload")]
    repr: crate::hot::TypedHandle<F>,
}

// This is in a module so that it is ordered before the items from `mod create`
// below.
mod get {
    use crate::{FnPtrBounds, Handle};

    /// Returns the function pointer currently associated with this handle.
    ///
    /// Usually you should call the returned function pointer immediately,
    /// without storing it anywhere.
    ///
    /// ```
    /// # if cfg!(feature = "unsafe-hot-reload") { return }
    /// #
    /// # use chaud::Handle;
    /// #
    /// fn do_some_math(a: u32, b: u32) -> u32 { a + b }
    ///
    /// let handle = Handle::create2(do_some_math);
    ///
    /// assert_eq!(handle.get()(1, 2), 3);
    /// ```
    ///
    /// # Failures
    ///
    /// This method cannot fail. It will always return the function pointer
    /// currently associated with this handle.
    ///
    /// If hot-reloading is **enabled**, any issues with reloading the handle
    /// will get [logged][crate#logging].
    ///
    /// # Performance
    ///
    /// If hot-reloading is **disabled**, this function simply returns the
    /// wrapped function pointer. `get` is essentially a no-op.
    ///
    /// If hot-reloading is **enabled**, this function perfoms an
    /// [atomic][core::sync::atomic] load to retrieve the most-recently loaded function
    /// pointer.
    impl<F: FnPtrBounds> Handle<F> {
        #[inline]
        #[must_use]
        pub fn get(self) -> F {
            #[cfg(not(feature = "unsafe-hot-reload"))]
            return self.repr;

            #[cfg(feature = "unsafe-hot-reload")]
            return self.repr.get();
        }
    }
}

// This is below `get` so that `get` shows up first in the docs.
mod create;

#[cfg(feature = "unsafe-hot-reload")]
mod hot;
