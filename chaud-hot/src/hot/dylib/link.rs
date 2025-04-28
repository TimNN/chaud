use anyhow::{Result, ensure};
use camino::Utf8Path;
use std::process::Command;

pub fn link_rlib_to_dylib(src: &Utf8Path, dst: &Utf8Path) -> Result<()> {
    let mut cmd = Command::new("cc");
    cmd.args(["-shared", "-o", dst.as_str()]);

    if cfg!(target_os = "macos") {
        cmd.args(["-undefined", "dynamic_lookup", "-Wl,-force_load"]);
    } else {
        cmd.args(["-z", "noexecstack", "-nodefaultlibs", "-Wl,--whole-archive"]);
    }

    cmd.arg(src.as_str());

    log::trace!("Executing: {cmd:?}");

    let st = cmd.status()?;
    ensure!(st.success(), "Linking failed: {st}");

    Ok(())
}
