use super::graph::{ClearDirtyResult, Graph, Krate};
use crate::util::assert::err_assert;
use crate::util::etx;
use anyhow::{Context as _, Result};
use core::cmp;
use std::borrow::Cow;
use std::fs;
use toml_edit::DocumentMut;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum PatchResult {
    UpToDate = 0,
    PatchApplied = 1,
}

impl PatchResult {
    #[must_use]
    fn merge(self, other: PatchResult) -> PatchResult {
        cmp::max(self, other)
    }
}

impl Graph {
    pub(super) fn patch_manifests(&self) -> Result<PatchResult> {
        let mut res = PatchResult::UpToDate;

        for krate in self.krates_needing_patch() {
            res = patch_manifest(krate)
                .with_context(etx!("Failed to patch manifest of {krate}"))?
                .merge(res);
        }

        Ok(res)
    }

    pub(super) fn clear_dirty_if_patched(&self) -> ClearDirtyResult {
        let mut res = ClearDirtyResult::Ok;

        for krate in self.krates() {
            res = krate.clear_dirty_if_patched().merge(res);
        }

        res
    }

    pub(super) fn clear_patched(&self) {
        for krate in self.krates() {
            krate.clear_patched();
        }
    }

    fn krates_needing_patch(&self) -> impl Iterator<Item = &Krate> {
        self.krates().iter().filter(|k| k.needs_patch())
    }
}

fn patch_manifest(krate: &Krate) -> Result<PatchResult> {
    let mani = krate.mani();

    let buf = fs::read_to_string(mani.path())?;

    let patched = apply_patch(krate.initial_version(), &buf)?;

    fs::write(mani.path(), patched)?;

    krate.mark_patched();
    Ok(PatchResult::PatchApplied)
}

fn apply_patch(initial_version: &str, manifest: &str) -> Result<String> {
    let mut doc: DocumentMut = manifest.parse()?;

    let version = doc
        .get_mut("package")
        .context("Missing [package]")?
        .get_mut("version")
        .context("Missing `version`")?;

    let version_str;
    let mut workspace = false;

    if version.get("workspace").is_some() {
        version_str = initial_version;
        workspace = true;
    } else {
        version_str = version
            .as_value()
            .context("`version` not a value")?
            .as_str()
            .context("`version` not a string")?;
    }

    *version = toml_edit::value(
        patch_version(version_str, workspace)
            .with_context(etx!("Failed to patch version {version_str:?}"))?,
    );

    Ok(doc.to_string())
}

fn patch_version(version: &str, workspace: bool) -> Result<String> {
    let Some((head, tail)) = version.split_once('+') else {
        let chaud = if workspace { "chaud-ws-0" } else { "chaud-0" };
        return Ok(format!("{version}+{chaud}"));
    };

    let mut parts: Vec<_> = tail.split('.').map(Cow::Borrowed).collect();

    let mut found = false;
    for part in &mut parts {
        let prefix;
        let num = if let Some(num) = part.strip_prefix("chaud-ws-") {
            prefix = "chaud-ws-";
            num
        } else if let Some(num) = part.strip_prefix("chaud-") {
            prefix = "chaud-";
            num
        } else {
            continue;
        };

        err_assert!(!found);
        found = true;

        let next = num
            .parse::<u32>()?
            .checked_add(1)
            .context("next version overflow")?;

        *part = Cow::Owned(format!("{prefix}{next}"));
    }

    if !found {
        let chaud = if workspace { "chaud-ws-0" } else { "chaud-0" };
        parts.push(Cow::Borrowed(chaud));
    }

    let meta = parts.join(".");

    Ok(format!("{head}+{meta}"))
}

#[cfg(test)]
mod tests {
    use super::apply_patch;
    use pretty_assertions::assert_eq;

    #[track_caller]
    fn check(initial: &str, input: &str, expected: &str) {
        let input = trim_lines(input);
        let expected = trim_lines(expected);

        let output = apply_patch(initial, &input).unwrap();
        assert_eq!(expected, output);
    }

    fn trim_lines(s: &str) -> String {
        s.split('\n').map(str::trim).collect::<Vec<_>>().join("\n")
    }

    #[test]
    fn initial_simple() {
        check(
            "2.0",
            r#"
                [package]
                version = "1.0"
            "#,
            r#"
                [package]
                version = "1.0+chaud-0"
            "#,
        );
    }

    #[test]
    fn initial_workspace() {
        check(
            "2.0",
            r#"
                [package]
                version.workspace = true
            "#,
            r#"
                [package]
                version = "2.0+chaud-ws-0"
            "#,
        );
    }

    #[test]
    fn initial_append() {
        check(
            "2.0",
            r#"
                [package]
                version = "1.0+foo"
            "#,
            r#"
                [package]
                version = "1.0+foo.chaud-0"
            "#,
        );
    }

    #[test]
    fn inc_normal() {
        check(
            "2.0",
            r#"
                [package]
                version = "1.0+chaud-1"
            "#,
            r#"
                [package]
                version = "1.0+chaud-2"
            "#,
        );
    }

    #[test]
    fn inc_workspace() {
        check(
            "2.0",
            r#"
                [package]
                version = "1.0+chaud-ws-1"
            "#,
            r#"
                [package]
                version = "1.0+chaud-ws-2"
            "#,
        );
    }

    #[test]
    fn inc_middle() {
        check(
            "2.0",
            r#"
                [package]
                version = "1.0+foo.chaud-1.bar"
            "#,
            r#"
                [package]
                version = "1.0+foo.chaud-2.bar"
            "#,
        );
    }
}
