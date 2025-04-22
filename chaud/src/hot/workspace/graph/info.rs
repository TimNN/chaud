use super::env::BuildEnv;
use super::{KrateIdx, KrateIndex};
use crate::hot::cargo::metadata::{
    Dependency, DependencyKind, KrateName, Package, PackageName, Target, TargetKind,
};
use crate::hot::util::etx;
use anyhow::{Context as _, Result, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use core::fmt;
use std::env::consts::{DLL_EXTENSION, DLL_PREFIX};

pub struct DylibPaths {
    manifest_file: Utf8PathBuf,
    dylib_file: Utf8PathBuf,

    root_dir: Utf8PathBuf,
    src_dir: Utf8PathBuf,

    /// Only set if different from [`root_dir`][Self::root_dir] and
    /// [`src_dir`][Self::src_dir].
    build_dir: Option<Utf8PathBuf>,
}

/// Immutable information about a crate.
pub struct KrateInfo {
    idx: KrateIdx,
    pkg: PackageName,
    name: KrateName<'static>,
    deps: Box<[KrateIdx]>,
    /// Only set if this crate is a `dylib`.
    paths: Option<DylibPaths>,
}

impl fmt::Display for KrateInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.pkg)
    }
}

impl KrateInfo {
    pub(super) fn new(env: &BuildEnv, index: &KrateIndex, pkg: &Package) -> Result<Self> {
        new_inner(env, index, pkg)
            .with_context(etx!("Failed to build crate info for {}", pkg.name()))
    }

    pub(super) fn idx(&self) -> KrateIdx {
        self.idx
    }

    pub(super) fn deps(&self) -> &[KrateIdx] {
        &self.deps
    }

    pub(super) fn dylib_paths(&self) -> Option<&DylibPaths> {
        self.paths.as_ref()
    }

    pub(super) fn is_dylib(&self) -> bool {
        self.dylib_paths().is_some()
    }

    pub(super) fn paths_iter(&self) -> impl Iterator<Item = &Utf8Path> {
        let paths: [Option<&Utf8Path>; 3] = if let Some(paths) = &self.paths {
            [
                Some(&paths.src_dir),
                Some(&paths.root_dir),
                paths.build_dir.as_ref().map(AsRef::as_ref),
            ]
        } else {
            [None, None, None]
        };

        paths.into_iter().flatten()
    }
}

fn new_inner(env: &BuildEnv, index: &KrateIndex, package: &Package) -> Result<KrateInfo> {
    let pkg = package.name().clone();
    let idx = index
        .get_pkg(&pkg)
        .context("Package not found in the index")?;
    let name = pkg.to_krate();

    let deps = filter_deps(index, package.dependencies());

    let paths = DylibPaths::new(env, package).context("Failed to determine dylib paths")?;

    Ok(KrateInfo { idx, pkg, name, deps, paths })
}

fn filter_deps(index: &KrateIndex, deps: &[Dependency]) -> Box<[KrateIdx]> {
    let include = |dk| [DependencyKind::Build, DependencyKind::Normal].contains(&dk);

    let mut deps: Vec<_> = deps
        .iter()
        .filter(|d| include(d.kind()))
        .filter_map(|d| index.get_pkg(d.name()))
        .collect();
    deps.sort_unstable();
    // In case the same dependency shows up multiple times, e.g. as a "build"
    // and "normal" dependency.
    deps.dedup();

    deps.into_boxed_slice()
}

impl DylibPaths {
    fn new(env: &BuildEnv, pkg: &Package) -> Result<Option<Self>> {
        let targets = Targets::new(pkg.targets())?;
        let Some(targets) = targets else {
            return Ok(None);
        };

        let manifest_file = pkg.manifest_path().to_owned();

        let root_dir = manifest_file
            .parent()
            .context("manifest_path has no parent")?
            .to_owned();

        let src_dir = targets
            .dylib
            .src_path()
            .parent()
            .context("src_path has no parent")?
            .to_owned();

        ensure!(
            src_dir != root_dir,
            "Root and src dir are identical for {} ({:?}), this is currently \
                unsupported.",
            pkg.name(),
            src_dir
        );

        let mut build_dir = None;
        if let Some(build) = targets.build {
            let dir = build
                .src_path()
                .parent()
                .context("build src_path has no parent")?;

            if dir != root_dir && dir != src_dir {
                build_dir = Some(dir.to_owned());
            }
        }

        let dylib_file = env.lib_dir().join(format!(
            "{DLL_PREFIX}{}.{DLL_EXTENSION}",
            pkg.name().to_krate().as_str()
        ));

        Ok(Some(Self {
            manifest_file,
            dylib_file,
            root_dir,
            src_dir,
            build_dir,
        }))
    }

    pub(super) fn dylib_file(&self) -> &Utf8Path {
        &self.dylib_file
    }
}

struct Targets<'a> {
    dylib: &'a Target,
    build: Option<&'a Target>,
}

impl<'a> Targets<'a> {
    fn new(targets: &'a [Target]) -> Result<Option<Self>> {
        let mut build = None;
        let mut dylib = None;

        for target in targets {
            if target.kind().contains(&TargetKind::CustomBuild) {
                ensure!(build.is_none(), "Multiple custom-build targets");
                build = Some(target);
            }
            if target.kind().contains(&TargetKind::Dylib) {
                ensure!(build.is_none(), "Multiple dylib targets");
                dylib = Some(target);
            }
        }

        let Some(dylib) = dylib else {
            return Ok(None);
        };

        Ok(Some(Self { dylib, build }))
    }
}
