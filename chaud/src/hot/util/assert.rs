//! Assertions that return an error instead of panicking.

macro_rules! _err_assert {
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
pub(crate) use _err_assert as err_assert;

macro_rules! _err_unreachable {
    () => {{
        let caller = core::panic::Location::caller();

        anyhow::bail!("unreachable ({}:{})", caller.file(), caller.line())
    }};
}
pub(crate) use _err_unreachable as err_unreachable;
