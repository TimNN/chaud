use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

static COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn counter() -> u32 {
    COUNTER.fetch_add(1, Relaxed)
}

#[inline(never)]
pub fn initially_unused() -> u32 {
    42
}
