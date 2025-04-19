# Contributing to Chaud

Hi! Thanks for considering to contribute to Chaud :).

Contributions are generally welcome, though please discuss feature additions or
larger changes in an issue first, to avoid wasted work.

Feel free to ping any PR that's been waiting for a reply from me for more than a
week (though I'll try to reply faster than that).

## Code Style

### Naming

- Use `krate` instead of `crate` in all identifiers.

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
