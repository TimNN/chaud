//! The **def**inition of the [`Graph`] type.

use super::{BuildEnv, KrateIdx, KrateIndex};
use crate::cargo::metadata::{KrateName, Metadata};
use crate::util::etx;
use anyhow::{Context as _, Result};

pub struct Graph {
    env: BuildEnv,
    index: KrateIndex,
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
}

fn new_inner() -> Result<&'static Graph> {
    let meta = Metadata::load()?;

    let index = KrateIndex::new(meta.packages())?;
    let env = BuildEnv::new(&meta, &index)?;

    Ok(Box::leak(Box::new(Graph { env, index })))
}
