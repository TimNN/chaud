use crate::hot::util::CommandExt as _;
use anyhow::{Context as _, Result};
use camino::{Utf8Path, Utf8PathBuf};
use core::fmt;
use core::str::Chars;
use nanoserde::{DeJson, DeJsonErr, DeJsonState, DeJsonTok};
use std::borrow::Cow;
use std::env::consts::{DLL_EXTENSION, DLL_PREFIX};
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
    #[nserde(proxy = "String")]
    manifest_path: Utf8PathBuf,
    dependencies: Vec<Dependency>,
    targets: Vec<Target>,
}

impl Package {
    pub fn name(&self) -> &PackageName {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn manifest_path(&self) -> &Utf8Path {
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

impl PackageName {
    pub fn to_krate(&self) -> KrateName<'static> {
        KrateName(Cow::Owned(self.0.replace('-', "_")))
    }
}

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
    kind: Vec<TargetKind>,
    #[nserde(proxy = "String")]
    src_path: Utf8PathBuf,
}

impl Target {
    pub fn kind(&self) -> &[TargetKind] {
        &self.kind
    }

    pub fn src_path(&self) -> &Utf8Path {
        &self.src_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KrateName<'a>(Cow<'a, str>);

impl DeJson for KrateName<'_> {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        s.string(i)?;
        Ok(KrateName(Cow::Owned(s.strbuf.clone())))
    }
}

impl fmt::Display for KrateName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.0)
    }
}

impl<'a> KrateName<'a> {
    pub fn borrowed(name: &'a str) -> KrateName<'a> {
        Self(Cow::Borrowed(name))
    }

    pub fn lib_file_name(&self) -> String {
        self.lib_file_name_suffix("")
    }

    pub fn lib_file_name_versioned(&self, version: u32) -> String {
        self.lib_file_name_suffix(format_args!(".{version}"))
    }

    fn lib_file_name_suffix(&self, suffix: impl fmt::Display) -> String {
        format!("{DLL_PREFIX}{}{}.{DLL_EXTENSION}", self.0, suffix)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TargetKind {
    Dylib,
    CustomBuild,
    Other,
}

impl DeJson for TargetKind {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        s.string(i)?;
        match s.strbuf.as_ref() {
            "dylib" => Ok(Self::Dylib),
            "custom-build" => Ok(Self::CustomBuild),
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
