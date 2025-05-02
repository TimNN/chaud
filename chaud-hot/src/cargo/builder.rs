use crate::util::CommandExt as _;
use crate::workspace::graph::BuildEnv;
use anyhow::{Context as _, Result, bail, ensure};
use camino::Utf8PathBuf;
use hashbrown::HashSet;
use nanoserde::DeJson;
use shlex::Shlex;
use std::process::{Command, Stdio};

pub struct Builder {
    cmd: Command,
    loaded_libs: HashSet<Utf8PathBuf>,
    latest_libs: Vec<Utf8PathBuf>,
}

impl Builder {
    pub fn init(env: &BuildEnv) -> Result<Self> {
        init_inner(env).context("Failed to init Builder")
    }

    pub fn mark_latest_as_loaded(&mut self) {
        for l in self.latest_libs.drain(..) {
            let inserted = self.loaded_libs.insert(l);
            debug_assert!(inserted);
        }
    }
}

fn init_inner(env: &BuildEnv) -> Result<Builder> {
    verify_fresh(env.flags()).context("Failed to check freshness")?;

    let mut cmd = cargo_cmd(env.flags());
    cmd.args(["--", "--print=link-args"]);

    let mut builder = Builder {
        cmd,
        loaded_libs: HashSet::new(),
        latest_libs: vec![],
    };

    extract_libs(&mut builder, true).context("Failed to get original rlibs")?;
    builder.mark_latest_as_loaded();

    // With `__CHAUD_RELOAD` set, Chaud will generate references to symbols that
    // only exist in the running binary (not in the newly compiled code). Using
    // `true` as the linker ensures that the compilation still succeeds (and has
    // the nice side-effect of avoiding unnecessary work).
    builder.cmd.arg("-Clinker=true");
    builder.cmd.env("__CHAUD_RELOAD", "");

    extract_libs(&mut builder, false).context("Failed to get reload rlibs")?;

    if !builder.latest_libs.is_empty() {
        log::warn!("RELOAD CHECK FAILED: Initial reload build unexpected found new rlibs");
        builder.mark_latest_as_loaded();
    }

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

    builder.latest_libs.clear();
    let mut parts = Shlex::new(output);

    for part in &mut parts {
        if warn_dead && (part.contains("--gc-sections") || part.contains("-dead_strip")) {
            warn_dead = false;
            log::warn!("DEAD CODE CHECK FAILED: `-Clink-dead-code` likely not set");
        }

        if !part.ends_with(".rlib") {
            continue;
        }

        let part = Utf8PathBuf::from(part);

        match part.metadata() {
            Ok(m) if m.is_file() => {}
            _ => {
                log::warn!("Ignoring invalid rlib {part:?}");
                continue;
            }
        }

        if !builder.loaded_libs.contains(&part) {
            builder.latest_libs.push(part);
        }
    }

    log::trace!("Found {} new rlibs", builder.latest_libs.len());

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
