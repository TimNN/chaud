use super::metadata::PackageName;
use crate::hot::workspace::graph::BuildEnv;
use anyhow::{Context as _, Result, ensure};
use std::process::{Command, Stdio};

pub fn build(
    env: &BuildEnv,
    tracked: impl IntoIterator<Item = &'static PackageName>,
) -> Result<()> {
    build_inner(env, tracked).context("`cargo build` failed")
}

fn build_inner(
    env: &BuildEnv,
    tracked: impl IntoIterator<Item = &'static PackageName>,
) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--profile", env.profile()]);

    for pkg in tracked {
        cmd.args(["-p", pkg.as_str()]);
    }

    cmd.stdin(Stdio::null());

    if !log::log_enabled!(log::Level::Trace) {
        cmd.stderr(Stdio::null());
        cmd.stdout(Stdio::null());
    }

    log::debug!("Executing: {cmd:?}");

    let st = cmd.status()?;
    ensure!(st.success(), "Cargo failed: {st}");

    Ok(())
}
