# Contributing to Chaud

Hi! Thanks for considering to contribute to Chaud :).

Contributions are generally welcome, though please discuss feature additions or
larger changes in an issue first, to avoid wasted work.

Feel free to ping any PR that's been waiting for a reply from me for more than a
week (though I'll try to reply faster than that).

## Code Style

Don't worry too much about this section. Anything that is really important
should be caught by CI, and everything else I'll check during code reviews.

This is documented here so that I can remember what I decided in the past and
help me stay consistent.

### Lints

On the latest stable release:

- There must not be any warnings from `rustc`, `clippy` or `rustdoc`.

- There must not be any `rustfmt` diffs.

This is checked by CI.

### Naming

- Use `krate` instead of `crate` in all identifiers.

### Organization

- Keep modules small, and use `mod.rs` for modules with submodules.

- Within a module, order items approximately as follows:

  1. Types, constants and statics that are important for the module,
     irrespective of visibility.

  2. Public API

  - _Short_ trait implementations (e.g. `PartialEq`) go directly beneath the
    type definition. Larger trait implementations go beneath the main `impl`
    block of the type.

  - Within the main `impl` block of a type: Constructors, then getters, then
    complex functions.

  3. Internal helpers (functions, types)

     - Free functions are usually preferable over static methods.

     - A helper should usually be defined after its first use. (i.e. for
       `fn foo() { bar() } fn bar() {}`, `foo` should be ordered before `bar`).

- The item ordering is somewhat flexible (and the existing code doesn't
  necessarily always follow it perfectly).

### Logging

- Use logging to provide a clue about what the program is doing.

- If you return information in an error, there is generally no need to log it.

- Log level usage guidelines:

  - Use `trace` for anything that produces lots of output or happens very
    frequently.
    - For example:
      - The loaded crate graph
      - Any time a changed file is detected
  - Use `debug` for anyhing that happens regularly and is usually not relevant.
    - For example:
      - A crate is marked as "dirty" (i.e., the first time a file belonging to
        the crate is modified).
  - Use `info` for anything that happens at most once per edit-compile-reload
    cycle, and is likely of interest to the user.
    - For example:
      - Information about initialization of the library (e.g. number of crates
        loaded, how many crates can be hot-reloaded).
      - A new handle is registered, new crates are watched for changes.
  - Use `warning` for anything that may indicate a problem, but does not
    necessarily break hot-reloading completely.
    - Chaud strongly encourages users to [always enable](README.md#logging)
      logging of these messages.
    - For example:
      - Watching a specific file failed.
  - Use `error` for anything that breaks hot-reloading.
    - These messages are the most likely to be seen by the user, so where
      possible try to ensure that they don't trigger too often.
    - For example:
      - A symbol name could not be resolved, thus a handle will never be
        hot-reloaded.

### Errors

- Any `pub` function should always include the full context in its error
  messages.

- For example, if `pub fn read_foo(x: Path) {...}` encounters an I/O error, the
  error message returned from `read_foo` should start with
  `Failed to read foo from <path>: <I/O error>`.

  - You can rely on this behavior in other places, so it's often fine to call
    `read_foo` just as `read_foo(...)?`, without adding any more context.

- Module-internal functions do not need to include the full context, as long the
  context is added by later.

### Panics

Avoid panicking. Many common panic sources trigger warnings by Clippy.

- `unwrap(...)` is strictly forbidden, use `expect(...)` instead.

- Other panic sources may be `#[expect]`ed with a `reason` explaining why they
  cannot happen.

  - This doesn't need to be as formal as a "safety" comment, but should explain
    why a panic should happen.

- If a function returns a `Result` already, prefer returning an error instead of
  panicking using `err_*` macros defined in `assert.rs`.

- Using `debug_assert!` is fine to verify assumptions, as long the behavior with
  debug assertions disabled is reasonable.
