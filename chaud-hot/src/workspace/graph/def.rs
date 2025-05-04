//! The **def**inition of the [`Graph`] type.

use super::{BuildEnv, Krate, KrateIdx, KrateIndex};
use crate::cargo::metadata::Metadata;
use crate::util::assert::err_assert;
use anyhow::{Context as _, Result};
use core::ops;
use hashbrown::HashSet;
use std::collections::VecDeque;

pub struct Graph {
    env: BuildEnv,
    index: KrateIndex,
    krates: Box<[Krate]>,
}

impl ops::Index<KrateIdx> for Graph {
    type Output = Krate;

    #[expect(
        clippy::indexing_slicing,
        reason = "crate indices are assumed to be valid"
    )]
    fn index(&self, idx: KrateIdx) -> &Self::Output {
        &self.krates[idx.usize()]
    }
}

impl Graph {
    pub fn new() -> Result<&'static Self> {
        new_inner().context("Failed to load crate graph")
    }

    pub fn env(&self) -> &BuildEnv {
        &self.env
    }

    pub fn krates(&self) -> &[Krate] {
        &self.krates
    }

    pub fn collect_krates_to_watch(&self) -> impl Iterator<Item = &Krate> {
        collect_inner(self).into_iter().map(|k| &self[k])
    }
}

fn new_inner() -> Result<&'static Graph> {
    let meta = Metadata::load()?;

    let index = KrateIndex::new(meta.packages())?;
    let env = BuildEnv::new(&meta, &index)?;
    let krates = load_krates(&meta, &env, &index)?;

    Ok(Box::leak(Box::new(Graph { env, index, krates })))
}

fn load_krates(meta: &Metadata, env: &BuildEnv, index: &KrateIndex) -> Result<Box<[Krate]>> {
    let mut krates = vec![];

    for pkg in meta.packages() {
        krates.push(Krate::new(env, index, pkg)?);
    }

    krates.sort_unstable_by_key(|k| k.idx());
    for (pos, krate) in krates.iter().enumerate() {
        err_assert!(pos == krate.idx().usize());
    }

    Ok(krates.into_boxed_slice())
}

fn collect_inner(graph: &Graph) -> Box<[KrateIdx]> {
    let mut found = vec![];
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();
    seen.insert(graph.env.root());
    queue.push_back(graph.env.root());

    while let Some(idx) = queue.pop_front() {
        found.push(idx);
        for &dep in graph[idx].deps() {
            if seen.insert(dep) {
                queue.push_back(dep);
            }
        }
    }

    found.into_boxed_slice()
}
