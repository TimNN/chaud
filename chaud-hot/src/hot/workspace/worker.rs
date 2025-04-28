use super::watcher::Watcher;
use crate::hot::cargo;
use crate::hot::registry::RegistryReceiver;
use crate::hot::util::latest::LatestReader;
use crate::hot::workspace::graph::{ClearDirtyResult, Graph};
use crate::hot::workspace::patch::PatchResult;
use crate::hot::workspace::symbols::Symbols;
use anyhow::Result;
use core::time::Duration;
use std::thread;
use std::time::Instant;

const DEBOUNCE: Duration = Duration::from_millis(350);

pub fn launch(registry: RegistryReceiver) {
    log::trace!("Launching worker thread");

    let spawn_result = thread::Builder::new()
        .name("chaud-worker".to_owned())
        .spawn(move || work(registry));

    if let Err(e) = spawn_result {
        log::error!("Failed to spawn chaud worker: {e:#}");
    }
}

fn work(registry: RegistryReceiver) {
    log::debug!("Worker thread is running");

    let (graph, symbols, mut latest) = match init(registry) {
        Ok(val) => val,
        Err(e) => {
            log::error!("Initialization failed, shutting down worker thread: {e:#}");
            return;
        }
    };

    main(graph, symbols, &mut latest);
}

fn init(
    registry: RegistryReceiver,
) -> Result<(&'static Graph, &'static Symbols, LatestReader<Instant>)> {
    let graph = Graph::new()?;
    let (watcher, latest) = Watcher::new(graph)?;
    let symbols = Symbols::new(graph, watcher)?;

    thread::Builder::new()
        .name("chaud-symbol-receiver".to_owned())
        .spawn(move || symbol_receiver(&registry, symbols))?;

    Ok((graph, symbols, latest))
}

fn symbol_receiver(registry: &RegistryReceiver, symbols: &Symbols) {
    log::debug!("Symbol receiver thread is running");

    loop {
        let Ok(item) = registry.recv() else {
            log::error!("Symbol sender has disconnected");
            return;
        };

        if let Err(e) = symbols.register(item.sym, item.handle) {
            log::error!("{e:#}");
        } else {
            log::info!("Registered {:?}", item.sym);
        }
    }
}

fn main(graph: &'static Graph, symbols: &'static Symbols, latest: &mut LatestReader<Instant>) {
    log::debug!("Initialization successful, starting main work loop");

    loop {
        if let Err(e) = main_one(graph, symbols, latest) {
            log::error!("Work failed (will try again on next file change): {e:#}");
        }
    }
}

fn main_one(
    graph: &'static Graph,
    symbols: &'static Symbols,
    latest: &mut LatestReader<Instant>,
) -> Result<()> {
    log::debug!("Waiting for watcher...");
    let mut last = latest.wait();

    'has_dirty: loop {
        debounce(&mut last, latest);

        log::debug!("Patching...");
        match graph.patch_manifests()? {
            PatchResult::UpToDate => {}
            PatchResult::PatchApplied => {
                // The events from the patches may not have arrived yet, so
                // manually act as if they had.
                last = Instant::now();
                continue 'has_dirty;
            }
        }

        log::debug!("Clearing dirty...");
        match graph.clear_dirty_if_patched() {
            ClearDirtyResult::Ok => {}
            ClearDirtyResult::UnpatchedDirty => {
                continue 'has_dirty;
            }
        }

        log::debug!("Building...");
        if let Err(e) = cargo::build(graph.env(), symbols.tracked_krates().iter()) {
            log::info!("{e:#}");
            // `cargo build` failing is expected, so don't return an error.
            return Ok(());
        }

        if let Some(l) = latest.check() {
            log::debug!("Dirty after build");
            last = l;
            continue 'has_dirty;
        }

        log::trace!("Copying libs...");
        symbols.copy_libs()?;

        log::trace!("Resolving symbols...");
        symbols.resolve_symbols()?;

        // Once we start loading libs, we need to patch again for any further
        // changes.
        log::trace!("Clearing patched...");
        graph.clear_patched();

        log::trace!("Loading libs...");
        symbols.load_libs()?;

        log::trace!("Activating symbols...");
        let activated = symbols.load_and_activate_symbols()?;

        log::info!("Reloaded {activated} handles");

        return Ok(());
    }
}

#[expect(clippy::needless_continue, reason = "intentionally explicit")]
fn debounce(last: &mut Instant, latest: &mut LatestReader<Instant>) {
    log::trace!("Debouncing...");
    loop {
        match DEBOUNCE.checked_sub(last.elapsed()) {
            Some(remaining) => {
                thread::sleep(remaining);
                // Check for any updates while we slept.
                match latest.check() {
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
