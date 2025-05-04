use super::graph::{Graph, KrateDir, KrateIdx};
use crate::util::latest::{LatestPublisher, LatestReader, make_latest};
use aho_corasick::{AhoCorasick, Anchored, Input, MatchKind, StartKind};
use anyhow::{Context as _, Result, ensure};
use core::ops;
use hashbrown::HashMap;
use notify::{
    EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _, recommended_watcher,
};
use std::path::Path;
use std::time::Instant;

pub struct Watcher {
    inner: RecommendedWatcher,
    latest: LatestReader<Instant>,
}

impl ops::Deref for Watcher {
    type Target = LatestReader<Instant>;

    fn deref(&self) -> &Self::Target {
        &self.latest
    }
}

impl ops::DerefMut for Watcher {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.latest
    }
}

impl Watcher {
    pub fn new(graph: &'static Graph) -> Result<Self> {
        new_inner(graph).context("Failed to create watcher")
    }
}

fn new_inner(graph: &'static Graph) -> Result<Watcher> {
    let (publisher, reader) = make_latest(Instant::now());

    let dirs = extract_dirs(graph)?;

    let mut inner = recommended_watcher(EvHandler {
        graph,
        paths: PathMap::new(&dirs)?,
        latest: publisher,
        reported_event_err: false,
        reported_mapping_err: false,
    })?;

    for (_, dir) in &dirs {
        inner.watch(dir.path().as_std_path(), dir.rec_mode())?;
    }

    log::debug!("Watching {} paths", dirs.len());

    Ok(Watcher { inner, latest: reader })
}

impl KrateDir {
    fn rec_mode(&self) -> RecursiveMode {
        match self {
            KrateDir::Src(_) => RecursiveMode::Recursive,
            KrateDir::Root(_) => RecursiveMode::NonRecursive,
        }
    }
}

struct EvHandler {
    graph: &'static Graph,
    paths: PathMap,
    latest: LatestPublisher<Instant>,
    reported_event_err: bool,
    reported_mapping_err: bool,
}

impl EventHandler for EvHandler {
    fn handle_event(&mut self, event: notify::Result<notify::Event>) {
        let event = match event {
            Ok(ev) => ev,
            Err(err) => return self.report_event_err(&err),
        };

        match event.kind {
            EventKind::Any | EventKind::Other | EventKind::Access(_) => return,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => (),
        }

        let mut should_publish = false;

        for path in &event.paths {
            if path.ends_with(Path::new("Cargo.lock")) {
                log::trace!("Ignoring `Cargo.lock` event");
                continue;
            }

            should_publish = true;

            match self.paths.lookup(path) {
                None => {
                    self.report_unmapped_err(path);
                }
                Some(idx) => {
                    self.graph[idx].mark_dirty();
                }
            }
        }

        if should_publish {
            self.latest.publish(Instant::now());
        }
    }
}

impl EvHandler {
    fn report_event_err(&mut self, err: &notify::Error) {
        if self.reported_event_err {
            log::trace!("Watcher error: {err}");
        } else {
            self.reported_event_err = true;
            log::warn!("Watcher error: {err}");
            log::warn!("Future Watcher errors will be logged only as TRACE");
        }
    }

    fn report_unmapped_err(&mut self, path: &Path) {
        // The path map already traces all mappings.
        if !self.reported_mapping_err {
            self.reported_mapping_err = true;
            log::warn!("Unmapped path: {path:?}");
            log::warn!("Future unmapped paths will be logged only as TRACE");
        }
    }
}

struct PathMap {
    auto: AhoCorasick,
    /// Mapping from [`aho_corasick::Match::pattern`] to [`KrateIdx`].
    mapping: Box<[KrateIdx]>,
}

impl PathMap {
    fn new(dirs: &[(KrateIdx, &KrateDir)]) -> Result<Self> {
        let mut builder = AhoCorasick::builder();
        builder.start_kind(StartKind::Anchored);
        // I don't think prefilters are ever used with anchored searches.
        builder.prefilter(false);
        builder.match_kind(MatchKind::LeftmostLongest);

        let auto = builder.build(dirs.iter().map(|(_, dir)| dir.path().as_str()))?;
        let mapping = dirs.iter().map(|&(idx, _)| idx).collect();

        Ok(PathMap { auto, mapping })
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "Pattern IDs are assumed to be valid."
    )]
    #[must_use]
    pub fn lookup(&self, path: &Path) -> Option<KrateIdx> {
        let input = Input::new(path.as_os_str().as_encoded_bytes()).anchored(Anchored::Yes);

        let res = self
            .auto
            .find(input)
            .map(|m| self.mapping[m.pattern().as_usize()]);

        log::trace!("Path mapped: {path:?} -> {res:?}");

        res
    }
}

fn extract_dirs(graph: &Graph) -> Result<Box<[(KrateIdx, &KrateDir)]>> {
    let mut dirs = HashMap::new();

    for krate in graph.collect_krates_to_watch() {
        for dir in krate.dirs() {
            let path = dir.path();

            let did_insert = dirs.try_insert(path, (krate.idx(), dir)).is_ok();

            ensure!(did_insert, "Duplicate crate path: {path:?}");
        }
    }

    let mut dirs: Box<[_]> = dirs.into_values().collect();
    dirs.sort_unstable_by_key(|&(_, dir)| dir.path());

    if log::log_enabled!(log::Level::Trace) {
        for (idx, dir) in &dirs {
            log::trace!(
                "Path mapped: {:?} -> {idx:?} ({:?})",
                dir.path(),
                dir.rec_mode()
            );
        }
    }

    Ok(dirs)
}
