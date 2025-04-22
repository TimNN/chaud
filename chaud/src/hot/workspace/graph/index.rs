use crate::hot::cargo::metadata::{KrateName, Package, PackageName};
use crate::hot::util::CfgInto as _;
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

/// An index of all crates, mapping [`PackageName`]/[`KrateName`] to
/// [`KrateIdx`].
pub struct KrateIndex {
    by_pkg: HashMap<PackageName, KrateIdx>,
    by_krate: HashMap<KrateName<'static>, KrateIdx>,
}

impl KrateIndex {
    pub fn new(pkgs: &[Package]) -> Result<Self> {
        let mut names: Vec<_> = pkgs.iter().map(|p| p.name().clone()).collect();
        // Do not depend on Cargo's output order for determinism.
        names.sort_unstable();

        let mut this = Self { by_pkg: HashMap::new(), by_krate: HashMap::new() };

        for name in names {
            this.insert(name).context("Failed to build package index")?;
        }

        Ok(this)
    }

    fn insert(&mut self, pkg_name: PackageName) -> Result<()> {
        let krate_name = pkg_name.to_krate();

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

        match self.by_krate.entry(krate_name) {
            hash_map::Entry::Occupied(entry) => bail!("Duplicate krate name: {:?}", entry.key()),
            hash_map::Entry::Vacant(entry) => entry.insert(next),
        };

        Ok(())
    }

    pub fn get_pkg(&self, name: &PackageName) -> Option<KrateIdx> {
        self.by_pkg.get(name).copied()
    }

    pub fn get_krate(&self, name: &KrateName<'_>) -> Option<KrateIdx> {
        self.by_krate.get(name).copied()
    }
}
