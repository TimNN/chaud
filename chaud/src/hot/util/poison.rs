use std::sync::LockResult;

/// Helper for ignoring the "poison" status of [`std::sync`] types.
pub trait IgnorePoisonExt {
    type T;

    fn ignore_poison(self) -> Self::T;
}

impl<T> IgnorePoisonExt for LockResult<T> {
    type T = T;

    #[inline]
    fn ignore_poison(self) -> Self::T {
        match self {
            Ok(t) => t,
            Err(p) => p.into_inner(),
        }
    }
}
