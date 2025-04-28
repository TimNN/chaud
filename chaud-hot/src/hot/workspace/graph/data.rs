use super::ClearDirtyResult;
use super::dylib::DylibIdx;
use super::flags::KrateFlags;
use super::info::KrateInfo;
use crate::hot::util::relaxed::RelaxedBool;
use core::{fmt, ops};

/// Mutable data / state of a crate.
pub struct KrateData {
    info: KrateInfo,
    dylib: Option<DylibIdx>,
    watched: RelaxedBool,
    flags: KrateFlags,
}

impl ops::Deref for KrateData {
    type Target = KrateInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl fmt::Display for KrateData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.info)
    }
}

impl KrateData {
    pub(super) fn new(info: KrateInfo) -> Self {
        Self {
            info,
            dylib: None,
            watched: RelaxedBool::new(false),
            flags: KrateFlags::new(),
        }
    }

    pub fn dylib(&self) -> Option<DylibIdx> {
        debug_assert_eq!(self.is_dylib(), self.dylib.is_some());
        self.dylib
    }

    pub fn needs_patch(&self) -> bool {
        self.flags.needs_patch()
    }

    pub fn mark_patched(&self) {
        self.flags.mark_patched();
    }

    pub fn mark_dirty(&self) {
        if self.flags.mark_dirty() {
            log::debug!("Mark dirty: {self}");
        }
    }

    pub fn clear_patched(&self) {
        self.flags.clear_patched();
    }

    pub fn clear_dirty_if_patched(&self) -> ClearDirtyResult {
        self.flags.clear_dirty_if_patched()
    }

    pub(super) fn watch(&self) -> bool {
        self.watched.swap(true)
    }

    pub(super) fn assign_dylib_idx(&mut self, dylib: DylibIdx) {
        debug_assert!(self.dylib.is_none());
        debug_assert!(self.is_dylib());
        self.dylib = Some(dylib);
    }
}
