//! A utility for publishing some value from one thread, and making the most
//! recently published value available to some other thread, with support
//! for waiting for the next published value.

use parking_lot::{Condvar, Mutex};
use std::sync::Arc;

struct Inner<T> {
    val: Mutex<T>,
    cond: Condvar,
}

pub struct LatestPublisher<T> {
    inner: Arc<Inner<T>>,
}

pub struct LatestReader<T> {
    inner: Arc<Inner<T>>,
    latest: T,
}

pub fn make_latest<T: Copy>(initial: T) -> (LatestPublisher<T>, LatestReader<T>) {
    let i = Arc::new(Inner { val: Mutex::new(initial), cond: Condvar::new() });

    (
        LatestPublisher { inner: i.clone() },
        LatestReader { inner: i, latest: initial },
    )
}

impl<T> LatestPublisher<T> {
    pub fn publish(&mut self, val: T) {
        *self.inner.val.lock() = val;
        self.inner.cond.notify_one();
    }
}

impl<T: Copy + PartialEq> LatestReader<T> {
    pub fn wait(&mut self) -> T {
        let mut guard = self.inner.val.lock();

        self.inner
            .cond
            .wait_while(&mut guard, |val| *val == self.latest);

        self.latest = *guard;
        self.latest
    }

    pub fn check(&mut self) -> Option<T> {
        let guard = self.inner.val.lock();

        match *guard == self.latest {
            true => None,
            false => {
                self.latest = *guard;
                Some(self.latest)
            }
        }
    }
}
