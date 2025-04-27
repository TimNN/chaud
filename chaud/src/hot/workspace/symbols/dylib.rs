use super::TrackedSymbol;
use crate::hot::cargo::metadata::KrateName;
use crate::hot::dylib::{Library, Sym, exported_symbols};
use crate::hot::handle::ErasedHandle;
use crate::hot::util::assert::err_unreachable;
use crate::hot::util::etx;
use crate::hot::workspace::graph::{BuildEnv, DylibIdx, KrateData};
use anyhow::{Context as _, Result, bail, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use core::{fmt, mem};
use hashbrown::{HashMap, hash_map};
use jiff::Timestamp;
use std::fs;

enum State {
    Initial,
    Error,
    Copied(Utf8PathBuf),
    Loaded(Utf8PathBuf, Library),
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

impl fmt::Display for DylibData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl DylibData {
    pub(super) fn new(krate: &'static KrateData) -> Result<Self> {
        new_inner(krate).with_context(etx!("Failed to init dylib data for {}", krate.pkg()))
    }

    pub(super) fn idx(&self) -> DylibIdx {
        self.idx
    }

    pub(super) fn maybe_copy(&mut self, env: &BuildEnv) -> Result<()> {
        let res = maybe_copy_inner(self, env);

        if res.is_err() {
            self.state = State::Error;
        }

        res.with_context(etx!("Failed to copy dylib for {self}"))
    }

    pub(super) fn resolve_symbols(&mut self) -> Result<()> {
        resolve_symbols_inner(self).with_context(etx!("Failed to resolve symbols for {self}"))
    }

    pub(super) fn load(&mut self) -> Result<()> {
        let res = load_inner(self);

        if res.is_err() {
            self.state = State::Error;
        }

        res.with_context(etx!("Failed to load {self}"))
    }

    pub(super) fn load_symbols(&mut self) -> Result<()> {
        let lib = match self.state {
            State::Initial => return Ok(()),
            State::Error => bail!("LoadSym: Invalid state: Error"),
            State::Copied(_) => bail!("LoadSym: Invalid state: Copied"),
            State::Loaded(_, lib) => lib,
        };

        for t in self.tracked.values_mut() {
            t.load(self.mtime, lib)
                .with_context(etx!("Failed to load symbols for {}", self.name))?;
        }
        Ok(())
    }

    pub(super) fn activate_symbols(&mut self, count: &mut u32) -> Result<()> {
        for t in self.tracked.values_mut() {
            t.activate(count)
                .with_context(etx!("Failed to activate symbols for {}", self.name))?;
        }
        Ok(())
    }

    pub(super) fn register(&mut self, sym: Sym, handle: ErasedHandle) -> Result<()> {
        let mtime = self.mtime_for_new_sym();

        let t = match self.tracked.entry(sym) {
            hash_map::Entry::Occupied(_) => bail!("Already registered: {sym:?}"),
            hash_map::Entry::Vacant(entry) => entry.insert(TrackedSymbol::new(mtime, sym, handle)),
        };

        if let Err(e) = eagerly_update(t, self.mtime, &self.state) {
            log::warn!("Failed to eagerly update {sym:?}: {e:#}");
        }

        Ok(())
    }

    fn mtime_for_new_sym(&self) -> Timestamp {
        match self.state {
            State::Initial => self.mtime,
            _ => Timestamp::UNIX_EPOCH,
        }
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

    log::trace!("Initialized {name} with mtime {mtime:?} from {file:?}");

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

fn maybe_copy_inner(d: &mut DylibData, env: &BuildEnv) -> Result<()> {
    let mtime = dylib_mtime(d.file)?;

    if mtime == d.mtime {
        return Ok(());
    }

    let dst = env.chaud_dir().join(d.name.lib_file_name_versioned(d.next));

    fs::copy(d.file, &dst)?;
    d.next = d.next.checked_add(1).context("Dylib version overflow")?;

    log::trace!("Copied dylib for {d} with mtime {mtime:?} to {dst:?}");

    d.mtime = mtime;
    d.state = State::Copied(dst);

    Ok(())
}

fn eagerly_update(t: &mut TrackedSymbol, mtime: Timestamp, state: &State) -> Result<()> {
    if matches!(state, State::Initial) {
        return Ok(());
    }

    let path = state.copied_path()?;

    exported_symbols(path, |sym, mangled| {
        if sym == t.sym() {
            t.mangled(mtime, mangled)?;
        }
        Ok(())
    })?;

    ensure!(t.mtime() == mtime, "Symbol not resolved");

    let State::Loaded(_, lib) = *state else {
        return Ok(());
    };

    t.load(mtime, lib)?;
    t.activate(&mut 0)
}

fn resolve_symbols_inner(d: &mut DylibData) -> Result<()> {
    if !d.tracked.values().any(|t| t.mtime() != d.mtime) {
        return Ok(());
    }

    let path = d.state.copied_path()?;

    exported_symbols(path, |sym, mangled| {
        if let Some(t) = d.tracked.get_mut(&sym.key()) {
            t.mangled(d.mtime, mangled)?;
        }

        Ok(())
    })?;

    for t in d.tracked.values() {
        ensure!(t.mtime() == d.mtime, "Symbol not resolved: {:?}", t.sym());
    }

    Ok(())
}

fn load_inner(d: &mut DylibData) -> Result<()> {
    let path = match &mut d.state {
        State::Initial => return Ok(()),
        State::Error => bail!("Load: Invalid state: Error"),
        State::Copied(path) => mem::take(path),
        State::Loaded(..) => return Ok(()),
    };

    let lib = Library::load(&path)?;

    log::trace!("Loaded {d} from {path:?}");

    d.state = State::Loaded(path, lib);

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

impl State {
    fn copied_path(&self) -> Result<&Utf8Path> {
        match self {
            State::Initial => bail!("CopiedPath: Invalid state: Initial"),
            State::Error => bail!("CopiedPath: Invalid state: Error"),
            State::Copied(path) | State::Loaded(path, _) => Ok(path),
        }
    }
}
