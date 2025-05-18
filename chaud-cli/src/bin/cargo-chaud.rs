#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    reason = "less restrictions on build-time tools"
)]

use anyhow::{Context as _, Result};
use chaud_cli::{actual_args, link_pre_args, run};
use std::env;
use std::ffi::OsStr;
use std::process::Command;

fn main() -> Result<()> {
    let args = actual_args()?;

    // Remove a leading `chaud`, if present.
    let args = match args.split_at_checked(1) {
        Some(([head], tail)) if head == "chaud" => tail,
        _ => &args,
    };

    let mut args = args.splitn(2, |a| a == "--");
    let build_flags = args.next().unwrap_or(&[]);
    let run_flags = args.next().unwrap_or(&[]);

    let mut rust_flags = env::var_os("RUSTFLAGS").unwrap_or_default();
    if !rust_flags.is_empty() {
        rust_flags.push(" ");
    }
    rust_flags.push("-Clink-dead-code ");
    rust_flags.push(link_pre_args()?);

    let feature_flags =
        extract_feature_flags(build_flags).context("Failed to extract feature flags")?;

    let mut cmd = cargo();
    cmd.arg("run")
        .args(build_flags)
        .arg("-Fchaud/unsafe-hot-reload")
        .arg("--")
        .args(run_flags)
        .env("RUSTFLAGS", rust_flags)
        .env("RUSTC_BOOTSTRAP", "1")
        .env("CHAUD_FEATURE_FLAGS", feature_flags);

    run(cmd)
}

fn extract_feature_flags(args: &[String]) -> Result<String> {
    use lexopt::prelude::*;

    let mut parser = lexopt::Parser::from_args(args);

    let mut features = vec![];
    while let Some(arg) = parser.next()? {
        match arg {
            Short('F') | Long("features") => {
                features.push(format!("-F{}", parser.value()?.parse::<String>()?));
            }
            Long("all-features") => features.push("--all-features".to_owned()),
            Long("no-default-features") => features.push("--no-default-features".to_owned()),
            Short(_) | Long(_) => {
                parser.optional_value();
            }
            _ => {}
        }
    }

    Ok(shlex::try_join(features.iter().map(|s| s.as_str()))?)
}

fn cargo() -> Command {
    let cargo = env::var_os("CARGO");
    let cargo = cargo
        .as_ref()
        .map_or(OsStr::new("cargo"), |o| o.as_os_str());

    Command::new(cargo)
}
