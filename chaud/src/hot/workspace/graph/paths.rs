use super::KrateIdx;
use super::data::KrateData;
use aho_corasick::{AhoCorasick, Anchored, Input, MatchKind, StartKind};
use anyhow::{Context as _, Result, ensure};
use camino::Utf8Path;
use hashbrown::HashMap;
use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub enum PathMapResult {
    Unmapped,
    Krate(KrateIdx),
}

pub struct PathMap {
    auto: AhoCorasick,
    /// Mapping from [`aho_corasick::Match::pattern`] to [`PathMapResult`].
    mapping: Box<[PathMapResult]>,
}

impl PathMap {
    pub(super) fn new(krates: &[KrateData]) -> Result<Self> {
        new_inner(krates).context("Failed to build path map")
    }

    #[must_use]
    pub(super) fn lookup(&self, path: &Path) -> PathMapResult {
        let input = Input::new(path.as_os_str().as_encoded_bytes()).anchored(Anchored::Yes);

        let res = match self.auto.find(input) {
            Some(m) => self.mapping[m.pattern().as_usize()],
            None => PathMapResult::Unmapped,
        };

        log::trace!("Path mapped: {path:?} -> {res:?}");

        res
    }
}

fn new_inner(krates: &[KrateData]) -> Result<PathMap> {
    let paths = extract_paths(krates)?;

    let mut builder = AhoCorasick::builder();
    builder.start_kind(StartKind::Anchored);
    // I don't think prefilters are ever used with anchored searches.
    builder.prefilter(false);
    builder.match_kind(MatchKind::LeftmostLongest);

    let auto = builder.build(paths.iter().map(|t| t.0.as_str()))?;
    let mapping = paths.into_iter().map(|t| t.1).collect();

    Ok(PathMap { auto, mapping })
}

fn extract_paths(krates: &[KrateData]) -> Result<Box<[(&Utf8Path, PathMapResult)]>> {
    let mut paths = HashMap::new();

    for krate in krates {
        for path in krate.paths_iter() {
            let did_insert = paths
                .try_insert(path, PathMapResult::Krate(krate.idx()))
                .is_ok();

            ensure!(did_insert, "Duplicate crate path: {path:?}");
        }
    }

    let mut paths: Box<[_]> = paths.into_iter().collect();
    paths.sort_unstable_by_key(|&(path, _)| path);

    if log::log_enabled!(log::Level::Trace) {
        for (path, res) in &paths {
            log::trace!("Registered path mapping: {path} -> {res:?}");
        }
    }

    Ok(paths)
}
