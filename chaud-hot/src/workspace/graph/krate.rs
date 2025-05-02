use super::{BuildEnv, ClearDirtyResult, KrateFlags, KrateIdx, KrateIndex};
use crate::cargo::metadata::{
    Dependency, DependencyKind, KrateName, ManifestPath, Package, PackageName, Target, TargetKind,
    TargetName,
};
use crate::util::etx;
use anyhow::{Context as _, Result, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use core::fmt;
use hashbrown::HashSet;

#[derive(Debug)]
pub enum KrateDir {
    Src(Utf8PathBuf),
    Root(Utf8PathBuf),
}

impl KrateDir {
    pub fn path(&self) -> &Utf8Path {
        match self {
            KrateDir::Root(p) | KrateDir::Src(p) => p,
        }
    }
}

pub struct Krate {
    idx: KrateIdx,
    pkg: PackageName,
    name: KrateName<'static>,
    initial_version: String,
    mani: ManifestPath,
    deps: Box<[KrateIdx]>,
    dirs: Box<[KrateDir]>,
    flags: KrateFlags,
}

impl fmt::Display for Krate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pkg)
    }
}

impl Krate {
    pub(super) fn new(env: &BuildEnv, index: &KrateIndex, pkg: &Package) -> Result<Self> {
        new_inner(env, index, pkg)
            .with_context(etx!("Failed to build crate info for {}", pkg.name()))
    }

    pub(super) fn pkg(&self) -> &PackageName {
        &self.pkg
    }

    pub fn idx(&self) -> KrateIdx {
        self.idx
    }

    pub(super) fn name(&self) -> &KrateName<'static> {
        &self.name
    }

    pub(super) fn initial_version(&self) -> &str {
        &self.initial_version
    }

    pub(super) fn deps(&self) -> &[KrateIdx] {
        &self.deps
    }

    pub fn dirs(&self) -> &[KrateDir] {
        &self.dirs
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
}

fn new_inner(env: &BuildEnv, index: &KrateIndex, package: &Package) -> Result<Krate> {
    let pkg = package.name().clone();
    let idx = index
        .get_pkg(&pkg)
        .context("Package not found in the index")?;
    let name = pkg.to_krate();

    let initial_version = package.version().to_owned();
    let mani = package.manifest_path().to_owned();

    let deps = filter_deps(index, package.dependencies());

    let root_bin = match env.root() == idx {
        true => Some(env.bin()),
        false => None,
    };
    let dirs = krate_dirs(root_bin, package).context("Failed to determine crate dirs")?;

    Ok(Krate {
        idx,
        pkg,
        name,
        initial_version,
        mani,
        deps,
        dirs,
        flags: KrateFlags::new(),
    })
}

fn filter_deps(index: &KrateIndex, deps: &[Dependency]) -> Box<[KrateIdx]> {
    let follow = [DependencyKind::Normal, DependencyKind::Build];

    let mut deps: Vec<_> = deps
        .iter()
        .filter(|k| follow.contains(&k.kind()))
        .filter_map(|d| index.get_pkg(d.name()))
        .collect();
    deps.sort_unstable();
    // In case the same dependency shows up multiple times, e.g. as a "build"
    // and "normal" dependency.
    deps.dedup();

    deps.into_boxed_slice()
}

fn krate_dirs(root_bin: Option<&TargetName>, pkg: &Package) -> Result<Box<[KrateDir]>> {
    let targets = filter_targets(root_bin, pkg.targets())?;
    let Some(targets) = targets else {
        return Ok(Box::new([]));
    };

    let mani = pkg.manifest_path().to_owned();

    let root_dir = mani
        .path()
        .parent()
        .context("manifest_path has no parent")?;

    let mut dirs = vec![KrateDir::Root(root_dir.to_owned())];
    let mut seen = HashSet::new();
    seen.insert(root_dir);

    for target in targets {
        let dir = target
            .src_path()
            .parent()
            .context("src_path has no parent")?;

        if seen.insert(dir) {
            dirs.push(KrateDir::Src(dir.to_owned()));
        }
    }

    Ok(dirs.into_boxed_slice())
}

fn filter_targets<'a>(
    root_bin: Option<&TargetName>,
    targets: &'a [Target],
) -> Result<Option<impl Iterator<Item = &'a Target>>> {
    let is_rlib = |tk| [TargetKind::Lib, TargetKind::RLib].contains(tk);

    let mut bin = None;
    let mut lib = None;
    let mut build = None;

    for target in targets {
        if let Some(root_bin) = root_bin {
            if target.kind().contains(&TargetKind::Bin) && target.name() == root_bin {
                ensure!(bin.is_none(), "Multiple root bin targets");
                bin = Some(target);
            }
        }

        if target.kind().iter().any(is_rlib) {
            ensure!(lib.is_none(), "Multiple lib targets");
            lib = Some(target);
        }
        if target.kind().contains(&TargetKind::CustomBuild) {
            ensure!(build.is_none(), "Multiple custom-build targets");
            build = Some(target);
        }
    }

    if bin.is_none() && lib.is_none() {
        return Ok(None);
    }

    Ok(Some([bin, lib, build].into_iter().flatten()))
}
