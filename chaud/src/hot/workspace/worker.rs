use super::watcher::Watcher;
use crate::hot::registry::RegistryReceiver;
use crate::hot::util::latest::LatestReader;
use crate::hot::workspace::graph::Graph;
use crate::hot::workspace::symbols::Symbols;
use anyhow::Result;
use std::thread;
use std::time::Instant;

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

    let _ = match init(registry) {
        Ok(val) => val,
        Err(e) => {
            log::error!("Initialization failed, shutting down worker thread: {e:#}");
            return;
        }
    };
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
