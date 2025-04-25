use super::SymRef;
use crate::hot::dylib::demangle;
use crate::hot::util::etx;
use anyhow::{Context as _, Result};
use camino::Utf8Path;
use object::Object as _;
use std::fs;

pub fn exported_symbols(file: &Utf8Path, f: impl FnMut(SymRef, &[u8]) -> Result<()>) -> Result<()> {
    exported_symbols_inner(file, f)
        .with_context(etx!("Failed to read exported symbols of {file:?}"))
}

#[expect(clippy::indexing_slicing, reason = "length checked just before")]
fn exported_symbols_inner(
    file: &Utf8Path,
    mut f: impl FnMut(SymRef, &[u8]) -> Result<()>,
) -> Result<()> {
    let data = fs::read(file)?;
    let obj = object::File::parse(&*data)?;

    let mut buf = String::new();
    for export in obj.exports()? {
        let mut name = export.name();
        if name.starts_with(b"__") {
            // macOS has an extra underscore prefix. Double underscore isn't
            // generate by the Rust compiler, so we can unconditionally strip
            // it.
            name = &name[1..];
        }

        let Ok(sym) = demangle(&mut buf, name) else {
            continue;
        };

        f(sym, name)?;
    }

    Ok(())
}
