#![allow(
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "less restrictions on internal tests"
)]

use expect_test::expect_file;
use fs_extra::dir::CopyOptions;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, fs};

fn main() {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let dst = tmp.join("demo");
    let target = tmp.join("demo_target");

    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    let src = root.join("demo");
    let chaud = root.join("chaud");
    let chaud = chaud.as_os_str().to_str().expect("not-utf8 root");

    if dst.exists() {
        fs::remove_dir_all(&dst).expect("remove dst failed");
    }
    fs::create_dir_all(&dst).expect("crate dst failed");

    fs_extra::dir::copy(
        src,
        &dst,
        &CopyOptions { content_only: true, ..Default::default() },
    )
    .expect("demo copy failed");

    let cargo = Cargo { target, manifest: dst.join("Cargo.toml") };

    let mani = fs::read_to_string(&cargo.manifest).expect("failed to read manifest");
    let mani = mani.replace("../chaud", chaud);
    fs::write(&cargo.manifest, mani).expect("failed to write manifest");

    test_expansion(&cargo);
}

fn test_expansion(cargo: &Cargo) {
    let expect = expect_file!["../../demo/expanded_cold.rs"];
    expect.assert_eq(&expand(cargo, ""));

    let expect = expect_file!["../../demo/expanded_hot.rs"];
    expect.assert_eq(&expand(cargo, "chaud/unsafe-hot-reload"));

    let expect = expect_file!["../../demo/expanded_reload.rs"];
    expect.assert_eq(&expand(
        cargo,
        "chaud/unsafe-hot-reload,chaud/internal-is-reload",
    ));
}

fn expand(cargo: &Cargo, features: &str) -> String {
    let mut cmd = cargo.base_cmd("rustc");
    cmd.stdout(Stdio::piped())
        .args([
            "-p",
            "expand",
            &format!("--features={features}"),
            "--",
            "-Zunpretty=expanded",
        ])
        .env("RUSTC_BOOTSTRAP", "1");

    let out = cmd.output().expect("cargo expand failed");
    assert!(out.status.success(), "cargo expand failed: {}", out.status);

    let mut cmd = Command::new("rustfmt");
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    let mut proc = cmd.spawn().expect("rusfmt failed");
    proc.stdin
        .take()
        .expect("unreachable")
        .write_all(&out.stdout)
        .expect("rustfmt write failed");
    let out = proc.wait_with_output().expect("rustfmt wait failed");
    assert!(out.status.success(), "rustfmt failed: {}", out.status);

    let formatted = String::from_utf8(out.stdout).expect("cargo expand failed: invalid utf8");

    let mut without = formatted
        .lines()
        // Only `ctor!` produces `link_section` attributes, which are platfrom
        // dependent, so just remove them.
        .filter(|l| !l.contains("#[link_section"))
        .collect::<Vec<_>>()
        .join("\n");
    without.push('\n');
    without
}

struct Cargo {
    manifest: PathBuf,
    target: PathBuf,
}

impl Cargo {
    pub fn base_cmd(&self, sub: &str) -> Command {
        let mut cmd = Command::new("cargo");
        cmd.stdin(Stdio::null())
            .stderr(Stdio::inherit())
            .args([sub, "--manifest-path"])
            .arg(&self.manifest)
            .arg("--target-dir")
            .arg(&self.target);

        cmd
    }
}
