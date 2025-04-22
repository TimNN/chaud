use super::KrateIdx;
use super::data::KrateData;
use crate::hot::util::{CfgInto as _, OrderedTopo};
use anyhow::{Context as _, Result};
use core::ops;

/// A numeric index / identifier of a dylib crate.
///
/// Assigned based on the topological order of the crate graph, such that a
/// dependency always has a smaller index than its dependents.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DylibIdx(u32);

/// Mapping from [`DylibIdx`] to [`KrateIdx`].
pub struct DylibMap {
    inner: Box<[KrateIdx]>,
}

impl ops::Index<DylibIdx> for DylibMap {
    type Output = KrateIdx;

    #[expect(
        clippy::indexing_slicing,
        reason = "Dylib indices are assumed to be valid."
    )]
    fn index(&self, idx: DylibIdx) -> &Self::Output {
        &self.inner[idx.usize()]
    }
}

impl DylibIdx {
    #[inline]
    #[must_use]
    pub fn usize(self) -> usize {
        self.0.cfg_into()
    }

    pub(super) fn assign(krates: &mut [KrateData]) -> Result<DylibMap> {
        assign_inner(krates).context("Failed to assign dylib indices")
    }
}

impl DylibMap {
    pub(super) fn len(&self) -> usize {
        self.inner.len()
    }
}

#[expect(
    clippy::indexing_slicing,
    reason = "crate indices are assumed to be valid"
)]
fn assign_inner(krates: &mut [KrateData]) -> Result<DylibMap> {
    let mut topo = OrderedTopo::new();

    for krate in krates.iter() {
        for dep in krate.deps() {
            topo.add_dependency(*dep, krate.idx());
        }
    }

    let order = topo.sort()?;

    let mut mapping = vec![];
    for idx in order {
        let krate = &mut krates[idx.usize()];
        if !krate.is_dylib() {
            continue;
        }

        let next = mapping
            .len()
            .try_into()
            .context("Crate idx overflowed `u32`")?;
        krate.assign_dylib_idx(DylibIdx(next));
        mapping.push(idx);
    }

    Ok(DylibMap { inner: mapping.into_boxed_slice() })
}
