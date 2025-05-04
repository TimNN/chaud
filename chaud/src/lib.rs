#![forbid(unsafe_code)]
#![doc = include_str!(env!("README"))]

#[doc(hidden)]
#[cfg(feature = "unsafe-hot-reload")]
pub use chaud_hot as __internal;

/// Marks a function that should be hot-reloaded.
///
/// When hot-reloading is **disabled**, this is essentially a no-op.
///
/// The name of the function must be unique within the current module.
///
/// There are some limitations on the supported syntax:
///
/// * Only plain `fn`s are supported.
///   * `const`, `unsafe`, `extern` and `async` are not supported.
///
/// * All function parameters must be written as `identifier: Type`.
///   * Patterns instead of identifiers are not supported.
///   * Shorthand-self (e.g. `&self` without an explicit type) is not supported.
///
/// * Methods are supported, however:
///   * They must be annotated with `#[chaud::hot(self)]`.
///   * The first parameter must **not** actually mention "self", it must still
///     be written as, e.g., `this: &ExplitTypeOfSelf`.
///   * They cannot refer to any types or generic parameters from the
///     surrounding `impl` block
///
/// * The only supported generic parameters are lifetime parameters.
///   * Bounds on the lifetime parameters are not supported. `fn foo<'a, 'b>` is
///     allowed, `fn foo<'a, 'b: 'a>` is not.
///
/// Some of these simply haven't been implemented yet. Others due to needing to
/// represent the function as a function pointer.
///
/// ## Examples
///
/// ```
/// struct Foo<T> { val: T }
///
/// impl Foo<String> {
///     #[chaud::hot(self)]
///     pub fn append<'a>(this: &'a mut Foo<String>, s: &str) -> &'a str {
///         this.val.push_str(s);
///         &this.val
///     }
/// }
///
/// let mut foo = Foo { val: String::new() };
/// foo.append("Hello");
/// ```
pub use chaud_macros::hot;

/// Marks a static that should persist / be shared across hot-reloads.
///
/// When hot-reloading is **disabled**, this is a no-op.
///
/// The name of the static must be unique within the current module.
///
/// The full `static` syntax should be supported. If you encounter code that
/// fails if `#[chaud::persist]` is applied, please report a bug.
///
/// ## Examples
///
/// ```
/// # use core::sync::atomic::AtomicU32;
/// #[chaud::persist]
/// pub static STATE: AtomicU32 = AtomicU32::new(42);
/// ```
pub use chaud_macros::persist;

/// Initializes Chaud.
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
