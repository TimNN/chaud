#![cfg_attr(not(feature = "unsafe-hot-reload"), forbid(unsafe_code))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(not(unix))]
compile_error!("Hot-reloading is only supported on `cfg(unix)` platforms");

mod func;

#[cfg(all(unix, feature = "unsafe-hot-reload"))]
mod hot;

/// A trait implemented for function pointers.
///
/// # Safety
///
/// Guaranteed to be only implemented for function pointers.
#[expect(private_bounds, reason = "sealed")]
pub trait FnPtr: func::Sealed + Copy + Send + Sized + 'static {}

/// A trait implemented for [`Fn`]s.
///
/// `Ptr` is the corresponding function pointer type.
#[expect(private_bounds, reason = "sealed")]
pub trait Func<Ptr: FnPtr>: func::Sealed<Ptr> {}

/// Provides access to a function pointer whose definition may change at
/// runtime.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Handle<F: FnPtr> {
    #[cfg(not(all(unix, feature = "unsafe-hot-reload")))]
    repr: F,
    #[cfg(all(unix, feature = "unsafe-hot-reload"))]
    repr: crate::hot::TypedHandle<F>,
}

impl<F: FnPtr> Handle<F> {
    /// Creates a new [`Handle`].
    ///
    /// Prefer using the [`handle!`] macro.
    ///
    /// If you call this method manually, pass the same expression for both
    /// parameters. The expression should be a [function item][fn-item].
    ///
    /// Hot-reloading will only work if the function item is defined in a
    /// library crate that has the `"dylib"` [`crate-type`] configured.
    /// Hot-reloading is unlikely to work if the function item includes any
    /// generic const or type parameters.
    ///
    /// ```
    /// # use chaud::Handle;
    /// #
    /// // In real code, `do_some_math` would be defined in a different crate.
    /// fn do_some_math(a: u32, b: u32) -> u32 { a + b }
    ///
    /// let handle = Handle::new(do_some_math, do_some_math);
    /// ```
    ///
    /// # Failures
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
    /// Creating a _non-shared_ handle always succeeds[^1] and
    /// [`get`][Self::get] will always return the function pointer provided when
    /// the handle was created.
    ///
    /// As explained in the [Logging][crate#logging] section, it is essential
    /// that you enable logging for this crate to be informed about any failures
    /// that do occur.
    ///
    /// # Performance
    ///
    /// For maximum performance, create only one handle for each function you
    /// wish to hot-reload and reuse it. Copying a handle does not affect
    /// performance.
    ///
    /// Repeatedly creating a handle for the same function is usually fine as
    /// well, since handles are cached internally.
    ///
    /// # Safety
    ///
    /// If hot-reloading is **enabled** using the `unsafe-hot-reload` feature,
    /// it is undefined behavior if the two parameters evaluate to different
    /// [function items][fn-item].
    ///
    /// [^1]: Unless there is a memory allocation error, in which case Rust's
    /// [allocation error handling][std::alloc::handle_alloc_error] applies.
    ///
    /// [fn-item]: https://doc.rust-lang.org/reference/types/function-item.html
    /// [`crate-type`]: https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-crate-type-field
    #[inline]
    pub fn new<Item: Func<F>>(_: Item, f: F) -> Self {
        #[cfg(not(all(unix, feature = "unsafe-hot-reload")))]
        return Self { repr: f };

        #[cfg(all(unix, feature = "unsafe-hot-reload"))]
        return Self {
            repr: crate::hot::create_handle(core::any::type_name::<Item>(), f),
        };
    }

    /// Returns the function pointer currently associated with this handle.
    ///
    /// Usually you should call the returned function pointer immediately,
    /// without storing it anywhere.
    ///
    /// ```
    /// # use chaud::handle;
    /// #
    /// fn do_some_math(a: u32, b: u32) -> u32 { a + b }
    ///
    /// let handle = handle!(do_some_math);
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
    #[inline]
    #[must_use]
    pub fn get(self) -> F {
        #[cfg(not(all(unix, feature = "unsafe-hot-reload")))]
        return self.repr;

        #[cfg(all(unix, feature = "unsafe-hot-reload"))]
        return self.repr.get();
    }
}

/// Creates a new [`Handle`].
///
/// The argument passed to this macro should be a [function item].
///
/// Hot-reloading will only work if the function item is defined in a library
/// crate that has the `"dylib"` [`crate-type`] configured. Hot-reloading is
/// unlikely to work if the function item includes any generic const or type
/// parameters.
///
/// ```
/// # use chaud::handle;
/// #
/// // In real code, `do_some_math` would be defined in a different crate.
/// fn do_some_math(a: u32, b: u32) -> u32 { a + b }
///
/// let handle = handle!(do_some_math);
/// ```
///
/// See [`Handle::new`] for further details.
///
/// [function item]: https://doc.rust-lang.org/reference/types/function-item.html
/// [`crate-type`]: https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-crate-type-field
#[macro_export]
macro_rules! handle {
    ($f:expr) => {
        $crate::Handle::new($f, $f)
    };
}
