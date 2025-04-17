use super::{AtomicFnPtr, ErasedFnPtr};

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
    ///   [`FnPtrBounds`].
    inner: &'static AtomicFnPtr,
}

impl ErasedHandle {
    #[inline]
    #[must_use]
    pub fn new(f: ErasedFnPtr) -> Self {
        let inner = Box::leak(Box::new(AtomicFnPtr::new(f)));

        // SAFETY: Initializing does not count as a change. The other
        // requirements are enforced by `ErasedFnPtr`.
        Self { inner }
    }

    /// # Safety
    ///
    /// FIXME
    #[inline]
    pub(super) fn set(self, f: ErasedFnPtr) {
        // SAFETY: FIXME
        unsafe { self.inner.store_relaxed(f) };
    }

    #[inline]
    #[must_use]
    pub(super) fn get(self) -> ErasedFnPtr {
        self.inner.load_relaxed()
    }
}
