use super::data::KrateData;
use super::env::BuildEnv;
use crate::hot::workspace::index::{KrateIdx, KrateIndex};
use core::ops;

pub struct Graph {
    index: KrateIndex,
    env: BuildEnv,
    krates: Box<[KrateData]>,
}

impl ops::Index<KrateIdx> for Graph {
    type Output = KrateData;

    fn index(&self, idx: KrateIdx) -> &Self::Output {
        &self.krates[idx.usize()]
    }
}
