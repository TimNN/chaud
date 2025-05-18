use super::StdioMode;
use crate::util::CommandExt as _;
use crate::util::assert::err_unreachable;
use crate::workspace::graph::BuildEnv;
use anyhow::{Context as _, Result, bail, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use core::iter::Peekable;
use hashbrown::HashMap;
use nanoserde::DeJson;
use std::io;
use std::process::Command;
use std::time::{Instant, SystemTime};

pub struct Builder {
    cmd: Command,
    linker: Linker,
    initial: HashMap<Utf8PathBuf, SystemTime>,
    latest: Vec<Utf8PathBuf>,
}

struct Linker {
    env_clear: Box<[String]>,
    env_set: Box<[(String, String)]>,
    bin: String,
    arg_pre: Box<[String]>,
    arg_post: Box<[String]>,
}

impl Builder {
    pub fn init(env: &BuildEnv) -> Result<Self> {
        init_inner(env).context("Failed to init Builder")
    }

    pub fn link_latest(&self, dst: &Utf8Path) -> Result<()> {
        link(dst, &self.linker, &self.latest)
    }

    pub fn build(&mut self) -> Result<()> {
        let parts = extract_link_args(&mut self.cmd).context("Build failed")?;

        // We need to check this, because the linker args won't be re-printed for a
        // fully fresh build, and we need to avoid clearing `latest_libs` in that
        // case.
        if parts.is_empty() {
            log::trace!("Empty output, rlibs seem fresh");
            return Ok(());
        }

        self.latest.clear();
        extract_libs(parts, &self.initial, |p, _| {
            log::trace!("Found new rlib: {p:?}");
            self.latest.push(p);
        });

        log::debug!("Found {} new rlibs", self.latest.len());
        Ok(())
    }
}

fn init_inner(env: &BuildEnv) -> Result<Builder> {
    verify_fresh(env).context("Failed to check freshness")?;

    let mut cmd = cargo_cmd(env);
    cmd.args(["--", "--print=link-args"]);
    cmd.arg(format!(
        r#"--cfg=chaud_force_dirty="{}""#,
        current_time_nanos()?
    ));

    let (linker, initial) = extract_linker(cmd).context("Failed to extract linker")?;

    let mut cmd = cargo_cmd(env);
    cmd.env("__CHAUD_RELOAD", "1");
    cmd.args([
        "--",
        "--print=link-args",
        // With `__CHAUD_RELOAD` set, Chaud will generate references to symbols
        // that only exist in the running binary (not in the newly compiled
        // code). Using `true` as the linker ensures that the compilation still
        // succeeds (and has the nice side-effect of avoiding unnecessary work).
        "-Clinker=true",
    ]);

    let mut builder = Builder { cmd, linker, initial, latest: vec![] };

    // Perform an initial build.
    builder.build()?;

    Ok(builder)
}

fn verify_fresh(env: &BuildEnv) -> Result<()> {
    #[derive(DeJson)]
    struct Message {
        fresh: bool,
    }

    let mut cmd = cargo_cmd(env);

    log::info!("Verifying freshness: {cmd:?}");

    cmd.arg("--message-format=json");

    let output = cmd.stdout_str()?;

    let Some(line) = output.lines().rev().nth(1) else {
        bail!("Not enough output lines");
    };

    let msg = Message::deserialize_json(line)?;

    if !msg.fresh {
        log::warn!(
            "FRESHNESS CHECK FAILED: The build flags are likely incorrect: {:?}",
            env.flags()
        );
    }

    Ok(())
}

fn extract_libs(
    parts: Vec<String>,
    initial: &HashMap<Utf8PathBuf, SystemTime>,
    mut found: impl FnMut(Utf8PathBuf, SystemTime),
) {
    for part in parts {
        let is_rlib = part.ends_with(".rlib");
        let is_obj = part.ends_with(".o");

        if !is_rlib && !is_obj {
            continue;
        }
        let part = Utf8PathBuf::from(part);

        match part.metadata() {
            Ok(m) if m.is_file() => {
                let mtime = match m.modified() {
                    Ok(t) => t,
                    Err(e) => {
                        log::warn!("Failed to get mtime of existing file {part:?}: {e}");
                        continue;
                    }
                };

                if initial.get(&part).is_none_or(|v| *v != mtime) {
                    found(part, mtime);
                }
            }
            Err(e) if is_obj && e.kind() == io::ErrorKind::NotFound => {
                log::trace!("Ignoring missing obj {part:?}");
            }
            _ => {
                log::warn!("Ignoring invalid rlib {part:?}");
            }
        }
    }
}

fn extract_link_args(cmd: &mut Command) -> Result<Vec<String>> {
    if log::log_enabled!(log::Level::Trace) {
        log::trace!("Running {cmd:?}");
    } else {
        log::info!("Cargo build in progress...");
    }

    let start = Instant::now();
    let output = cmd.stdout_str();

    log::info!(
        "Cargo build {} in {:.1}s",
        if output.is_ok() {
            "succeeded"
        } else {
            "failed"
        },
        start.elapsed().as_secs_f32()
    );

    let output = output?;
    let output = output.trim();
    ensure!(!output.contains('\n'), "Too many output lines");

    shlex::split(output).context("shlex failed")
}

fn cargo_cmd(env: &BuildEnv) -> Command {
    let mut cmd = env.cargo_rustc(StdioMode::QuietCapture);
    cmd.args(["-q", "--offline"]);
    cmd
}

fn extract_linker(mut cmd: Command) -> Result<(Linker, HashMap<Utf8PathBuf, SystemTime>)> {
    fn skip_out(parts: &mut Peekable<impl Iterator<Item = String>>) -> Result<()> {
        if parts.peek().is_some_and(|p| p == "-o") {
            parts.next();
            let out = parts.next().context("Too short: -o")?;
            log::trace!("linker: out: {out:?}");
        }
        Ok(())
    }

    let mut has_strip = false;
    let mut has_no_whole = false;
    let mut has_whole = false;
    let mut check_arg = |arg: &str| {
        has_strip |= arg.contains("--gc-sections") || arg.contains("-dead_strip");
        has_no_whole |= arg.contains("--no-whole-archive");
        has_whole |= arg.contains("--whole-archive") || arg.contains("-all_load");
    };

    let mut parts = extract_link_args(&mut cmd)?.into_iter().peekable();

    let env = parts.next().context("Too short: env")?;
    ensure!(env == "env");

    let mut env_clear = vec![];
    while parts.peek().is_some_and(|p| p == "-u") {
        parts.next();
        let name = parts.next().context("Too short: -u")?;
        log::trace!("linker: env_clear: {name:?}");
        env_clear.push(name);
    }

    let mut env_set = vec![];
    while let Some((k, v)) = parts.peek().and_then(|s| s.split_once('=')) {
        if !k.chars().all(|c| c.is_ascii_alphabetic() || c == '_') {
            break;
        }
        log::trace!("linker: env_set: {k:?} = {v:?}");
        env_set.push((k.to_owned(), v.to_owned()));
        parts.next();
    }

    let linker = parts.next().context("Too short: linker")?;
    log::trace!("linker: linker: {linker:?}");

    let mut arg_pre = vec![];
    while parts.peek().is_some_and(|p| !p.ends_with(".o")) {
        skip_out(&mut parts)?;

        let arg = parts.next().context("unreachable: peeked")?;
        check_arg(&arg);
        log::trace!("linker: arg_pre: {arg:?}");
        arg_pre.push(arg);
    }

    let mut files = vec![];
    while parts.peek().is_some_and(|p| p.ends_with(".o")) {
        let arg = parts.next().context("unreachable: peeked")?;
        log::trace!("linker: object: {arg:?}");
        files.push(arg);
    }

    while parts.peek().is_some_and(|p| !p.ends_with(".rlib")) {
        skip_out(&mut parts)?;

        let arg = parts.next().context("unreachable: peeked")?;
        check_arg(&arg);
        log::trace!("linker: custom: {arg:?}");
    }

    while parts.peek().is_some_and(|p| p.ends_with(".rlib")) {
        let arg = parts.next().context("unreachable: peeked")?;
        log::trace!("linker: rlib: {arg:?}");
        files.push(arg);
    }

    let mut arg_post = vec![];
    while parts.peek().is_some() {
        skip_out(&mut parts)?;

        let arg = parts.next().context("unreachable: peeked")?;
        check_arg(&arg);
        log::trace!("linker: arg_post: {arg:?}");
        arg_post.push(arg);
    }

    if has_strip {
        log::warn!("DEAD CODE CHECK FAILED: `-Clink-dead-code` likely not set");
    }
    if has_no_whole {
        log::warn!(
            "CUSTOM LINK CHECK FAILED: `--no-whole-archive` detected, likely from custom native linking"
        );
    }
    if !has_whole {
        log::warn!("LINK ALL CHECK FAILED: `-Zpre-link-args` likely not set properly");
    }

    let mut initial = HashMap::new();
    extract_libs(files, &HashMap::new(), |p, m| {
        if p.as_str().ends_with(".rlib") {
            initial.insert(p, m);
        }
    });
    log::debug!("Found {} initial rlibs", initial.len());

    Ok((
        Linker {
            env_clear: env_clear.into_boxed_slice(),
            env_set: env_set.into_boxed_slice(),
            bin: linker,
            arg_pre: arg_pre.into_boxed_slice(),
            arg_post: arg_post.into_boxed_slice(),
        },
        initial,
    ))
}

fn link(dst: &Utf8Path, linker: &Linker, latest: &[Utf8PathBuf]) -> Result<()> {
    let mut cmd = Command::new(&linker.bin);

    for (k, v) in &linker.env_set {
        cmd.env(k, v);
    }
    for k in &linker.env_clear {
        cmd.env_remove(k);
    }

    cmd.args(&linker.arg_pre)
        .args(latest)
        .args(&linker.arg_post);

    if cfg!(target_os = "macos") {
        cmd.args(["-undefined", "dynamic_lookup"]);
    }

    cmd.args(["-shared", "-o", dst.as_str()]);

    log::trace!("Executing: {cmd:?}");

    let st = cmd.status()?;
    ensure!(st.success(), "Linking failed: {st}");

    Ok(())
}

fn current_time_nanos() -> Result<u128> {
    let now = SystemTime::now();
    let Ok(now) = now.duration_since(SystemTime::UNIX_EPOCH) else {
        err_unreachable!();
    };

    Ok(now.as_nanos())
}
