/// # Safety
///
/// [`Self::Ptr`] must be a function pointer.
pub unsafe trait Handle: Copy + 'static {
    type Ptr: Copy;

    fn create() -> Self;

    fn get(self) -> Self::Ptr;

    #[doc(hidden)]
    fn __chaud_internal_do_not_implement();
}
