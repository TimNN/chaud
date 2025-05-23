# Chaud 🔥

<!-- Parts of this README are based on https://github.com/dtolnay's setup. -->
<!-- Badge colors were picked from https://uchu.style/. -->

<!-- TODO: Updated URLs after publishing. -->

[<img alt="github" src="https://img.shields.io/badge/github-timnn/chaud-afecb6?style=for-the-badge&logo=github" height="20">](https://github.com/dtolnay/anyhow)
[<img alt="crates.io" src="https://img.shields.io/crates/v/regex?style=for-the-badge&logo=rust&color=3984f2" height="20">](https://crates.io/crates/chaud)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-chaud-c7abe9?style=for-the-badge&logo=docs.rs" height="20">](https://docs.rs/chaud)
<img alt="license" src="https://img.shields.io/crates/l/regex?style=for-the-badge&color=e3e5e5" height="20">
[<img alt="CI" src="https://img.shields.io/github/actions/workflow/status/rust-lang/regex/ci.yml?style=for-the-badge" height="20">](https://github.com/TimNN/chaud/actions/workflows/ci.yml)

_Chaud_ (French for "hot") is a hot-reloading library for Cargo workspaces
designed for ease of use. [Unix only](#platform-support).

```rust <!--no_run-->
use std::sync::atomic::{AtomicU32, Ordering};

// Statics annotated with `persist` will keep their value, even if the crate
// is hot-reloaded.
#[chaud::persist]
static STATE: AtomicU32 = AtomicU32::new(0);

// Functions annotated with `hot` will be hot-reloaded, and use the latest
// available version every time they are called.
#[chaud::hot]
fn do_something() -> u32 {
    STATE.fetch_add(1, Ordering::Relaxed)
}

fn main() {
    chaud::init!();

    loop {
        println!("Something: {}", do_something());
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
```

Unless the relevant Cargo feature is enabled, the `#[chaud::*]` macros are
essentially no-ops (and can thus be present even in production code). Check the
[documentation](https://docs.rs/chaud) for details on the syntax supported by
the macros.

Enabling the `unsafe-hot-reload` feature will rewrite the items annotated with
`#[chaud::*]` so that they can be hot-reloaded. Then, once you call
`chaud::init!()`, Chaud does everything necessary to hot-reload your code:

- It determines which crates in your workspace need to be watched.
- It watches the filesystem for changes to those crates.
- It rebuilds the affected crates when changes are detected.
- It reloads any modified libraries, updating all `#[chaud::hot]` functions to
  their latest version.

This requires some specific linker features to work, which need to be
[configured](#setup) and are [not supported](#platform-support) on Windows.

See [How It Works](#how-it-works) if you are curios about the details.

<!-- prettier-ignore -->
> [!CAUTION]
> Hot-reloading is fundamentally unsafe. Use at your own risk.
>
> Care was taken when writing the unsafe code in Chaud itself, but at this point
> it has not been audited by any (community) experts.
>
> Chaud is still experimental and needs more extensive testing, especially in
> non-standard linking scenarios and on platforms other than macOS.

## Platform Support

Chaud's hot-reloading implementation is tested on `aarch64-apple-darwin` and
`x86_64-unknown-linux-gnu` via CI. Other Unix platforms are expected to work as
well, unless their linkers differ significantly from the Linux linker, in which
case Chaud may require platform-specific support.

Hot-reloading is not supported on Windows, because as far as I could tell it is
not (easily) possible to create DLLs with undefined symbols.

If hot-reloading is not enabled (i.e., the `unsafe-hot-reload` feature is not
enabled), then Chaud should compile on all platforms.

<!-- FIXME: Test this on CI for Windows -->

Chaud is tested on `stable`, `beta` and `nightly`. However, it requires some
unstable `rustc` flags to operate properly, and generally depends on `rustc`
implementation details that could change at any time.

For now Chaud targets the latest stable Rust version. In the future older Rust
versions will likely be supported as well, probably with a policy along the
lines of "if hot-reloading is enabled, the current or previous stable release is
required; otherwise a stable release from the past 18 months is required".

## Setup

Add `chaud` as a dependency to your application and call `chaud::init!()` during
your startup process (after you have configured [logging](#logging)).

The `init!()` **macro** can only be used in the package that contains your
`fn main`. The `init()` function can be used from any package, but requires more
manual setup.

`chaud` must be a dependency of the package that contains your `fn main`, so
that its features can be enabled with the `--features chaud/<feature name>`
flag.

The easiest way to actually enabled hot-reloading is via
`cargo install chaud-cli`. This enabled the `cargo chaud` and `chaud-rustc`
integrations.

### `cargo chaud`

`cargo chaud` takes the same arguments as `cargo run`, but automatically does
everything necessary to enable hot-reloading.

### `chaud-rustc`

If you cannot use `cargo chaud` (e.g. because `cargo` is invoked by some other
build tool), you can instead set `RUSTC_WRAPPER=chaud-rustc` to get most of the
same benefits.

`chaud-rustc` will automatically add the necessary `rustc` flags when it detects
compilation of a binary that has hot-reloading enabled.

If used with the `init!()` **macro** it can also detect enabled features
automatically. In case that does not work, you must manually specify
`CHAUD_FEATURE_FLAGS` as described in the next section.

To actually enable hot-reloading you must enable Chaud's `unsafe-hot-reload`
feature.

### Manual Setup

- As when using `chaud-rustc`, you must enable the `unsafe-hot-reload` feature
  to actually enabled hot-reloading.

- To ensure that everything is linked correcty, you must pass the
  `-Clink-dead-code -Zpre-link-args=<platform specific>` flags to `rustc` when
  it links your application. This is often accomplished via the `RUSTFLAGS`
  environment variable. The `<platform specifci>` part is:

  - On macOS: `-Wl,-all_load`
  - On Linux: `-Wl,--whole-archive`

- If you are not using `nightly`, you must set `RUSTC_BOOTSTRAP=1` to use the
  `-Z` flag.

- If you are not using your crate's default features, you set
  `CHAUD_FEATURE_FLAGS` to inform Chaud about the enabled features. For example,
  `CHAUD_FEATURE_FLAGS="--no-default-features --features=alpha,beta"`.

## Safety

Hot-reloading is fundamentally unsafe. By enabling the `unsafe-hot-reload`
feature, you acknowledge and accept the associated risks.

The following is an incomplete list of things to keep in mind:

- The following crates will be hot-reloaded:
  - Any crate in the workspace that you edit (and all crates that depend on it).
- Do not change the definition of any types that persist across hot-reloads.
- Do not apply Chaud's macros to items with the same name in the same module.
  - Items with the same name in different modules / crates are fine.
- `static`s defined in hot-reloaded crates will be duplicated, unless they are
  annotated with `#[chaud::persist]`.
- Thread local variables in crates that are hot-reloaded are not supported. It's
  possible that they work (in which case they would still be duplicated), but
  that's not guaranteed.
- Hot-reloaded code only becomes active once a function annotated with
  `#[chaud::hot]` is called. If such a function is never called, old code will
  keep running indefinitely.
- Function pointers and trait objects are some ways in which old code can
  continue to run even after a hot-reload.

## Logging

The vast majority of Chaud's work happens in the background, which leaves
logging as the only real option for reporting any errors. The
[`log`](https://docs.rs/log) crate is used for that purpose.

Many things can go wrong while hot-reloading, so to avoid any confusion it is
important that you enable logging for Chaud at least at the **`warn`** level to
be informed about any issues.

Enabling the **`info`** level is recommended to track what Chaud is doing, so
you know when e.g. a Cargo build is running, or a hot-reload completes.

If you do not configure any logger, Chaud will install a simple one (which
prints to stderr) for you.

If you do configure your own logger, but do not enable at least the **`warn`**
level for Chaud, Chaud will print a single message to stderr complaining about
that fact. (You can disable this behavior with the `silence-log-level-warning`
feature).

### Log Levels

- **`error`**: Unrecoverable errors, hot-reloading will not work
- **`warn`**: Potentially recoverable errors, hot-reloading likely won't work
  correctly
- **`info`**: High-level messages about what Chaud is doing
  - Enable this to know approximately what Chaud is doing.
- **`debug`**: Detailed messages about what Chaud is doing
  - Mostly irrelevant, unless you are interested in Chaud's internal operations.
  - Log volume should be low enough to leave this permanently enabled.
- **`trace`**: Verbose messages to aid in debugging
  - Log volume can be quite high.
  - I recommend only enabling this when Chaud isn't working as expected.

## Troubleshooting

Carefully read all `warn` messages logged by Chaud, they may be able to point
out what the problem is.

If that doesn't help, then feel free to open an issue and I'll do my best to
help. Please include `trace` logs for `chaud`.

To debug issues with undefined symbols, compiling with
`-Csymbol-mangling-version=v0` can be useful because it includes more
information in the symbol name.

## How It Works

- During the initial compilation with the `unsafe-hot-reload` feature enabled,
  Chaud generates code similar to the following:

  ```rust <!--ignore-->
  // For `#[chaud::hot]`:
  fn do_something(args) {
    #[chaud::persist]
    static __chaud_FUNC: AtomicFnPtr = AtomicFnPtr::new(actual_fn);

    __chaud_FUNC.get()(args)
  }

  // For `#[chaud::persist]`:
  #[export_name = "_CHAUD::module::path::STATE"]
  static STATE: Whatever = Whatever::new();
  ```

  The `hot` macro stores a function pointer to the actual implementation in an
  atomic `static`, and just calls the latest value of the atomic every time.

  The `persist` macro gives that `static` a non-mangled name that never changes.

  To see the full expansion, check out the `expanded_*.rs` files in the `/demo/`
  directory.

- The `-Clink-dead-code -Zpre-link-args=..` flags are necessary to avoid
  problems with undefined symbols:

  - `-Clink-dead-code` disables dead-code stripping (which Rust otherwise
    enables by default).
  - `-Zpre-link-args=...` causes the linker to include all symbols from static
    libraries in the final artifact, even if they are completely unused.
    - Without this, hot-reloaded code would be unable to use any function from a
      dependency that wasn't already being used by the original code.

- `chaud::init()` isn't particulary intersting. It spawns a background thread
  that:

  - Runs `cargo metadata` to understand the structure of the workspace.
  - Figures out the root crate and binary and all its workspace dependencies.
  - Watches all those crates for changes.
  - Rebuilds and reloads when changes are detected.

- A reload build sets the `__CHAUD_RELOAD` environment variable in addition to
  enabling the `unsafe-hot-reload` feature. This changes the code generated by
  the macros:

  ```rust <!--ignore-->
  // For `#[chaud::hot]`:
  fn do_something(args) {
      #[chaud::persist]
      static __chaud_FUNC: AtomicFnPtr = AtomicFnPtr::new(actual_fn);

      // NEW:
      ctor! { __chaud_FUNC.update(actual_fn) }

      __chaud_FUNC.get()(args)
  }

  // For `#[chaud::persist]`:
  unsafe extern "Rust" {
      #[link_name = "_CHAUD::module::path::STATE"]
      static STATE: Whatever;
  }
  ```

  The `persist` macro changes the `static` to reference the one defined by the
  initial compilation.

  The `hot` macro now defines a [`ctor!`](https://docs.rs/ctor) that
  automatically updates the function pointer stored in the atomic to the latest
  version once the dynamic library containing it is loaded.

- Since the `extern` `static`s would produce linker errors, Chaud performs
  reload builds with `-Clinker=true` and records the linker invocation via
  `--print=link-args`.

  From the linker invocation, it extracts the `rlib` and object files that have
  changed since the initial build, manually links them together into a dynamic
  library, and then loads that library.

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
