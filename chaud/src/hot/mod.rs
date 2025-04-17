use crate::FnPtrBounds;

#[derive(Copy, Clone)]
pub struct TypedHandle<F: FnPtrBounds> {
    f: F,
}

impl<F: FnPtrBounds> TypedHandle<F> {
    /// # Safety
    ///
    /// `F` must be a function pointer.
    pub unsafe fn create(f: F) -> Self {
        Self { f }
    }

    pub fn get(self) -> F {
        self.f
    }
}
