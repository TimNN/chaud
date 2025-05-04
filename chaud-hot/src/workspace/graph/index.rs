use crate::cargo::metadata::{Package, PackageName};
use crate::util::CfgInto as _;
use anyhow::{Context as _, Result, bail};
use hashbrown::{HashMap, hash_map};

/// A numeric index / identifier of a crate.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct KrateIdx(u32);

impl KrateIdx {
    #[inline]
    #[must_use]
    pub fn usize(self) -> usize {
        self.0.cfg_into()
    }
}

/// An index of all crates, mapping [`PackageName`] to [`KrateIdx`].
pub struct KrateIndex {
    by_pkg: HashMap<PackageName, KrateIdx>,
}

impl KrateIndex {
    pub fn new(pkgs: &[Package]) -> Result<Self> {
        let mut pkgs: Vec<_> = pkgs.iter().collect();
        // Do not depend on Cargo's output order for determinism.
        pkgs.sort_unstable_by_key(|p| p.name());

        let mut this = Self { by_pkg: HashMap::new() };

        for pkg in pkgs {
            this.insert(pkg).context("Failed to build package index")?;
        }

        Ok(this)
    }

    fn insert(&mut self, pkg: &Package) -> Result<()> {
        let pkg_name = pkg.name().clone();

        let next = KrateIdx(
            self.by_pkg
                .len()
                .try_into()
                .context("Crate idx overflowed `u32`")?,
        );

        match self.by_pkg.entry(pkg_name) {
            hash_map::Entry::Occupied(entry) => bail!("Duplicate package name: {:?}", entry.key()),
            hash_map::Entry::Vacant(entry) => entry.insert(next),
        };

        Ok(())
    }

    pub fn get_pkg(&self, name: &PackageName) -> Option<KrateIdx> {
        self.by_pkg.get(name).copied()
    }
}
