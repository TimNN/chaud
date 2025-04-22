use super::info::KrateInfo;
use core::ops;

/// Mutable data / state of a crate.
pub struct KrateData {
    info: KrateInfo,
}

impl ops::Deref for KrateData {
    type Target = KrateInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}
