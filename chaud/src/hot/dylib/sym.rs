use crate::hot::cargo::metadata::KrateName;
use crate::hot::util::etx;
use anyhow::{Context as _, Result, bail};
use core::fmt::Write as _;
use core::{fmt, str};
use hashbrown::Equivalent;
use rustc_demangle::try_demangle;

/// A (demangled) symbol.
pub type Sym = SymRef<'static>;

/// A (demangled) symbol reference.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymRef<'a> {
    /// The demangled symbol name.
    ///
    /// Guaranteed to contain at least one path separator ("::").
    name: &'a str,
}

impl<'a> From<&SymRef<'a>> for SymRef<'a> {
    #[inline]
    fn from(value: &SymRef<'a>) -> Self {
        *value
    }
}

impl fmt::Debug for SymRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Sym").field(&self.name).finish()
    }
}

impl<'a> SymRef<'a> {
    pub fn new(name: &'a str) -> Result<Self> {
        name.find("::")
            .with_context(etx!("Could not find crate name separator in {name:?}"))?;

        Ok(Self { name })
    }

    pub fn key(self) -> SymRefKey<'a> {
        SymRefKey(self)
    }
}

impl<'a> SymRef<'a> {
    #[inline]
    #[must_use]
    #[expect(clippy::expect_used, reason = "`demangle` ensures this is unreachable")]
    pub fn krate(&self) -> KrateName<'a> {
        let krate = self.name.split_once("::").expect("unreachable").0;
        KrateName::borrowed(krate)
    }
}

pub(super) fn demangle<'a>(buf: &'a mut String, mangled: &[u8]) -> Result<SymRef<'a>> {
    let mangled = str::from_utf8(mangled)?;

    let Ok(demangled) = try_demangle(mangled) else {
        bail!("Demangling failed");
    };

    buf.clear();
    write!(buf, "{demangled:#}").context("Formatting demangled symbol failed")?;

    Ok(SymRef { name: &*buf })
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymRefKey<'a>(SymRef<'a>);

impl<'a> Equivalent<SymRef<'a>> for SymRefKey<'_> {
    fn equivalent(&self, key: &SymRef<'a>) -> bool {
        self.0 == *key
    }
}
