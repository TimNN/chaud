use crate::util::CommandExt as _;
use anyhow::{Context as _, Result};
use camino::{Utf8Path, Utf8PathBuf};
use core::fmt;
use core::str::Chars;
use nanoserde::{DeJson, DeJsonErr, DeJsonState, DeJsonTok};
use std::process::{Command, Stdio};

#[derive(Debug, DeJson)]
pub struct Metadata {
    packages: Vec<Package>,
}

impl Metadata {
    pub fn load() -> Result<Self> {
        let buf = run_cargo().context("Failed to run `cargo metadata`")?;
        let md = Self::deserialize_json(&buf).context("Failed to parse `cargo metadata` output")?;
        Ok(md)
    }

    pub fn packages(&self) -> &[Package] {
        &self.packages
    }
}
#[derive(Debug, DeJson)]
pub struct Package {
    name: PackageName,
    version: String,
    manifest_path: ManifestPath,
    dependencies: Vec<Dependency>,
    targets: Vec<Target>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ManifestPath(Utf8PathBuf);

impl DeJson for ManifestPath {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        s.string(i)?;
        Ok(ManifestPath(s.strbuf.clone().into()))
    }
}

impl ManifestPath {
    pub fn path(&self) -> &Utf8Path {
        &self.0
    }
}

impl Package {
    pub fn name(&self) -> &PackageName {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn manifest_path(&self) -> &ManifestPath {
        &self.manifest_path
    }

    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    pub fn targets(&self) -> &[Target] {
        &self.targets
    }
}

#[derive(Debug, DeJson, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[nserde(transparent)]
pub struct PackageName(String);

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.0)
    }
}

#[derive(Debug, DeJson)]
pub struct Dependency {
    name: PackageName,
    kind: DependencyKind,
}

impl Dependency {
    pub fn name(&self) -> &PackageName {
        &self.name
    }

    pub fn kind(&self) -> DependencyKind {
        self.kind
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DependencyKind {
    Normal,
    Build,
    Other,
}

impl DeJson for DependencyKind {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        if s.tok == DeJsonTok::Null {
            s.next_tok(i)?;
            return Ok(Self::Normal);
        }
        s.string(i)?;
        match s.strbuf.as_ref() {
            "build" => Ok(Self::Build),
            _ => Ok(Self::Other),
        }
    }
}

#[derive(Debug, DeJson)]
pub struct Target {
    name: TargetName,
    kind: Vec<TargetKind>,
    #[nserde(proxy = "String")]
    src_path: Utf8PathBuf,
}

impl Target {
    pub fn name(&self) -> &TargetName {
        &self.name
    }

    pub fn kind(&self) -> &[TargetKind] {
        &self.kind
    }

    pub fn src_path(&self) -> &Utf8Path {
        &self.src_path
    }
}

#[derive(Debug, DeJson, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[nserde(transparent)]
pub struct TargetName(String);

impl PartialEq<str> for TargetName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl TargetName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TargetKind {
    Bin,
    Lib,
    RLib,
    CustomBuild,
    ProcMacro,
    Other,
}

impl DeJson for TargetKind {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        s.string(i)?;
        match s.strbuf.as_ref() {
            "bin" => Ok(Self::Bin),
            "lib" => Ok(Self::Lib),
            "rlib" => Ok(Self::RLib),
            "custom-build" => Ok(Self::CustomBuild),
            "proc-macro" => Ok(Self::ProcMacro),
            _ => Ok(Self::Other),
        }
    }
}

fn run_cargo() -> Result<String> {
    let mut cmd = Command::new("cargo");

    cmd.args(["metadata", "--format-version=1", "--no-deps"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    log::trace!("Running {cmd:?}");

    let output = cmd.stdout_str()?;

    Ok(output)
}
