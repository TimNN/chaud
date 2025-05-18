use super::graph::Graph;
use super::watcher::Watcher;
use crate::cargo::Builder;
use crate::cargo::metadata::ManifestPath;
use crate::dylib;
use crate::util::minilog;
use anyhow::{Context as _, Result};
use core::time::Duration;
use parking_lot::Once;
use std::thread;
use std::time::Instant;

const DEBOUNCE: Duration = Duration::from_millis(350);

/// Launch the worker thread.
///
/// This function is idempotent.
///
/// If [`log`] has not been initialized yet, a minimal logger will be installed.
pub fn launch(root_pkg_manifest: &str, feature_flags: Option<&'static str>) {
    static INIT: Once = Once::new();

    let root_mani = ManifestPath::new(root_pkg_manifest);

    INIT.call_once(move || {
        minilog::init();

        log::trace!("Launching worker thread");

        let spawn_result = thread::Builder::new()
            .name("chaud-worker".to_owned())
            .spawn(move || work(root_mani, feature_flags));

        if let Err(e) = spawn_result {
            log::error!("Failed to spawn Chaud worker: {e:#}");
        }
    });
}

fn work(root_mani: ManifestPath, feature_flags: Option<&'static str>) {
    log::debug!("Chaud worker thread is running");

    let worker = match init(root_mani, feature_flags) {
        Ok(val) => val,
        Err(e) => {
            log::error!("Initialization failed, shutting down worker thread: {e:#}");
            return;
        }
    };

    main(worker);
}

struct Worker {
    graph: &'static Graph,
    builder: Builder,
    watcher: Watcher,
    epoch: u32,
}

fn init(root_mani: ManifestPath, feature_flags: Option<&'static str>) -> Result<Worker> {
    let graph = Graph::new(root_mani, feature_flags)?;
    let builder = Builder::init(graph.env())?;
    let watcher = Watcher::new(graph)?;
    Ok(Worker { graph, builder, watcher, epoch: 0 })
}

fn main(mut w: Worker) {
    log::debug!("Initialization successful, starting main work loop");

    loop {
        if let Err(e) = main_one(&mut w) {
            log::error!("Work failed (will try again on next file change): {e:#}");
        }
    }
}

fn main_one(Worker { graph, builder, watcher, epoch }: &mut Worker) -> Result<()> {
    let env = graph.env();

    log::debug!("Waiting for watcher...");
    let mut last = watcher.wait();

    'has_dirty: loop {
        debounce(&mut last, watcher);

        log::debug!("Preparing & building...");

        if let Err(e) = builder.build() {
            log::info!("{e:#}");
            // `cargo build` failing is expected, so don't return an error.
            return Ok(());
        }

        if let Some(l) = watcher.check() {
            log::debug!("Dirty after build, starting over");
            last = l;
            continue 'has_dirty;
        }

        *epoch = epoch.checked_add(1).context("Epoch overflowed")?;
        let dst = env
            .chaud_dir()
            .join(format!("{}.{epoch}.hot", env.bin().as_str()));
        builder.link_latest(&dst)?;

        log::debug!("Loading {dst:?}...");
        dylib::load(&dst)?;

        log::info!("Reload complete");

        return Ok(());
    }
}

#[expect(clippy::needless_continue, reason = "intentionally explicit")]
fn debounce(last: &mut Instant, watcher: &mut Watcher) {
    log::trace!("Debouncing...");
    loop {
        match DEBOUNCE.checked_sub(last.elapsed()) {
            Some(remaining) => {
                thread::sleep(remaining);
                // Check for any updates while we slept.
                match watcher.check() {
                    Some(l) => {
                        *last = l;
                        continue;
                    }
                    None => return,
                }
            }
            None => return,
        };
    }
}
