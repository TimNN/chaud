/// An alias for `|| format!(...)`.
///
/// Primarily meant for use with [`anyhow::Context::with_context`], thus
/// "**E**rror con**T**e**X**t".
macro_rules! _etx {
    ($($t:tt)*) => {
        || format!($($t)*)
    }
}
pub(crate) use _etx as etx;
