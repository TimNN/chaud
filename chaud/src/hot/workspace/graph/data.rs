use super::flags::KrateFlags;
use super::info::KrateInfo;
use crate::hot::util::relaxed::RelaxedBool;
use core::{fmt, ops};

/// Mutable data / state of a crate.
pub struct KrateData {
    info: KrateInfo,
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
        write!(f, "`{}`", self.info)
    }
}

impl KrateData {
    pub(super) fn new(info: KrateInfo) -> Self {
        Self {
            info,
            watched: RelaxedBool::new(false),
            flags: KrateFlags::new(),
        }
    }
}
