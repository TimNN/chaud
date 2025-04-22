//! Assertions that return an error instead of panicking.

macro_rules! _err_assert {
    ($cond:expr) => {
        anyhow::ensure!(
            $cond,
            "assertion failed: `{}` ({}:{})",
            stringify!($cond),
            file!(),
            line!()
        )
    };
}
pub(crate) use _err_assert as err_assert;
