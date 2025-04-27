//! The **def**inition of the [`Graph`] type.

use super::data::KrateData;
use super::env::BuildEnv;
use super::info::DylibDir;
use super::paths::PathMap;
use super::{DylibMap, KrateIdx, KrateIndex};
use crate::hot::cargo::metadata::{KrateName, Metadata};
use crate::hot::util::assert::err_assert;
use crate::hot::util::etx;
use crate::hot::workspace::graph::DylibIdx;
use crate::hot::workspace::graph::info::KrateInfo;
use anyhow::{Context as _, Result};
use core::ops;

pub struct Graph {
    env: BuildEnv,
    index: KrateIndex,
    krates: Box<[KrateData]>,
    dylib_map: DylibMap,
    path_map: PathMap,
}

impl ops::Index<KrateIdx> for Graph {
    type Output = KrateData;

    #[expect(
        clippy::indexing_slicing,
        reason = "crate indices are assumed to be valid"
    )]
    fn index(&self, idx: KrateIdx) -> &Self::Output {
        &self.krates[idx.usize()]
    }
}

impl ops::Index<DylibIdx> for Graph {
    type Output = KrateData;

    #[expect(
        clippy::indexing_slicing,
        reason = "dylib indices are assumed to be valid"
    )]
    fn index(&self, idx: DylibIdx) -> &Self::Output {
        &self.krates[self.dylib_map[idx].usize()]
    }
}

impl Graph {
    pub fn new() -> Result<&'static Self> {
        new_inner().context("Failed to load crate graph")
    }

    pub fn env(&self) -> &BuildEnv {
        &self.env
    }

    pub fn krate_named(&self, name: &KrateName) -> Result<KrateIdx> {
        self.index
            .get_krate(name)
            .with_context(etx!("No crate named {name}"))
    }

    pub fn dylibs(&self) -> impl Iterator<Item = &KrateData> {
        self.dylib_map.indices().map(|i| &self[i])
    }

    pub fn path_map(&self) -> &PathMap {
        &self.path_map
    }

    pub fn watch(&self, idx: KrateIdx, mut new_path: impl FnMut(DylibDir)) {
        watch_inner(self, idx, true, &mut new_path);
    }

    pub fn clear_patched(&self) {
        for krate in &self.krates {
            krate.clear_patched();
        }
    }
}

fn new_inner() -> Result<&'static Graph> {
    let meta = Metadata::load()?;

    let env = BuildEnv::new(&meta)?;
    let index = KrateIndex::new(meta.packages())?;
    let mut krates = load_krates(&meta, &env, &index)?;
    let path_map = PathMap::new(&krates)?;
    let dylib_map = DylibIdx::assign(&mut krates)?;

    log::info!("Found {} crates ({} dylibs)", krates.len(), dylib_map.len());

    Ok(Box::leak(Box::new(Graph {
        env,
        index,
        krates,
        dylib_map,
        path_map,
    })))
}

fn load_krates(meta: &Metadata, env: &BuildEnv, index: &KrateIndex) -> Result<Box<[KrateData]>> {
    let mut krates = vec![];

    for pkg in meta.packages() {
        let krate = KrateData::new(KrateInfo::new(env, index, pkg)?);
        if krate.is_dylib() {
            log::info!("Found dylib crate: {krate}");
        }
        krates.push(krate);
    }

    krates.sort_unstable_by_key(|k| k.idx());
    for (pos, krate) in krates.iter().enumerate() {
        err_assert!(pos == krate.idx().usize());
    }

    Ok(krates.into_boxed_slice())
}

fn watch_inner(graph: &Graph, idx: KrateIdx, explicit: bool, new_path: &mut impl FnMut(DylibDir)) {
    let krate = &graph[idx];
    if krate.watch() {
        log::trace!("Already watching {krate}");
        return;
    }

    krate.dirs_iter().for_each(&mut *new_path);

    if !krate.is_dylib() {
        if explicit {
            log::error!("Cannot watch {krate}, `crate-type` does not include \"dylib\"");
        } else if let Some(dep) = first_dylib_dep(graph, krate) {
            log::error!(
                "{krate} does not include `crate-type` \"dylib\", but depends on dylib {dep}"
            );
        } else {
            log::info!("{krate} will not be watched (`crate-type` does not include \"dylib\")");
        }
    }

    for &dep in krate.deps() {
        watch_inner(graph, dep, false, new_path);
    }
}

fn first_dylib_dep<'g>(graph: &'g Graph, krate: &KrateData) -> Option<&'g KrateData> {
    for &dep in krate.deps() {
        let dep = &graph[dep];
        if dep.is_dylib() {
            return Some(dep);
        }
    }
    None
}
