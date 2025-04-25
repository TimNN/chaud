use super::{AtomicFnPtr, ErasedFnPtr};
use core::fmt;
use core::ptr;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ErasedHandle {
    /// # Safety
    ///
    /// See the [module][super#safety] docs:
    ///
    /// * `inner` itself must never change. (The value stored in the
    ///   [`AtomicFnPtr`] may change).
    /// * The actual type must be a function pointer implementing
    ///   [`crate::FnPtr`].
    inner: &'static AtomicFnPtr,
}

impl PartialEq for ErasedHandle {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner, other.inner)
    }
}

impl fmt::Debug for ErasedHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ErasedHandle")
            .field(&ptr::from_ref(self.inner))
            .finish()
    }
}

impl ErasedHandle {
    #[inline]
    #[must_use]
    pub fn new(f: ErasedFnPtr) -> Self {
        let inner = Box::leak(Box::new(AtomicFnPtr::new(f)));

        #[cfg(miri)]
        crate::hot::util::miri::intentionally_leaked(inner);

        // SAFETY: Initializing does not count as a change. The other
        // requirements are enforced by `ErasedFnPtr`.
        Self { inner }
    }

    /// # Safety
    ///
    /// The passed argument must have the same actual type as `self`.
    #[inline]
    pub(super) fn set(self, f: ErasedFnPtr) {
        // SAFETY: The caller must ensure that `f` has the same actual type as
        // `self`.
        // SAFETY: `self` / `inner` are consumed by value, so `inner` does not
        // change.
        unsafe { self.inner.store_relaxed(f) };
    }

    #[inline]
    #[must_use]
    pub(super) fn get(self) -> ErasedFnPtr {
        // SAFETY: `self` / `inner` are consumed by value, so `inner` does not
        // change.
        self.inner.load_relaxed()
    }
}
