use super::ErasedHandle;
use crate::FnPtrBounds;
use core::marker::PhantomData;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TypedHandle<F: FnPtrBounds> {
    _pd: PhantomData<F>,
    /// # Safety
    ///
    /// See the [module][super#safety] docs:
    ///
    /// * Must never change.
    /// * The actual type must be `F`, which must be function pointer.
    inner: ErasedHandle,
}

impl<F: FnPtrBounds> TypedHandle<F> {
    /// # Safety
    ///
    /// The passed argument's actual type must be `F`.
    #[inline]
    #[must_use]
    pub unsafe fn new(h: ErasedHandle) -> Self {
        // SAFETY: Initializing does not count as a change, and the actual type
        // requirements are enforced or need to be upheld by the caller.
        Self { _pd: PhantomData, inner: h }
    }

    #[inline]
    #[must_use]
    pub fn get(self) -> F {
        let erased = self.inner.get();

        // SAFETY: `inner`'s actual type is `F`.
        unsafe { erased.typed::<F>() }
    }
}
