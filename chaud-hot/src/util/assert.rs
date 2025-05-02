//! Assertions that return an error instead of panicking.

macro_rules! err_assert {
    ($cond:expr) => {{
        let caller = core::panic::Location::caller();

        anyhow::ensure!(
            $cond,
            "assertion failed: `{}` ({}:{})",
            stringify!($cond),
            caller.file(),
            caller.line(),
        )
    }};
}
pub(crate) use err_assert;

macro_rules! err_unreachable {
    () => {{
        let caller = core::panic::Location::caller();

        anyhow::bail!("unreachable ({}:{})", caller.file(), caller.line())
    }};
}
pub(crate) use err_unreachable;
