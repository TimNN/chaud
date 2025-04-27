//! The **def**inition of the [`Symbols`] type.

use super::DylibData;
use crate::hot::cargo::metadata::PackageName;
use crate::hot::dylib::Sym;
use crate::hot::handle::ErasedHandle;
use crate::hot::util::assert::err_assert;
use crate::hot::util::etx;
use crate::hot::workspace::graph::{DylibIdx, Graph};
use crate::hot::workspace::watcher::Watcher;
use anyhow::{Context as _, Result};
use core::ops;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use std::collections::BTreeSet;

pub struct Symbols {
    inner: Mutex<SymbolsInner>,
    watcher: Mutex<Watcher>,
    graph: &'static Graph,
}

pub(super) struct SymbolsInner {
    dylibs: Box<[DylibData]>,
    with_tracked: BTreeSet<&'static PackageName>,
}

pub struct TrackedKrates<'a> {
    guard: MappedMutexGuard<'a, BTreeSet<&'static PackageName>>,
}

impl TrackedKrates<'_> {
    pub fn iter(&self) -> impl Iterator<Item = &'static PackageName> {
        self.guard.iter().copied()
    }
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
    pub fn new(graph: &'static Graph, watcher: Watcher) -> Result<&'static Symbols> {
        new_inner(graph, watcher).context("Failed to initialize symbol tracker")
    }

    pub fn tracked_krates(&self) -> TrackedKrates {
        TrackedKrates {
            guard: MutexGuard::map(self.inner.lock(), |inner| &mut inner.with_tracked),
        }
    }

    pub fn copy_libs(&self) -> Result<()> {
        let inner = &mut *self.inner.lock();
        for d in &mut inner.dylibs {
            d.maybe_copy(self.graph.env().lib_dir())?;
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

    pub fn load_libs(&self) -> Result<()> {
        let inner = &mut *self.inner.lock();
        for d in &mut inner.dylibs {
            d.load()?;
        }
        Ok(())
    }

    pub fn load_and_activate_symbols(&self) -> Result<u32> {
        let inner = &mut *self.inner.lock();

        // Ensure that _all_ symbols have been successfully loaded before
        // activating _any_ of them.
        for d in &mut inner.dylibs {
            d.load_symbols()?;
        }

        let mut count = 0;
        for d in &mut inner.dylibs {
            d.activate_symbols(&mut count)?;
        }
        Ok(count)
    }

    pub fn register(&self, sym: Sym, handle: ErasedHandle) -> Result<()> {
        register_inner(self, sym, handle).with_context(etx!("Failed to register {sym:?}"))
    }
}

fn new_inner(graph: &'static Graph, watcher: Watcher) -> Result<&'static Symbols> {
    let mut dylibs = vec![];

    for krate in graph.dylibs() {
        dylibs.push(DylibData::new(krate)?);
    }

    dylibs.sort_unstable_by_key(|d| d.idx());
    for (pos, dylib) in dylibs.iter().enumerate() {
        err_assert!(pos == dylib.idx().usize());
    }

    Ok(Box::leak(Box::new(Symbols {
        graph,
        inner: Mutex::new(SymbolsInner {
            dylibs: dylibs.into_boxed_slice(),
            with_tracked: BTreeSet::new(),
        }),
        watcher: Mutex::new(watcher),
    })))
}

fn register_inner(s: &Symbols, sym: Sym, handle: ErasedHandle) -> Result<()> {
    let krate = s.graph.krate_named(&sym.krate())?;

    {
        let watcher = &mut *s.watcher.lock();
        s.graph.watch(krate, |dir| watcher.watch(dir));
    }

    let krate = &s.graph[krate];
    let dylib = krate.dylib().with_context(etx!("Not a dylib: {krate}"))?;

    let inner = &mut *s.inner.lock();

    inner[dylib].register(sym, handle)?;
    inner.with_tracked.insert(krate.pkg());

    Ok(())
}
