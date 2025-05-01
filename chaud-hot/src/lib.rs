use core::marker::PhantomData;

/// # Safety
///
/// [`Self::Ptr`] must be a function pointer.
#[allow(non_upper_case_globals)]
pub unsafe trait Func {
    type Ptr;

    const ID: &str;

    const actual: Self::Ptr;

    const init: Self::Ptr;

    const jump: Self::Ptr;
}

pub struct AtomicFunc<F: Func> {
    _pd: PhantomData<F>,
}

impl<F: Func> AtomicFunc<F> {
    #[must_use]
    #[expect(clippy::new_without_default, reason = "intentional")]
    pub const fn new() -> Self {
        Self { _pd: PhantomData }
    }

    #[must_use]
    pub fn get(&self) -> F::Ptr {
        todo!();
    }

    #[must_use]
    pub fn init(&self) -> F::Ptr {
        todo!();
    }
}
