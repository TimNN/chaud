use super::metadata::ManifestPath;
use std::env;
use std::ffi::OsString;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct Cargo {
    cargo: OsString,
    mani: ManifestPath,
}

#[derive(Copy, Clone)]
pub enum StdioMode {
    /// Standard output is captured, standard error is ignored.
    QuietCapture,
    /// Standard output is captured, standard error is inherited.
    LoudCapture,
}

impl Cargo {
    pub fn new(mani: ManifestPath) -> Self {
        let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());

        Self { cargo, mani }
    }

    pub fn mani(&self) -> &ManifestPath {
        &self.mani
    }

    pub fn cmd(&self, sub: &str, mode: StdioMode) -> Command {
        let mut cmd = Command::new(&self.cargo);
        cmd.args([sub, "--manifest-path", self.mani.path().as_str()])
            .stdout(Stdio::null());

        match mode {
            StdioMode::QuietCapture => cmd.stdout(Stdio::piped()).stderr(Stdio::null()),
            StdioMode::LoudCapture => cmd.stdout(Stdio::piped()).stderr(Stdio::inherit()),
        };

        cmd
    }
}
