use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

#[chaud::hot]
pub fn version() -> u32 {
    2001 // VERSION
}

#[chaud::hot]
pub fn leaf_version() -> u32 {
    leaf::version()
}

#[chaud::hot]
pub fn counters() -> (u32, u32, u32) {
    #[chaud::persist]
    static SINCE_START: AtomicU32 = AtomicU32::new(0);

    static SINCE_RELOAD: AtomicU32 = AtomicU32::new(0);

    (
        SINCE_RELOAD.fetch_add(1, Relaxed),
        SINCE_START.fetch_add(1, Relaxed),
        cold::counter(),
    )
}
