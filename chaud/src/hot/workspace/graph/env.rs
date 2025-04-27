use crate::hot::cargo::metadata::Metadata;
use anyhow::{Context as _, Result};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

#[derive(Debug)]
pub struct BuildEnv {
    profile: String,
    lib_dir: Utf8PathBuf,
    chaud_dir: Utf8PathBuf,
}

impl BuildEnv {
    pub(super) fn new(meta: &Metadata) -> Result<Self> {
        new_inner(meta).context("Failed to load build env")
    }

    pub fn profile(&self) -> &str {
        &self.profile
    }

    pub fn lib_dir(&self) -> &Utf8Path {
        &self.lib_dir
    }
}

fn new_inner(_meta: &Metadata) -> Result<BuildEnv> {
    let exe_file = Utf8PathBuf::try_from(std::env::current_exe()?)?;
    let exe_dir = exe_file.parent().context("exe has no parent")?;

    let mut profile = exe_dir
        .components()
        .next_back()
        .context("missing profile component")?
        .as_str();
    if profile == "debug" {
        profile = "dev";
    }
    let profile = profile.to_owned();

    let chaud_dir = exe_dir.join("chaud");
    fs::create_dir_all(&chaud_dir)?;

    let this = BuildEnv { profile, lib_dir: exe_dir.to_owned(), chaud_dir };

    log::debug!("{this:?}");

    Ok(this)
}
