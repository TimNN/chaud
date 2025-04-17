#![allow(
    clippy::multiple_unsafe_ops_per_block,
    clippy::undocumented_unsafe_blocks
)]

use super::{ErasedFnPtr, ErasedHandle, TypedHandle};

#[test]
fn end_to_end() {
    fn alpha(x: u32) -> u32 {
        x + 1
    }
    fn beta(x: u32) -> u32 {
        x * 2
    }

    unsafe {
        let erased_alpha = ErasedFnPtr::erase(alpha as fn(u32) -> u32);
        let erased_beta = ErasedFnPtr::erase(beta as fn(u32) -> u32);

        let handle = ErasedHandle::new(erased_alpha);
        let typed = TypedHandle::<fn(u32) -> u32>::new(handle);

        assert_eq!(typed.get()(2), 3);

        handle.set(erased_beta);

        assert_eq!(typed.get()(2), 4);
    }
}
