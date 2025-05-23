#![allow(
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "less restrictions on internal tests"
)]

use std::fs::FileTimes;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;
use std::{env, fs};
use walkdir::WalkDir;

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

    eprintln!("\n---\nSELFTEST: `RUSTC_WRAPPER=\"...\" cargo run`\n---");
    selftest(
        &dirs,
        &["run", "-Fselftest,chaud/unsafe-hot-reload"],
        &[("RUSTC_WRAPPER", "chaud-rustc")],
    );

    eprintln!("\n---\nSELFTEST: `RUSTC_WRAPPER=\"...\" cargo run --release`\n---");
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
    sync(dirs.demo, dirs.dst);

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
        .env("CARGO_TARGET_DIR", dirs.target)
        .envs(env.iter().copied())
        .current_dir(dirs.dst);

    let status = cmd.status().expect("cargo failed");
    assert!(status.success(), "cargo failed: {status}");
}

fn sync(src: &Path, dst: &Path) {
    if dst.exists() {
        fs::remove_dir_all(dst).expect("remove dst failed");
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).expect("create dst parent failed");
    }

    let mut buf = PathBuf::new();

    let times = FileTimes::new().set_modified(SystemTime::now());

    for entry in WalkDir::new(src) {
        let entry = entry.expect("walking src failed");
        let rel = entry
            .path()
            .strip_prefix(src)
            .expect("src not parent of entry");

        buf.clear();
        buf.push(dst);
        buf.push(rel);

        if entry.file_type().is_dir() {
            fs::create_dir(&buf).expect("create dir in dst failed");
            continue;
        }
        assert!(entry.file_type().is_file());

        fs::copy(entry.path(), &buf).expect("copy file to dst failed");
        let f = fs::OpenOptions::new()
            .create(false)
            .append(true)
            .open(&buf)
            .expect("open file in dst failed");

        // `fs::copy` copies mtimes. We need to update the mtime, otherwise
        // the mtime moves back in time from Cargo's perspective, which breaks
        // dirty detection.
        f.set_times(times).expect("set times in dst failed");
    }
}
