use crate::hot::util::etx;
use anyhow::{Context as _, Result};
use camino::Utf8Path;
#[cfg(unix)]
use libloading::os::unix as ll;
#[cfg(windows)]
use libloading::os::windows as ll;

#[derive(Copy, Clone)]
pub struct Library(&'static ll::Library);

impl Library {
    pub fn load(path: &Utf8Path) -> Result<Self> {
        let lib = load_inner(path).with_context(etx!("Failed to load {path:?}"))?;
        Ok(Self(Box::leak(Box::new(lib))))
    }
}

fn load_inner(path: &Utf8Path) -> Result<ll::Library> {
    // SAFETY: We cannot guarantee anything about the initialization routines.
    // This is covered under the `unsafe-hot-reload` feature opt-in. We'll never
    // run termination routines.
    unsafe {
        #[cfg(unix)]
        let lib = ll::Library::open(Some(path), ll::RTLD_GLOBAL | ll::RTLD_NOW)?;

        #[cfg(windows)]
        let lib = ll::Library::new(path)?;

        Ok(lib)
    }
}
