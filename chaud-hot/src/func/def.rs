/// A trait that aggregates information about hot-reloadable functions.
///
/// This is implemented for macro-generated synthetic types. The `Self` type
/// has little meaning.
///
/// # Safety
///
/// [`Self::Ptr`] must be a function pointer.
#[expect(non_upper_case_globals, reason = "function-like usage")]
pub unsafe trait Func {
    /// The function pointer type corresponding to this function.
    type Ptr: FnPtrLike;

    /// A name describing this function.
    ///
    /// Only for human usage.
    const NAME: &str;

    /// The "actual" implementation of this function.
    ///
    /// This is the function as defind by the user.
    const actual: Self::Ptr;
}

/// A trait implemented for types that might be function pointers.
///
/// This is primarily used to aggregate and enforce the bounds we care about.
///
/// Ideally we'd use [`core::marker::FnPtr`] (if something like it is ever
/// stabilized).
pub trait FnPtrLike: Copy + Send + Sync + 'static {}

impl<F: Copy + Send + Sync + 'static> FnPtrLike for F {}
