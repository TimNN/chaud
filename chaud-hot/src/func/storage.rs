use super::{AtomicFnPtr, ErasedFnPtr, Func};
use core::marker::PhantomData;

/// Stores the necessary runtime information about a hot-reloadable function.
#[repr(transparent)]
pub struct FuncStorage<F: Func> {
    _pd: PhantomData<F>,
    /// # Safety
    ///
    /// See the [module][super#safety] docs:
    ///
    /// * The actual type must never change.
    /// * The contained actual type must be `F::Ptr`.
    inner: AtomicFnPtr,
}

impl<F: Func> FuncStorage<F> {
    #[must_use]
    #[expect(clippy::new_without_default, reason = "default would be unused")]
    pub const fn new() -> Self {
        let inner = AtomicFnPtr::new(ErasedFnPtr::erase::<F>(F::actual));

        // SAFETY: Initializing does not count as a change, and the actual type
        // requirements are enforced or need to be upheld by the caller.
        Self { _pd: PhantomData, inner }
    }

    #[inline]
    #[must_use]
    pub fn get(&'static self) -> F::Ptr {
        let erased = self.inner.load_relaxed();

        // SAFETY: `inner`'s actual type is `F::Ptr`.
        unsafe { erased.typed::<F::Ptr>() }
    }

    pub fn update(&'static self) {
        let erased = ErasedFnPtr::erase::<F>(F::actual);

        // SAFETY: `inner`'s actual type is `F::Ptr`.
        unsafe { self.inner.store_relaxed(erased) };

        log::debug!("Updated {:?} to {:?}", F::NAME, erased);
    }
}
