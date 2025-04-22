use super::TrackedSymbol;
use crate::hot::dylib::Sym;
use crate::hot::util::assert::err_unreachable;
use crate::hot::util::etx;
use crate::hot::workspace::graph::{DylibIdx, KrateData};
use anyhow::{Context as _, Result, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use hashbrown::HashMap;
use std::time::SystemTime;

enum State {
    Initial,
    Copied(Utf8PathBuf),
    Loaded,
}

pub struct DylibData {
    idx: DylibIdx,
    file: Utf8PathBuf,
    mtime: SystemTime,
    state: State,
    tracked: HashMap<Sym, TrackedSymbol>,
}

impl DylibData {
    pub(super) fn new(krate: &KrateData) -> Result<Self> {
        new_inner(krate).with_context(etx!("Failed to int dylib data for {}", krate.pkg()))
    }

    pub(super) fn idx(&self) -> DylibIdx {
        self.idx
    }
}

fn new_inner(krate: &KrateData) -> Result<DylibData> {
    let Some(paths) = krate.dylib_paths() else {
        err_unreachable!();
    };
    let Some(idx) = krate.dylib() else {
        err_unreachable!();
    };

    let file = paths.dylib_file().to_owned();
    let mtime = dylib_mtime(&file)?;

    Ok(DylibData {
        idx,
        file,
        mtime,
        state: State::Initial,
        tracked: HashMap::new(),
    })
}

fn dylib_mtime(path: &Utf8Path) -> Result<SystemTime> {
    let inner = || {
        let meta = path.metadata()?;
        ensure!(
            meta.is_file(),
            "Dylib is not a file ({:?})",
            meta.file_type()
        );
        let mtime = meta.modified()?;
        Ok(mtime)
    };

    inner().with_context(etx!("Failed to get mtime of {path:?}"))
}
