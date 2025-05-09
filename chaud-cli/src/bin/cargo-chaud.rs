#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    reason = "less restrictions on build-time tools"
)]

use anyhow::{Context as _, Result};
use chaud_cli::{actual_args, link_pre_args, run, stdout};
use nanoserde::DeJson;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::process::{Command, Stdio};
use std::{env, fs};
use toml_edit::{DocumentMut, Table};

fn main() -> Result<()> {
    let args = actual_args()?;

    // Remove a leading `chaud`, if present.
    let args = match args.split_at_checked(1) {
        Some(([head], tail)) if head == "chaud" => tail,
        _ => &args,
    };

    match args.split_at_checked(1) {
        Some(([head], tail)) if head == "cleanup" => return cleanup(tail),
        _ => {}
    }

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

fn cleanup(args: &[String]) -> Result<()> {
    let mut cmd = cargo();
    cmd.args(["metadata", "--format-version=1", "--no-deps"])
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let out = stdout(cmd)?;

    let md = Metadata::deserialize_json(&out).context("Invalid `cargo metadata` output")?;

    for pkg in &md.packages {
        unpatch(pkg).with_context(|| format!("Failed to clean up `{}`", pkg.name))?;
    }

    Ok(())
}

fn unpatch(pkg: &Package) -> Result<()> {
    if !pkg.version.contains("chaud-") {
        return Ok(());
    }

    let buf = fs::read_to_string(&pkg.manifest_path)?;

    if let Some(unpatched) = apply_unpatch(&pkg.version, &buf)? {
        fs::write(&pkg.manifest_path, unpatched)?;
        eprintln!("Cleaned up `{}`", pkg.name);
    }

    Ok(())
}

fn apply_unpatch(version: &str, manifest: &str) -> Result<Option<String>> {
    let Some((head, tail)) = version.split_once('+') else {
        return Ok(None);
    };

    let mut parts: Vec<_> = tail.split('.').map(Cow::Borrowed).collect();

    let mut found_chaud = false;
    let mut is_workspace = false;

    parts.retain(|p| {
        if p.starts_with("chaud-") {
            found_chaud = true;
            is_workspace |= p.starts_with("chaud-ws-");
            false
        } else {
            true
        }
    });

    if !found_chaud {
        return Ok(None);
    }

    let mut doc: DocumentMut = manifest.parse()?;
    let version = doc
        .get_mut("package")
        .context("Missing [package]")?
        .get_mut("version")
        .context("Missing `version`")?;
    if is_workspace {
        let mut t = Table::new();
        t.set_dotted(true);
        t.insert("workspace", toml_edit::value(true));

        *version = t.into();
    } else {
        let val = match parts.is_empty() {
            true => head.to_owned(),
            false => format!("{head}+{}", parts.join(".")),
        };

        *version = toml_edit::value(val);
    }

    Ok(Some(doc.to_string()))
}

fn cargo() -> Command {
    let cargo = env::var_os("CARGO");
    let cargo = cargo
        .as_ref()
        .map_or(OsStr::new("cargo"), |o| o.as_os_str());

    Command::new(cargo)
}

#[derive(Debug, DeJson)]
struct Metadata {
    packages: Vec<Package>,
}

#[derive(Debug, DeJson)]
pub struct Package {
    name: String,
    version: String,
    manifest_path: String,
}

#[cfg(test)]
mod tests {
    use super::apply_unpatch;
    use pretty_assertions::assert_eq;

    #[track_caller]
    fn check(version: &str, input: &str, expected: &str) {
        let input = trim_lines(input);
        let expected = trim_lines(expected);

        let output = apply_unpatch(version, &input).unwrap();
        assert_eq!(expected, output.unwrap_or(input));
    }

    fn trim_lines(s: &str) -> String {
        s.split('\n').map(str::trim).collect::<Vec<_>>().join("\n")
    }

    #[test]
    fn simple() {
        check(
            "1.0+chaud-0",
            r#"
                [package]
                version = "N/A"
            "#,
            r#"
                [package]
                version = "1.0"
            "#,
        );
    }

    #[test]
    fn workspace() {
        check(
            "1.0+chaud-ws-0",
            r#"
                [package]
                version = "N/A"
            "#,
            r#"
                [package]
                version.workspace = true
            "#,
        );
    }

    #[test]
    fn keep_meta() {
        check(
            "1.0+foo.chaud-hello.bar",
            r#"
                [package]
                version = "N/A"
            "#,
            r#"
                [package]
                version = "1.0+foo.bar"
            "#,
        );
    }
}
