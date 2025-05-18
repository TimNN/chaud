use super::graph::{Graph, KrateDir};
use crate::util::latest::{LatestPublisher, LatestReader, make_latest};
use anyhow::{Context as _, Result, ensure};
use core::ops;
use hashbrown::HashMap;
use notify::{
    EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _, recommended_watcher,
};
use std::time::Instant;

pub struct Watcher {
    #[expect(dead_code, reason = "keep alive")]
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

    let mut inner =
        recommended_watcher(EvHandler { latest: publisher, reported_event_err: false })?;

    for dir in &dirs {
        inner.watch(dir.path().as_std_path(), dir.rec_mode())?;
        log::trace!("Watching: {:?} ({:?})", dir.path(), dir.rec_mode());
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
    latest: LatestPublisher<Instant>,
    reported_event_err: bool,
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

        if !event.paths.is_empty() {
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
}

fn extract_dirs(graph: &Graph) -> Result<Box<[&KrateDir]>> {
    let mut dirs = HashMap::new();

    for krate in graph.collect_krates_to_watch() {
        for dir in krate.dirs() {
            let path = dir.path();

            let did_insert = dirs.try_insert(path, dir).is_ok();

            ensure!(did_insert, "Duplicate crate path: {path:?}");
        }
    }

    let mut dirs: Box<[_]> = dirs.into_values().collect();
    dirs.sort_unstable_by_key(|dir| dir.path());

    Ok(dirs)
}
