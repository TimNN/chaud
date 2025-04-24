use super::TrackedSymbol;
use crate::hot::cargo::metadata::KrateName;
use crate::hot::dylib::Sym;
use crate::hot::util::assert::err_unreachable;
use crate::hot::util::etx;
use crate::hot::workspace::graph::{DylibIdx, KrateData};
use anyhow::{Context as _, Result, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use hashbrown::HashMap;
use jiff::Timestamp;
use std::fs;

enum State {
    Initial,
    Copied(Utf8PathBuf),
    Loaded,
}

pub struct DylibData {
    idx: DylibIdx,
    name: &'static KrateName<'static>,
    file: &'static Utf8Path,
    next: u32,
    mtime: Timestamp,
    state: State,
    tracked: HashMap<Sym, TrackedSymbol>,
}

impl DylibData {
    pub(super) fn new(krate: &'static KrateData) -> Result<Self> {
        new_inner(krate).with_context(etx!("Failed to int dylib data for {}", krate.pkg()))
    }

    pub(super) fn idx(&self) -> DylibIdx {
        self.idx
    }

    pub(super) fn maybe_copy(&mut self, lib_dir: &Utf8Path) -> Result<()> {
        maybe_copy_inner(self, lib_dir).with_context(etx!("Failed to copy {:?}", self.file))
    }
}

fn new_inner(krate: &'static KrateData) -> Result<DylibData> {
    let Some(paths) = krate.dylib_paths() else {
        err_unreachable!();
    };
    let Some(idx) = krate.dylib() else {
        err_unreachable!();
    };

    let name = krate.name();
    let file = paths.dylib_file();
    let mtime = dylib_mtime(file)?;

    log::trace!("Initialized {idx:?} with mtime {mtime:?} from {file:?}");

    Ok(DylibData {
        idx,
        name,
        file,
        next: 0,
        mtime,
        state: State::Initial,
        tracked: HashMap::new(),
    })
}

fn maybe_copy_inner(d: &mut DylibData, lib_dir: &Utf8Path) -> Result<()> {
    let mtime = dylib_mtime(d.file)?;

    if mtime == d.mtime {
        return Ok(());
    }

    let dst = lib_dir.join(d.name.lib_file_name_versioned(d.next));
    fs::copy(d.file, &dst)?;
    d.next = d.next.checked_add(1).context("Dylib version overflow")?;

    d.state = State::Copied(dst);

    Ok(())
}

fn dylib_mtime(path: &Utf8Path) -> Result<Timestamp> {
    let inner = || {
        let meta = path.metadata()?;
        ensure!(
            meta.is_file(),
            "Dylib is not a file ({:?})",
            meta.file_type()
        );
        let mtime = meta.modified()?;
        Ok(mtime.try_into()?)
    };

    inner().with_context(etx!("Failed to get mtime of {path:?}"))
}
