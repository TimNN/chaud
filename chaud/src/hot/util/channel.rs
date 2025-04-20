/// Helper for ignoring / logging diconnection errors from channels.
pub trait IgnoreDisconnectExt {
    type Ret;

    /// Logs the disconnect, and returns whatever parts of `Sefl` remain.
    ///
    /// The `msg` parameter is usually supplied via the [`super::etx!`] macro.
    #[track_caller]
    fn log_disconnect(self, msg: impl FnOnce() -> String) -> Self::Ret;
}

impl<T> IgnoreDisconnectExt for Result<(), std::sync::mpsc::SendError<T>> {
    type Ret = ();

    #[inline]
    fn log_disconnect(self, msg: impl FnOnce() -> String) -> Self::Ret {
        if self.is_err() {
            log::warn!("Receiver has shut down: {}", msg());
        }
    }
}
