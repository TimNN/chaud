use crate::hot::cargo::metadata::KrateName;
use crate::hot::handle::ErasedFnPtr;
use crate::hot::util::etx;
use anyhow::{Context as _, Result, bail, ensure};
use core::ffi::CStr;
use core::fmt::Write as _;
use core::{fmt, ptr, str};
use rustc_demangle::try_demangle;

/// A (demangled) symbol
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sym {
    /// The demangled symbol name.
    ///
    /// Guaranteed to contain at least one path separator ("::").
    name: Box<str>,
}

impl From<&Sym> for Sym {
    #[inline]
    fn from(value: &Sym) -> Self {
        value.clone()
    }
}

impl fmt::Debug for Sym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Sym").field(&self.name).finish()
    }
}

impl Sym {
    pub fn of(f: ErasedFnPtr) -> Result<Self> {
        let resolved = resolve(f);
        let resolved = resolved.with_context(etx!("Failed to resolve {f:?})"))?;

        // FIXME(https://github.com/rust-lang/rust/issues/134915): Switch to
        // `ByteStr` for formatting `resolved` once stable.

        let mut buf = String::new();
        demangle(&mut buf, resolved).with_context(etx!(
            "Failed to demangle {:?}",
            String::from_utf8_lossy(resolved)
        ))?;

        let sym = Self { name: buf.into() };

        // FIXME(https://github.com/rust-lang/rust/issues/134915): Switch to
        // `ByteStr` for formatting `resolved` once stable.
        log::debug!(
            "Resolved {f:?} to {sym:?} ({:?})",
            String::from_utf8_lossy(resolved)
        );

        Ok(sym)
    }

    #[inline]
    #[must_use]
    pub fn krate(&self) -> KrateName {
        let krate = self.name.split_once("::").expect("unreachable").0;
        KrateName::borrowed(krate)
    }
}

fn resolve(f: ErasedFnPtr) -> Result<&'static [u8]> {
    let mut info = libc::Dl_info {
        dli_fname: ptr::null(),
        dli_fbase: ptr::null_mut(),
        dli_sname: ptr::null(),
        dli_saddr: ptr::null_mut(),
    };

    // SAFETY: `dladdr` does not document any preconditions. `info` being
    // valid is implied, which is the case here.
    let res = unsafe { libc::dladdr(f.raw(), &mut info) };

    ensure!(res != 0, "`dladdr` returned 0");

    ensure!(!info.dli_saddr.is_null(), "`dladdr` did not find a symbol");

    ensure!(
        !info.dli_sname.is_null(),
        "`dladdr` did not find a symbol name"
    );

    // SAFETY: If `dladdr` did not return an error, and the name is not null
    // (checked above), it is assumed to be a valid C string.
    //
    // It is unclear how long the data returned by `dladdr` is valid. According
    // to <https://stackoverflow.com/a/64160509> "until the object is unloaded
    // via `dlclose`". We already assume that `f` (a function pointer) is valid
    // for `'static` (thus that the underlying object is never unloaded). So
    // we might as well assume that the symbol name is valid for `'static`. Any
    // uncertainty here is covered by the `unsafe-hot-reload` feature opt-in.
    let name = unsafe { CStr::from_ptr::<'static>(info.dli_sname) };

    ensure!(
        f == info.dli_saddr,
        "Address of the provided pointer does not match the address found by \
            `dladdr` ({:?} for {name:?})",
        info.dli_saddr
    );

    Ok(name.to_bytes())
}

fn demangle(buf: &mut String, mangled: &[u8]) -> Result<()> {
    let mangled = str::from_utf8(mangled)?;

    let Ok(demangled) = try_demangle(mangled) else {
        bail!("Demangling failed");
    };

    buf.clear();
    write!(buf, "{demangled:#}").context("Formatting demangled symbol failed")?;

    buf.find("::")
        .with_context(etx!("Could not find crate name separator in {buf:?}"))?;

    todo!();
}
