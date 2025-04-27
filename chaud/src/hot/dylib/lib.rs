use crate::hot::handle::{ErasedFnPtr, RawErasedFnPtr};
use crate::hot::util::etx;
use anyhow::{Context as _, Result};
use camino::Utf8Path;
use core::ffi::CStr;
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

    pub fn get(self, name: &CStr) -> Result<ErasedFnPtr> {
        get_inner(self.0, name).with_context(etx!("Failed to get {name:?}"))
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

fn get_inner(lib: &ll::Library, name: &CStr) -> Result<ErasedFnPtr> {
    // SAFETY: The symbol is immediately converted into a raw pointer, so its
    // type doesn't really matter.
    let f = unsafe { lib.get::<RawErasedFnPtr>(name.to_bytes_with_nul()) }?;
    let f: RawErasedFnPtr = f.as_raw_ptr();

    // SAFETY: We assume that `name` resolved to a function pointer. This is
    // covered under the `unsafe-hot-reload` feature opt-in.
    let f = unsafe { ErasedFnPtr::from_raw_maybe_null(f) };

    f.context("Loaded symbol was null")
}
