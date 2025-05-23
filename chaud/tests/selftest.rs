#![allow(
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "less restrictions on internal tests"
)]

use fs_extra::dir::CopyOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

fn main() {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let target = tmp.join("demo_target");
    let dst = tmp.join("demo_dst");

    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();

    let demo = root.join("demo");
    let chaud = root.join("chaud");
    let chaud = chaud.as_os_str().to_str().expect("not-utf8 root");

    let dirs = Dirs { chaud, demo: &demo, dst: &dst, target: &target };

    eprintln!("\n---\nSELFTEST: `cargo chaud`\n---");
    selftest(&dirs, &["chaud", "-Fselftest"], &[]);

    eprintln!("\n---\nSELFTEST: `cargo chaud --release`\n---");
    selftest(&dirs, &["chaud", "-Fselftest", "--release"], &[]);

    eprintln!("\n---\nSELFTEST: `RUSTC_WRAPPER=... cargo run`\n---");
    selftest(
        &dirs,
        &["run", "-Fselftest,chaud/unsafe-hot-reload"],
        &[("RUSTC_WRAPPER", "chaud-rustc")],
    );

    eprintln!("\n---\nSELFTEST: `RUSTC_WRAPPER=... cargo run --release`\n---");
    selftest(
        &dirs,
        &["run", "-Fselftest,chaud/unsafe-hot-reload", "--release"],
        &[("RUSTC_WRAPPER", "chaud-rustc")],
    );
}

struct Dirs<'a> {
    chaud: &'a str,
    demo: &'a Path,
    dst: &'a Path,
    target: &'a Path,
}

fn selftest(dirs: &Dirs, args: &[&str], env: &[(&str, &str)]) {
    if dirs.dst.exists() {
        fs::remove_dir_all(dirs.dst).expect("remove dst failed");
    }
    fs::create_dir_all(dirs.dst).expect("crate dst failed");

    fs_extra::dir::copy(
        dirs.demo,
        dirs.dst,
        &CopyOptions { content_only: true, ..Default::default() },
    )
    .expect("demo copy failed");

    let manifest = dirs.dst.join("Cargo.toml");

    let mani = fs::read_to_string(&manifest).expect("failed to read manifest");
    let mani = mani.replace("../chaud", dirs.chaud);
    fs::write(&manifest, mani).expect("failed to write manifest");

    run(dirs, args, env);
}

fn run(dirs: &Dirs, args: &[&str], env: &[(&str, &str)]) {
    let mut cmd = Command::new("cargo");
    cmd.stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .args(args)
        .arg("--target-dir")
        .arg(dirs.target)
        .envs(env.iter().copied())
        .current_dir(dirs.dst);

    let status = cmd.status().expect("cargo failed");
    assert!(status.success(), "cargo failed: {status}");
}
