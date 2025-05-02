use super::{KrateIdx, KrateIndex};
use crate::cargo::metadata::{Metadata, TargetKind, TargetName};
use crate::util::assert::err_unreachable;
use crate::util::etx;
use anyhow::{Context as _, Result, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

#[derive(Debug)]
pub struct BuildEnv {
    root: KrateIdx,
    bin: TargetName,
    chaud_dir: Utf8PathBuf,
    flags: Box<[String]>,
}

impl BuildEnv {
    pub(super) fn new(meta: &Metadata, index: &KrateIndex) -> Result<Self> {
        new_inner(meta, index).context("Failed to load build env")
    }

    pub fn root(&self) -> KrateIdx {
        self.root
    }

    pub fn bin(&self) -> &TargetName {
        &self.bin
    }

    pub fn chaud_dir(&self) -> &Utf8Path {
        &self.chaud_dir
    }

    pub fn flags(&self) -> &[String] {
        &self.flags
    }
}

fn new_inner(meta: &Metadata, index: &KrateIndex) -> Result<BuildEnv> {
    let exe_file = Utf8PathBuf::try_from(std::env::current_exe()?)?;
    let exe_dir = exe_file.parent().context("exe has no parent")?;

    let bin = exe_file.file_stem().context("exe has no stem")?;

    let mut root = None;
    for pkg in meta.packages() {
        for t in pkg.targets() {
            if !t.kind().contains(&TargetKind::Bin) {
                continue;
            }

            if t.name() != bin {
                continue;
            }

            ensure!(root.is_none(), "Multiple `bin` candidates for {bin:?}");

            let Some(krate) = index.get_pkg(pkg.name()) else {
                err_unreachable!();
            };

            root = Some((krate, t.name().clone()));
        }
    }
    let (root, bin) = root.with_context(etx!("No `bin` found for {bin:?}"))?;

    let mut profile = exe_dir
        .components()
        .next_back()
        .context("missing profile component")?
        .as_str();
    if profile == "debug" {
        profile = "dev";
    }

    let chaud_dir = exe_dir.join("chaud");
    fs::create_dir_all(&chaud_dir)?;

    let flags = [
        "--bin",
        bin.as_str(),
        "--features",
        "chaud/unsafe-hot-reload",
        "--profile",
        profile,
    ];

    let flags = flags.into_iter().map(|s| s.to_owned()).collect();

    let this = BuildEnv { root, bin, chaud_dir, flags };

    log::trace!("{this:?}");

    Ok(this)
}
