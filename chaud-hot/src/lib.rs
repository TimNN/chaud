use chaud_def::Handle;
use parking_lot::Once;

mod func;

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

#[repr(transparent)]
pub struct AtomicHandle<H: Handle> {
    f: H::Ptr,
}

impl<H: Handle> AtomicHandle<H> {
    #[must_use]
    pub const fn new(f: H::Ptr) -> Self {
        Self { f }
    }

    #[inline]
    pub fn get(&self) -> H::Ptr {
        self.f
    }
}

#[allow(dead_code)]
pub struct HandleId<H: Handle> {
    init: Once,
    id: &'static str,
    handle: &'static AtomicHandle<H>,
    trampoline: H::Ptr,
}

impl<H: Handle> HandleId<H> {
    #[must_use]
    pub const fn new(
        id: &'static str,
        handle: &'static AtomicHandle<H>,
        trampoline: H::Ptr,
    ) -> Self {
        Self { init: Once::new(), id, handle, trampoline }
    }

    #[inline]
    pub fn register(&'static self) {}
}
