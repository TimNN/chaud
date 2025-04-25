//! The **def**inition of the [`Symbols`] type.

use super::DylibData;
use crate::hot::util::assert::err_assert;
use crate::hot::workspace::graph::{DylibIdx, Graph};
use anyhow::{Context as _, Result};
use camino::Utf8Path;
use core::ops;
use parking_lot::Mutex;

pub struct Symbols {
    inner: Mutex<SymbolsInner>,
}

pub(super) struct SymbolsInner {
    dylibs: Box<[DylibData]>,
    lib_dir: &'static Utf8Path,
}

impl ops::Index<DylibIdx> for SymbolsInner {
    type Output = DylibData;

    #[expect(
        clippy::indexing_slicing,
        reason = "dylib indices are assumed to be valid"
    )]
    fn index(&self, idx: DylibIdx) -> &Self::Output {
        &self.dylibs[idx.usize()]
    }
}

impl ops::IndexMut<DylibIdx> for SymbolsInner {
    #[expect(
        clippy::indexing_slicing,
        reason = "dylib indices are assumed to be valid"
    )]
    fn index_mut(&mut self, idx: DylibIdx) -> &mut Self::Output {
        &mut self.dylibs[idx.usize()]
    }
}

impl Symbols {
    pub fn new(graph: &'static Graph) -> Result<&'static Symbols> {
        new_inner(graph).context("Failed to initialize symbol tracker")
    }

    pub fn copy_libs(&self) -> Result<()> {
        let inner = &mut *self.inner.lock();
        for d in &mut inner.dylibs {
            d.maybe_copy(inner.lib_dir)?;
        }
        Ok(())
    }

    pub fn resolve_symbols(&self) -> Result<()> {
        let inner = &mut *self.inner.lock();
        for d in &mut inner.dylibs {
            d.resolve_symbols()?;
        }
        Ok(())
    }
}

fn new_inner(graph: &'static Graph) -> Result<&'static Symbols> {
    let mut dylibs = vec![];

    for krate in graph.dylibs() {
        dylibs.push(DylibData::new(krate)?);
    }

    dylibs.sort_unstable_by_key(|d| d.idx());
    for (pos, dylib) in dylibs.iter().enumerate() {
        err_assert!(pos == dylib.idx().usize());
    }

    let lib_dir = graph.env().lib_dir();

    Ok(Box::leak(Box::new(Symbols {
        inner: Mutex::new(SymbolsInner { dylibs: dylibs.into_boxed_slice(), lib_dir }),
    })))
}
