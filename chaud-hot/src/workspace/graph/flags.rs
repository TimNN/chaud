use crate::util::relaxed::RelaxedU8;
use core::cmp;

const DIRTY_BIT: u8 = 0b01;
const PATCH_BIT: u8 = 0b10;

const DIRTY_AND_PATCH_BITS: u8 = 0b11;

/// Tracks various bits about the crate that can be atomically updated.
///
/// * **dirty:** Whether the crate is dirty, i.e., whether any of its files
///   have been modified since the last reload.
/// * **patched:* Whether the crate is patched, i.e., whether the version in
///   its manifest has been modified since the last reload.
#[repr(transparent)]
pub struct KrateFlags {
    inner: RelaxedU8,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ClearDirtyResult {
    Ok = 0,
    UnpatchedDirty = 1,
}

impl ClearDirtyResult {
    #[must_use]
    pub fn merge(self, other: ClearDirtyResult) -> ClearDirtyResult {
        cmp::max(self, other)
    }
}

impl KrateFlags {
    pub(super) const fn new() -> Self {
        // Both bits false.
        Self { inner: RelaxedU8::new(0b00) }
    }

    pub(super) fn needs_patch(&self) -> bool {
        // Need patching if dirty and not yet patched.
        self.inner.load() == DIRTY_BIT
    }

    /// Returns whether the flags changed.
    pub(super) fn mark_dirty(&self) -> bool {
        // PATCH_BIT isn't modified.
        let prev = self.inner.fetch_or(DIRTY_BIT);

        // Changed if DIRTY_BIT was unset previously.
        (prev & DIRTY_BIT) == 0
    }

    pub(super) fn mark_patched(&self) {
        // It's fine to clear DIRTY_BIT when marking as patched, and `swap`
        // might be faster than `fetch_or`.
        let prev = self.inner.swap(PATCH_BIT);

        // Should only mark as patched if dirty and unpatched.
        debug_assert_eq!(prev, DIRTY_BIT);
    }

    pub(super) fn clear_dirty_if_patched(&self) -> ClearDirtyResult {
        let prev = self.inner.compare_exchange(DIRTY_AND_PATCH_BITS, PATCH_BIT);

        // Ok(_) and Err(_) both contain the previous value. We only care about
        // the previous value being "dirty only", which can never be the case
        // for Ok(_) (because then the swap would have failed).
        match prev {
            Err(DIRTY_BIT) => ClearDirtyResult::UnpatchedDirty,
            _ => ClearDirtyResult::Ok,
        }
    }

    pub(super) fn clear_patched(&self) {
        self.inner.fetch_and(!PATCH_BIT);
    }
}
