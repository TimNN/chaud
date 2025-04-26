use crate::hot::dylib::{Library, Sym};
use crate::hot::handle::{ErasedFnPtr, ErasedHandle};
use crate::hot::util::etx;
use anyhow::{Context as _, Result, bail, ensure};
use jiff::Timestamp;
use std::ffi::CString;

#[derive(Debug)]
enum State {
    Active,
    Mangled(CString),
    Loaded(ErasedFnPtr),
}

pub struct TrackedSymbol {
    sym: Sym,
    handle: ErasedHandle,
    mtime: Timestamp,
    state: State,
}

impl TrackedSymbol {
    pub(super) fn new(mtime: Timestamp, sym: Sym, handle: ErasedHandle) -> Self {
        Self { sym, handle, mtime, state: State::Active }
    }

    pub(super) fn sym(&self) -> Sym {
        self.sym
    }

    pub(super) fn mtime(&self) -> Timestamp {
        self.mtime
    }

    pub(super) fn mangled(&mut self, mtime: Timestamp, mangled: &[u8]) -> Result<()> {
        if mtime == self.mtime {
            if let State::Mangled(ref m) = self.state {
                // FIXME(https://github.com/rust-lang/rust/issues/134915):
                // Switch to `ByteStr` for formatting `mangled` once stable.
                ensure!(
                    m.as_bytes() == mangled,
                    "multiple mangled symbol candidates for {:?}: {:?}, {:?}",
                    self.sym,
                    m,
                    String::from_utf8_lossy(mangled)
                );
            }
            return Ok(());
        }

        let mangled = CString::new(mangled)
            .ok()
            .context("mangled symbol name contained nul byte")?;

        self.state = State::Mangled(mangled);
        self.mtime = mtime;

        Ok(())
    }

    pub(super) fn load(&mut self, mtime: Timestamp, lib: Library) -> Result<()> {
        load_inner(self, mtime, lib).with_context(etx!(
            "Failed to load {:?} ({:?})",
            self.sym,
            self.state
        ))
    }

    pub(super) fn activate(&mut self) -> Result<()> {
        let f = match self.state {
            State::Active => return Ok(()),
            State::Mangled(_) => bail!("Failed to activate {:?}: Invalid state: Mangled", self.sym),
            State::Loaded(f) => f,
        };

        // SAFETY: We assume that `f` has the correct type. This is covered
        // under the `unsafe-hot-reload` feature opt-in.
        unsafe { self.handle.set(f) };

        self.state = State::Active;

        Ok(())
    }
}

fn load_inner(t: &mut TrackedSymbol, mtime: Timestamp, lib: Library) -> Result<()> {
    ensure!(t.mtime == mtime, "mtime outdated");
    let mangled = match &t.state {
        State::Mangled(m) => m,
        State::Loaded(_) | State::Active => return Ok(()),
    };

    t.state = State::Loaded(lib.get(mangled)?);

    Ok(())
}
