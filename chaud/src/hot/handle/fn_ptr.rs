use crate::FnPtrBounds;
use core::ffi::c_void;
use core::mem;
use core::ptr::NonNull;

pub(super) type ErasedFnPtrPointee = c_void;

pub type RawErasedFnPtr = *mut ErasedFnPtrPointee;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ErasedFnPtr {
    /// # Safety
    ///
    /// See the [module][super#safety] docs:
    ///
    /// * Must never change.
    /// * The actual type must be a function pointer implementing
    ///   [`FnPtrBounds`].
    inner: NonNull<ErasedFnPtrPointee>,
}

// SAFETY: The actual type must imlement `FnPtrBounds`, which requires `Send`.
unsafe impl Send for ErasedFnPtr {}

// SAFETY: `ErasedFnPtr` is send and does not allow mutating access.
unsafe impl Sync for ErasedFnPtr {}

impl ErasedFnPtr {
    /// # Safety
    ///
    /// `F` must be a function pointer.
    #[inline]
    #[must_use]
    pub unsafe fn erase<F: FnPtrBounds>(f: F) -> Self {
        // SAFETY: The caller must ensure that `F` is a function pointer (and
        // thus non-null). Aside from that, transmutes from function pointers
        // to pointers are valid.
        let inner = unsafe { transmute_copy_layout_checked::<F, NonNull<ErasedFnPtrPointee>>(f) };

        // SAFETY: Initializing does not count as a change, and the actual type
        // requirements are enforced or need to be upheld by the caller.
        Self { inner }
    }

    /// # Safety
    ///
    /// The passed argument must be a function pointer implementing
    /// [`FnPtrBounds`] (and thus non-null).
    #[inline]
    #[must_use]
    pub(super) unsafe fn from_raw_never_null(raw: RawErasedFnPtr) -> Self {
        // SAFETY: The caller must ensure that `raw` is non-null.
        let inner = unsafe { NonNull::new_unchecked(raw) };

        // SAFETY: Initializing does not count as a change, and the actual type
        // requirements need to be upheld by the caller.
        Self { inner }
    }

    /// # Safety
    ///
    /// The passed argument must either be `null`, or a function pointer
    /// implementing [`FnPtrBounds`].
    #[inline]
    #[must_use]
    pub(super) unsafe fn from_raw_maybe_null(raw: RawErasedFnPtr) -> Option<Self> {
        let inner = NonNull::new(raw)?;

        // SAFETY: Initializing does not count as a change, and the actual type
        // requirements need to be upheld by the caller.
        Some(Self { inner })
    }

    #[inline]
    #[must_use]
    pub(super) fn raw(self) -> RawErasedFnPtr {
        // SAFETY: `self` / `inner` are consumed by value, so `inner` does not
        // change.
        self.inner.as_ptr()
    }

    /// # Safety
    ///
    /// `F` must be the actual type of `self`.
    #[inline]
    #[must_use]
    pub(super) unsafe fn typed<F: FnPtrBounds>(self) -> F {
        // SAFETY: The caller must ensure that `F` is the actual type of `self`,
        // thus transmuting the pointer back to a F (which must be a function
        // pointer) is valid.
        unsafe { transmute_copy_layout_checked::<NonNull<ErasedFnPtrPointee>, F>(self.inner) }
    }
}

/// A wrapper around [`transmute_copy`][mem::transmute_copy] that:
///
/// * Only works on [`Copy`] types.
/// * Enforces that `Src` and `Dst` have the same size and alignment.
///
/// # Safety
///
/// See [`transmute_copy`][mem::transmute_copy].
unsafe fn transmute_copy_layout_checked<Src: Copy, Dst: Copy>(src: Src) -> Dst {
    const {
        assert!(size_of::<Src>() == size_of::<Dst>());
        assert!(align_of::<Src>() == align_of::<Dst>());
    }
    // SAFETY: Must be upheld by the caller.
    unsafe { mem::transmute_copy::<Src, Dst>(&src) }
}
