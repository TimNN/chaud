use crate::hot::handle::{ErasedFnPtr, ErasedHandle};
use anyhow::{Context as _, Result, ensure};
use jiff::Timestamp;
use std::ffi::CString;

enum State {
    Active,
    Mangled(CString),
    Loaded(ErasedFnPtr),
}

pub struct TrackedSymbol {
    handle: ErasedHandle,
    mtime: Timestamp,
    state: State,
}

impl TrackedSymbol {
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
                    "multiple mangled symbol candidates: {:?}, {:?}",
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
}
