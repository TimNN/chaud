use crate::util::etx;
use anyhow::{Context as _, Result};
use camino::Utf8Path;
use core::ffi::c_int;
use core::mem;
use libloading::os::unix as ll;

// "deep bind" ensure that a dylib being loaded will prefer its own symbols
// over symbols available at the global scope.
#[cfg(target_os = "linux")]
const DEEP_BIND: c_int = libc::RTLD_DEEPBIND;

#[cfg(not(target_os = "linux"))]
const DEEP_BIND: c_int = 0;

pub fn load(path: &Utf8Path) -> Result<()> {
    // SAFETY: We cannot guarantee anything about the initialization routines.
    // This is covered under the `unsafe-hot-reload` feature opt-in. We'll never
    // run termination routines.
    let lib = unsafe { ll::Library::open(Some(path), ll::RTLD_GLOBAL | ll::RTLD_NOW | DEEP_BIND) };

    let lib = lib.with_context(etx!("Failed to load {path:?}"))?;

    mem::forget(lib);

    Ok(())
}
