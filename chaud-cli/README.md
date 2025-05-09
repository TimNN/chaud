# Chaud CLI Tools

Utilities for working with [Chaud 🔥](https://docs.rs/chaud), a hot-reloading
library for Cargo workspaces designed for ease of use.

## Available Tools

- `cargo chaud`: The same as `cargo run`, but automatically does everything
  necessary to enable hot-reloading.
- `cargo chaud cleanup`: Reverts any modifications Chaud made to the
  `Cargo.toml` files in the current workspace.
- `chaud-rustc`: When used as `RUSTC_WRAPPER`, provides most of the features of
  `cargo chaud`.

See the Chaud [documentation](https://docs.rs/chaud) for more information.
