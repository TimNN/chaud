/// An alias for `|| format!(...)`.
///
/// Primarily meant for use with [`anyhow::Context::with_context`] (thus
/// "**E**rror con**T**e**X**t"), but also used by some logging helpers.
#[macro_export]
macro_rules! _etx {
    ($($t:tt)*) => {
        || format!($($t)*)
    }
}
pub use _etx as etx;
