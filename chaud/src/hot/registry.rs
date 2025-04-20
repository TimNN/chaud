use super::dylib::Sym;
use super::handle::ErasedHandle;
use super::util::{IgnoreDisconnectExt as _, etx};
use super::workspace::worker;
use std::sync::mpsc;

pub struct RegistryItem {
    pub sym: Sym,
    pub handle: ErasedHandle,
}

pub type RegistrySender = mpsc::Sender<RegistryItem>;
pub type RegistryReceiver = mpsc::Receiver<RegistryItem>;

pub struct Registry {
    sender: Option<RegistrySender>,
}

impl Registry {
    pub const fn new() -> Self {
        Self { sender: None }
    }

    pub fn register(&mut self, sym: Sym, handle: ErasedHandle) {
        self.sender()
            .send(RegistryItem { sym, handle })
            .log_disconnect(etx!(
                "Registering {handle:?} failed, hot-reloading will not work."
            ));
    }

    fn sender(&mut self) -> &RegistrySender {
        self.sender.get_or_insert_with(|| {
            let (sender, receiver) = mpsc::channel();
            worker::launch(receiver);
            sender
        })
    }
}
