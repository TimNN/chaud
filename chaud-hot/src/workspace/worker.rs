use super::graph::Graph;
use super::watcher::Watcher;
use crate::cargo::Builder;
use crate::util::minilog;
use anyhow::Result;
use core::time::Duration;
use parking_lot::Once;
use std::thread;

/// Launch the worker thread.
///
/// This function is idempotent.
///
/// If [`log`] has not been initialized yet, a minimal logger will be installed.
pub fn launch() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        minilog::init();

        log::trace!("Launching worker thread");

        let spawn_result = thread::Builder::new()
            .name("chaud-worker".to_owned())
            .spawn(work);

        if let Err(e) = spawn_result {
            log::error!("Failed to spawn Chaud worker: {e:#}");
        }
    });
}

fn work() {
    log::debug!("Chaud worker thread is running");

    let worker = match init() {
        Ok(val) => val,
        Err(e) => {
            log::error!("Initialization failed, shutting down worker thread: {e:#}");
            return;
        }
    };

    main(&worker);
}

struct Worker {
    graph: &'static Graph,
    builder: Builder,
    watcher: Watcher,
}

fn init() -> Result<Worker> {
    let graph = Graph::new()?;
    let builder = Builder::init(graph.env())?;
    let watcher = Watcher::new(graph)?;
    Ok(Worker { graph, builder, watcher })
}

fn main(_w: &Worker) {
    loop {
        thread::sleep(Duration::from_secs(5));
    }
}
