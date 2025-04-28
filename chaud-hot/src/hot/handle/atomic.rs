use super::{ErasedFnPtr, ErasedFnPtrPointee};
use core::sync::atomic::{AtomicPtr, Ordering::Relaxed};

#[repr(transparent)]
pub struct AtomicFnPtr {
    /// # Safety
    ///
    /// See the [module][super#safety] docs:
    ///
    /// * The actual type must never change.
    /// * The actual type must be a function pointer implementing
    ///   [`crate::FnPtr`].
    inner: AtomicPtr<ErasedFnPtrPointee>,
}

impl AtomicFnPtr {
    #[inline]
    #[must_use]
    pub(super) fn new(f: ErasedFnPtr) -> Self {
        // SAFETY: Initializing defines the actual type stored. The other
        // requirements are enforced by `ErasedFnPtr`.
        Self { inner: AtomicPtr::new(f.raw()) }
    }

    /// # SAFETY
    ///
    /// The passed argument must have the same actual type as `self`.
    #[inline]
    pub(super) unsafe fn store_relaxed(&self, f: ErasedFnPtr) {
        // SAFETY: The caller must ensure that `f` has the same actual type as
        // `self`.
        self.inner.store(f.raw(), Relaxed);
    }

    #[inline]
    #[must_use]
    pub(super) fn load_relaxed(&self) -> ErasedFnPtr {
        let inner = self.inner.load(Relaxed);

        // SAFETY: The actual type stored is a function pointer implementing
        // `FnPtr`.
        unsafe { ErasedFnPtr::from_raw_never_null(inner) }
    }
}
