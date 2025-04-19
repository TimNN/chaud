use std::sync::{LockResult, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

/// A wrapper around [`std::sync::Mutex`] that ignores all poison.
#[repr(transparent)]
pub struct Mutex<T> {
    inner: std::sync::Mutex<T>,
}

impl<T> Mutex<T> {
    #[inline]
    #[must_use]
    pub const fn new(val: T) -> Self {
        Self { inner: std::sync::Mutex::new(val) }
    }

    #[inline]
    pub fn lock(&self) -> MutexGuard<T> {
        let guard = self.inner.lock();
        ignore_poison(guard)
    }
}

/// A wrapper around [`std::sync::RwLock`] that ignores all poison.
#[repr(transparent)]
pub struct RwLock<T> {
    inner: std::sync::RwLock<T>,
}

impl<T> RwLock<T> {
    #[inline]
    #[must_use]
    pub const fn new(val: T) -> Self {
        Self { inner: std::sync::RwLock::new(val) }
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<T> {
        let guard = self.inner.read();
        ignore_poison(guard)
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<T> {
        let guard = self.inner.write();
        ignore_poison(guard)
    }
}

#[inline]
fn ignore_poison<T>(result: LockResult<T>) -> T {
    match result {
        Ok(t) => t,
        Err(p) => p.into_inner(),
    }
}
