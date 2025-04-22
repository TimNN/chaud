use anyhow::{Result, ensure};
use core::hash::Hash;
use topological_sort::TopologicalSort;

/// Performs a topological sort, ordering equal elements based on their [`Ord`]
/// implementation.
///
/// Uses [`TopologicalSort`] internally.
pub struct OrderedTopo<T> {
    inner: TopologicalSort<T>,
}

impl<T: Hash + Ord + Clone> OrderedTopo<T> {
    pub fn new() -> Self {
        Self { inner: TopologicalSort::new() }
    }

    /// Registers a dependency between `prec` and `succ`.
    ///
    /// The `prec` will be ordered before `succ`.
    ///
    /// See [`TopologicalSort::add_dependency`].
    pub fn add_dependency(&mut self, prec: T, succ: T) {
        self.inner.add_dependency(prec, succ);
    }

    pub fn sort(self) -> Result<Box<[T]>> {
        let mut inner = self.inner;

        let mut buf = vec![];
        loop {
            let mut next = inner.pop_all();
            if next.is_empty() {
                break;
            }
            next.sort_unstable();
            buf.extend(next);
        }
        ensure!(inner.is_empty(), "cycle detected");

        Ok(buf.into_boxed_slice())
    }
}
