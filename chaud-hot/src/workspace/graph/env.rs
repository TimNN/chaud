use super::{KrateIdx, KrateIndex};
use crate::cargo::metadata::{Metadata, TargetKind, TargetName};
use crate::cargo::{Cargo, StdioMode};
use crate::util::assert::err_unreachable;
use crate::util::etx;
use anyhow::{Context as _, Result, bail, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use std::env::VarError;
use std::process::Command;
use std::{env, fs};

#[derive(Debug)]
pub struct BuildEnv {
    root: KrateIdx,
    bin: TargetName,
    chaud_dir: Utf8PathBuf,
    cargo: Cargo,
    flags: Box<[String]>,
    /// Feature flags from `chaud-rustc` that must be passed to cargo.
    rustc_features_env: Option<String>,
}

impl BuildEnv {
    pub(super) fn new(
        cargo: Cargo,
        feature_flags: Option<&'static str>,
        meta: &Metadata,
        index: &KrateIndex,
    ) -> Result<Self> {
        new_inner(cargo, feature_flags, meta, index).context("Failed to load build env")
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

    pub fn cargo_rustc(&self, mode: StdioMode) -> Command {
        let mut cmd = self.cargo.cmd("rustc", mode);
        cmd.args(&self.flags);
        if let Some(f) = &self.rustc_features_env {
            cmd.env("__CHAUD_RUSTC_FEATURE_FLAGS", f);
        }
        cmd
    }
}

fn new_inner(
    cargo: Cargo,
    ct_feature_flags: Option<&'static str>,
    meta: &Metadata,
    index: &KrateIndex,
) -> Result<BuildEnv> {
    let exe_file = Utf8PathBuf::try_from(std::env::current_exe()?)?;
    let exe_dir = exe_file.parent().context("exe has no parent")?;

    let bin = exe_file.file_stem().context("exe has no stem")?;

    let mut root = None;
    for pkg in meta.packages() {
        if pkg.manifest_path() != cargo.mani() {
            continue;
        }

        ensure!(root.is_none(), "Multiple packages for {:?}", cargo.mani());

        for t in pkg.targets() {
            if !t.kind().contains(&TargetKind::Bin) {
                continue;
            }

            if t.name() != bin {
                continue;
            }

            ensure!(
                root.is_none(),
                "Multiple `bin` candidates for {bin:?} in {:?}",
                cargo.mani().path()
            );

            let Some(krate) = index.get_pkg(pkg.name()) else {
                err_unreachable!();
            };

            root = Some((krate, t.name().clone()));
        }

        ensure!(
            root.is_some(),
            "No `bin` target for {bin:?} in {:?}",
            cargo.mani().path()
        );
    }
    let (root, bin) = root.with_context(etx!("No package for {:?}", cargo.mani().path()))?;

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
        "--profile",
        profile,
        "-Fchaud/unsafe-hot-reload",
    ];

    let rt_feature_flags = match env::var("CHAUD_FEATURE_FLAGS") {
        Ok(f) => Some(f),
        Err(VarError::NotPresent) => None,
        Err(VarError::NotUnicode(_)) => bail!("Invalid UTF-8 in CHAUD_FEATURE_FLAGS"),
    };
    let rt_feature_flags = rt_feature_flags.as_deref();

    if let (Some(ct), Some(rt)) = (ct_feature_flags, rt_feature_flags) {
        ensure!(
            ct == rt,
            "Compile-time and run-time CHAUD_FEATURE_FLAGS divereged. ct: {:?}, rt: {:?}",
            ct_feature_flags,
            rt_feature_flags
        );
    }

    let mut features_env = None;
    if let (Some(ct), None) = (ct_feature_flags, rt_feature_flags) {
        features_env = Some(ct.to_owned());
    }

    let feature_flags = ct_feature_flags.or(rt_feature_flags).unwrap_or("");

    let feature_flags =
        shlex::split(feature_flags).context("shlex of CHAUD_FEATURE_FLAGS failed")?;

    let flags = flags
        .into_iter()
        .map(|s| s.to_owned())
        .chain(feature_flags)
        .collect();

    let this = BuildEnv {
        root,
        bin,
        chaud_dir,
        cargo,
        flags,
        rustc_features_env: features_env,
    };

    log::trace!("{this:?}");

    Ok(this)
}
