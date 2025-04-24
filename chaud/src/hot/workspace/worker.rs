use crate::hot::registry::RegistryReceiver;
use crate::hot::workspace::graph::Graph;
use crate::hot::workspace::symbols::Symbols;
use anyhow::Result;
use std::thread;

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

fn init(_registry: RegistryReceiver) -> Result<(&'static Graph, &'static Symbols)> {
    let graph = Graph::new()?;
    let symbols = Symbols::new(graph)?;

    Ok((graph, symbols))
}
