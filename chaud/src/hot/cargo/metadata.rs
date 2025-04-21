use crate::hot::util::CommandExt as _;
use anyhow::{Context as _, Result};
use camino::{Utf8Path, Utf8PathBuf};
use core::str::Chars;
use nanoserde::{DeJson, DeJsonErr, DeJsonState};
use std::borrow::Cow;
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
        s.string(i)?;
        match s.strbuf.as_ref() {
            "normal" => Ok(Self::Normal),
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

impl<'a> KrateName<'a> {
    pub fn borrowed(name: &'a str) -> KrateName<'a> {
        Self(Cow::Borrowed(name))
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

    println!("{output}");

    Ok(output)
}
