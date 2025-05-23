use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use parking_lot::{Condvar, Mutex};

static EPOCH: AtomicU32 = AtomicU32::new(0);

static WAIT: (Mutex<()>, Condvar) = (Mutex::new(()), Condvar::new());

#[inline]
pub fn current() -> u32 {
    EPOCH.load(Relaxed)
}

#[inline]
pub fn check(epoch: &mut u32) -> bool {
    let prev = *epoch;
    *epoch = EPOCH.load(Relaxed);
    prev != *epoch
}

#[inline]
pub fn wait(epoch: &mut u32) {
    if check(epoch) {
        return;
    }

    WAIT.1.wait_while(&mut WAIT.0.lock(), |_| !check(epoch));
}

/// Utility to track "init done" in Chaud integration tests.
pub fn track_init() {
    EPOCH.store(u32::MAX, Relaxed);
}

/// Utility to track "init done" in Chaud integration tests.
pub(crate) fn init_done() {
    EPOCH.store(0, Relaxed);
    WAIT.1.notify_all();
}

pub(crate) fn did_reload() {
    EPOCH.fetch_add(1, Relaxed);
    WAIT.1.notify_all();
}
