//! Interact with the hot-reloading lifecycle.

#[cfg(feature = "unsafe-hot-reload")]
mod imp {
    pub use chaud_hot::cycle::*;

    pub type Epoch = u32;
}

#[cfg(not(feature = "unsafe-hot-reload"))]
mod imp {
    pub type Epoch = ();

    #[inline]
    pub fn current() {}

    #[inline]
    pub fn check(_: &mut ()) -> bool {
        false
    }

    #[inline]
    #[track_caller]
    pub fn wait(_: &mut ()) {
        panic!("Cannot wait for hot-reload if hot-reloading is disabled.")
    }
}

/// Check for hot reloads since `self` was created.
#[derive(Copy, Clone)]
pub struct Check {
    epoch: imp::Epoch,
}

/// Track hot reloads since the last time any method on `self` was called.
#[derive(Clone)]
pub struct Track {
    epoch: imp::Epoch,
}

impl Check {
    /// Start checking for future hot-reloads.
    #[inline]
    #[must_use]
    #[expect(clippy::new_without_default, reason = "stateful")]
    pub fn new() -> Self {
        Self { epoch: imp::current() }
    }

    /// Returns `true` if a reload has happened (since `self` was created).
    #[inline]
    #[must_use]
    pub fn did_reload(mut self) -> bool {
        imp::check(&mut self.epoch)
    }

    /// Wait for the next reload (since `self` was created).
    ///
    /// Returns immediately such a reload has already occured.
    ///
    /// # Panics
    ///
    /// If hot-reloading is disabled (as this function would block indefinitely
    /// in that case).
    #[inline]
    #[track_caller]
    pub fn wait(mut self) {
        imp::wait(&mut self.epoch);
    }
}

impl Track {
    /// Start tracking future hot-reloads.
    #[inline]
    #[must_use]
    #[expect(clippy::new_without_default, reason = "stateful")]
    pub fn new() -> Self {
        Self { epoch: imp::current() }
    }

    /// Returns `true` if a reload has happened (since `self` was last called).
    #[inline]
    #[must_use]
    pub fn did_reload(&mut self) -> bool {
        imp::check(&mut self.epoch)
    }

    /// Wait for the next reload (since `self` was last called).
    ///
    /// Returns immediately such a reload has already occured.
    ///
    /// # Panics
    ///
    /// If hot-reloading is disabled (as this function would block indefinitely
    /// in that case).
    #[inline]
    #[track_caller]
    pub fn wait(&mut self) {
        imp::wait(&mut self.epoch);
    }
}
