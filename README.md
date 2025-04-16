# Chaud 🔥

<!-- Parts of this README are based on https://github.com/dtolnay's setup. -->
<!-- Badge colors were picked from https://uchu.style/. -->

[<img alt="github" src="https://img.shields.io/badge/github-timnn/chaud-afecb6?style=for-the-badge&logo=github" height="20">](https://github.com/dtolnay/anyhow)
[<img alt="crates.io" src="https://img.shields.io/crates/v/regex?style=for-the-badge&logo=rust&color=3984f2" height="20">](https://crates.io/crates/chaud)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-chaud-c7abe9?style=for-the-badge&logo=docs.rs" height="20">](https://docs.rs/chaud)
<img alt="license" src="https://img.shields.io/crates/l/regex?style=for-the-badge&logo=docs.rs&color=e3e5e5" height="20">

_Chaud_ (French for "hot") is a hot-reloading library for Cargo workspaces
designed for ease of use.

```rust
use chaud::Handle;

fn main() {
    // Create a handle to a function defined in some other crate
    // in the workspace.
    let handle = Handle::create0(some_other_crate::do_something);

    loop {
        // Retrieve the latest version of the handle, and call it.
        let something = handle.get()();
        println!("Something: {something}");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
```

That's it. [`Handle`](https://docs.rs/chaud/latest/chaud/struct.Handle.html) is
the entire API. You'll need to do a small amount of [setup](#setup), but aside
from that, Chaud takes care of everything:

- In _production mode_ (the default) Chaud is an entirely safe wrapper around
  normal function pointers.
  - Chaud is built with `#[forbid(unsafe_code)]` in this configuration and does
    not have any dependencies.
  - This way, in most cases, you do not need to distinghuish between
    "hot-reloading enabled" and "hot-reloading disabled" in your code.
- In _development mode_ (enabled via the `unsafe-hot-reload` feature), Chaud
  automatically does everything necessary to hot-reload your code:
  - It determines which crates in your workspace need to be tracked.
  - It watches the filesystem for changes to those crates.
  - It rebuilds the affected crates when changes are detected.
  - It reload any modified libraries and updates the corresponding handles.

<!-- prettier-ignore -->
> [!CAUTION]
> Hot-reloading is fundamentally unsafe. Use at your own risk.
>
> Care was taken when writing the unsafe code in Chaud itself, but at this point
> it has not been audited by any (community) experts.
>
> Chaud is still experimental and needs more extensive testing, especially on
> platforms other than macOS.

## Logging

The vast majority of Chaud's work happens in the background, which leaves
logging as the only real option for reporting any errors. The
[`log`](https://docs.rs/log) crate is used for that purpose.

Many things can go wrong while hot-reloading, so to avoid any confusion it is
important that you enable logging for Chaud at least at the "warn" level.

If you do not configure any logger, Chaud will install a simple one (which
prints to stderr) for you.

If you do configure your own logger, but do not enable at least the "warn" level
for Chaud, Chaud will print a single message to stderr complaining about that
fact. (You can disable this behavior with the `silence-log-level-warning`
feature).

## Setup

<!-- readme-license-begin -->

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
