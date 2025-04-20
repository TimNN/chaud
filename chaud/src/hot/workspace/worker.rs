use crate::hot::registry::RegistryReceiver;

pub fn launch(_registry: RegistryReceiver) {
    log::debug!("Launching worker thread.");
}
