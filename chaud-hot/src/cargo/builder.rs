use crate::util::CommandExt as _;
use crate::workspace::graph::BuildEnv;
use anyhow::{Context as _, Result, bail, ensure};
use camino::Utf8PathBuf;
use hashbrown::HashSet;
use nanoserde::DeJson;
use shlex::Shlex;
use std::io;
use std::process::{Command, Stdio};

pub struct Builder {
    cmd: Command,
    loaded: HashSet<Utf8PathBuf>,
    latest: Vec<Utf8PathBuf>,
}

impl Builder {
    pub fn init(env: &BuildEnv) -> Result<Self> {
        init_inner(env).context("Failed to init Builder")
    }

    pub fn mark_latest_as_loaded(&mut self) {
        for l in self.latest.drain(..) {
            let inserted = self.loaded.insert(l);
            debug_assert!(inserted);
        }
    }
}

fn init_inner(env: &BuildEnv) -> Result<Builder> {
    verify_fresh(env.flags()).context("Failed to check freshness")?;

    let mut cmd = cargo_cmd(env.flags());
    cmd.args(["--", "--print=link-args"]);

    let mut builder = Builder { cmd, loaded: HashSet::new(), latest: vec![] };

    extract_libs(&mut builder, true).context("Failed to get original rlibs")?;
    builder.mark_latest_as_loaded();

    // With `__CHAUD_RELOAD` set, Chaud will generate references to symbols that
    // only exist in the running binary (not in the newly compiled code). Using
    // `true` as the linker ensures that the compilation still succeeds (and has
    // the nice side-effect of avoiding unnecessary work).
    builder.cmd.arg("-Clinker=true");
    builder.cmd.env("__CHAUD_RELOAD", "");

    extract_libs(&mut builder, false).context("Failed to get reload rlibs")?;

    // This may find new objects, ignore them for now.
    if builder.latest.iter().any(|p| p.extension() == Some("rlib")) {
        log::warn!("RELOAD CHECK FAILED: Initial reload build unexpectedly found new rlibs");
    }
    builder.mark_latest_as_loaded();

    extract_libs(&mut builder, false).context("Failed to get reload rlibs")?;

    // There shouldn't even be new objects this time.
    if !builder.latest.is_empty() {
        log::warn!("RELOAD CHECK FAILED: Second reload build unexpectedly found new rlibs");
    }
    builder.mark_latest_as_loaded();

    Ok(builder)
}

fn verify_fresh(flags: &[String]) -> Result<()> {
    #[derive(DeJson)]
    struct Message {
        fresh: bool,
    }

    let mut cmd = cargo_cmd(flags);

    log::info!("Verifying freshness: {cmd:?}");

    cmd.arg("--message-format=json");

    let output = cmd.stdout_str()?;

    let Some(line) = output.lines().rev().nth(1) else {
        bail!("Not enough output lines");
    };

    let msg = Message::deserialize_json(line)?;

    if !msg.fresh {
        log::warn!("FRESHNESS CHECK FAILED: The build flags are likely incorrect: {flags:?}");
    }

    Ok(())
}

fn extract_libs(builder: &mut Builder, mut warn_dead: bool) -> Result<()> {
    log::trace!("Running {:?}", builder.cmd);

    let output = builder.cmd.stdout_str()?;

    let output = output.trim();
    ensure!(!output.contains('\n'), "Too many output lines");

    // We need to check this, because the linker args won't be re-printed for a
    // fully fresh build, and we need to avoid clearing `latest_libs` in that
    // case.
    if output.is_empty() {
        log::trace!("Empty output, rlibs seem fresh");
        return Ok(());
    }

    builder.latest.clear();
    let mut parts = Shlex::new(output);

    for part in &mut parts {
        if warn_dead && (part.contains("--gc-sections") || part.contains("-dead_strip")) {
            warn_dead = false;
            log::warn!("DEAD CODE CHECK FAILED: `-Clink-dead-code` likely not set");
        }

        let is_rlib = part.ends_with(".rlib");
        let is_obj = part.ends_with(".o");

        if !is_rlib && !is_obj {
            continue;
        }

        let part = Utf8PathBuf::from(part);

        match part.metadata() {
            Ok(m) if m.is_file() => {}
            Err(e) if is_obj && e.kind() == io::ErrorKind::NotFound => {
                log::trace!("Ignoring missing obj {part:?}");
                continue;
            }
            _ => {
                log::warn!("Ignoring invalid rlib {part:?}");
                continue;
            }
        }

        if !builder.loaded.contains(&part) {
            builder.latest.push(part);
        }
    }

    log::debug!("Found {} new rlibs", builder.latest.len());

    Ok(())
}

fn cargo_cmd(flags: &[String]) -> Command {
    let mut cmd = Command::new("cargo");

    cmd.args(["rustc", "-q", "--offline"])
        .args(flags)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    cmd
}
