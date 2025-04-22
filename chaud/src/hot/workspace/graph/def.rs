//! The **def**inition of the [`Graph`] type.

use super::data::KrateData;
use super::env::BuildEnv;
use super::paths::PathMap;
use super::{DylibMap, KrateIdx, KrateIndex};
use crate::hot::cargo::metadata::Metadata;
use crate::hot::util::assert::err_assert;
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
