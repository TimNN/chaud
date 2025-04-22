use super::data::KrateData;
use super::env::BuildEnv;
use super::{KrateIdx, KrateIndex};
use crate::hot::cargo::metadata::Metadata;
use crate::hot::util::OrderedTopo;
use crate::hot::workspace::graph::info::KrateInfo;
use anyhow::{Context as _, Result, ensure};
use core::ops;

pub struct Graph {
    env: BuildEnv,
    index: KrateIndex,
    krates: Box<[KrateData]>,
    reload_order: Box<[KrateIdx]>,
}

impl ops::Index<KrateIdx> for Graph {
    type Output = KrateData;

    fn index(&self, idx: KrateIdx) -> &Self::Output {
        &self.krates[idx.usize()]
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
    let krates = load_krates(&meta, &env, &index)?;
    let reload_order = reload_order(&krates)?;

    Ok(Box::leak(Box::new(Graph {
        env,
        index,
        krates,
        reload_order,
    })))
}

fn load_krates(meta: &Metadata, env: &BuildEnv, index: &KrateIndex) -> Result<Box<[KrateData]>> {
    let mut krates = vec![];
    let mut dylib_cnt = 0;

    for pkg in meta.packages() {
        let krate = KrateData::new(KrateInfo::new(env, index, pkg)?);
        if krate.is_dylib() {
            dylib_cnt += 1;
            log::info!("Found dylib crate: {krate}");
        }
        krates.push(krate);
    }

    krates.sort_unstable_by_key(|k| k.idx());
    for (pos, krate) in krates.iter().enumerate() {
        ensure!(pos == krate.idx().usize());
    }

    log::info!("Found {} crates ({dylib_cnt} dylibs)", krates.len());

    Ok(krates.into_boxed_slice())
}

fn reload_order(krates: &[KrateData]) -> Result<Box<[KrateIdx]>> {
    let mut topo = OrderedTopo::new();

    for krate in krates {
        for dep in krate.deps() {
            topo.add_dependency(*dep, krate.idx());
        }
    }

    topo.sort()
}
