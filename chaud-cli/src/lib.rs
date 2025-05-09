/*!
# `chaud-cli` (**library**)

Implementation detail of [Chaud 🔥](https://docs.rs/chaud).

**Do not use this crate directly!**

This crate implements shared functionality for the Chaud CLI Tools.
*/
#![allow(
    clippy::missing_errors_doc,
    reason = "less restrictions on build-time tools"
)]

use anyhow::{Result, bail, ensure};
use core::fmt;
use std::env;
use std::process::Command;
use std::sync::LazyLock;

pub fn actual_args() -> Result<Vec<String>> {
    let mut args = vec![];
    for arg in env::args_os().skip(1) {
        let arg = match arg.into_string() {
            Ok(arg) => arg,
            Err(arg) => bail!("Non UTF-8 argument: {arg:?}"),
        };
        args.push(arg);
    }
    Ok(args)
}

pub fn link_pre_args() -> Result<&'static str> {
    if cfg!(target_os = "macos") {
        Ok("-Zpre-link-args=-Wl,-all_load")
    } else if cfg!(unix) {
        Ok("-Zpre-link-args=-Wl,--whole-archive")
    } else {
        bail!("Hot-reloading not supported on the current platform");
    }
}

pub fn run(mut cmd: Command) -> Result<()> {
    verbose!("Executing: {cmd:?}");

    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => bail!("Failed to spawn ({e}): {cmd:?}"),
    };
    ensure!(status.success(), "Failed to run ({status}): {cmd:?}");
    Ok(())
}

pub fn stdout(mut cmd: Command) -> Result<String> {
    verbose!("Executing: {cmd:?}");

    let out = match cmd.output() {
        Ok(s) => s,
        Err(e) => bail!("Failed to spawn ({e}): {cmd:?}"),
    };
    ensure!(
        out.status.success(),
        "Failed to run ({}): {cmd:?}",
        out.status
    );

    match String::from_utf8(out.stdout) {
        Ok(s) => Ok(s),
        Err(_) => bail!("Non-UTF8 output: {cmd:?}"),
    }
}

pub fn verbose(x: impl fmt::Display) {
    static VERBOSE: LazyLock<bool> = LazyLock::new(|| env::var_os("CHAUD_CLI_VERBOSE").is_some());

    if *VERBOSE {
        eprintln!("{x}");
    }
}

#[macro_export]
macro_rules! verbose {
    ($($t:tt)*) => { $crate::verbose(format_args!($($t)*)); }
}
