use super::graph::{DylibDir, Graph, PathMapResult};
use crate::hot::util::latest::{LatestPublisher, LatestReader, make_latest};
use anyhow::{Context as _, Result};
use notify::{
    EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _, recommended_watcher,
};
use std::path::Path;
use std::time::Instant;

pub struct Watcher {
    inner: RecommendedWatcher,
}

struct EvHandler {
    graph: &'static Graph,
    latest: LatestPublisher<Instant>,
    reported_event_err: bool,
    reported_mapping_err: bool,
}

impl Watcher {
    pub fn new(graph: &'static Graph) -> Result<(Self, LatestReader<Instant>)> {
        let (latest, reader) = make_latest(Instant::now());

        let inner = recommended_watcher(EvHandler {
            graph,
            latest,
            reported_event_err: false,
            reported_mapping_err: false,
        })
        .context("Failed to create recommended watcher")?;

        Ok((Watcher { inner }, reader))
    }

    pub fn watch(&mut self, dir: DylibDir) {
        if let Err(e) = self.inner.watch(dir.path().as_std_path(), dir.rec_mode()) {
            log::warn!("Failed to watch {:?}: {e}", dir.path());
        } else {
            log::debug!("Watching: {:?}, {:?}", dir.path(), dir.rec_mode());
        }
    }
}

impl DylibDir<'_> {
    fn rec_mode(self) -> RecursiveMode {
        match self {
            DylibDir::Src(_) => RecursiveMode::Recursive,
            DylibDir::Root(_) => RecursiveMode::NonRecursive,
        }
    }
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

        if event.paths.is_empty() {
            return;
        }

        for path in &event.paths {
            match self.graph.path_map().lookup(path) {
                PathMapResult::Unmapped => {
                    self.report_unmapped_err(path);
                }
                PathMapResult::Krate(idx) => {
                    self.graph[idx].mark_dirty();
                }
            }
        }

        self.latest.publish(Instant::now());
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
