use log::{Level, LevelFilter, Log, Metadata, Record};

struct MiniLogger;

impl Log for MiniLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with("chaud")
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        eprintln!(
            "[chaud] [{}] [{}]: {}",
            record.level(),
            record.target(),
            record.args()
        );
    }

    fn flush(&self) {}
}

/// Configure [`MiniLogger`] as the logger if no logger has been configured yet.
///
/// This function should be called only once.
pub fn init() {
    if log::set_logger(&MiniLogger).is_ok() {
        log::set_max_level(LevelFilter::Warn);
        log::warn!("No logger installed. Installing minimal stderr logging.");
    } else if !log::log_enabled!(Level::Warn) && !cfg!(feature = "silence-log-level-warning") {
        eprintln!(
            "[chaud] [WARN] Logging for `chaud` is disabled, you may miss \
                    important messages about hot reloading issues."
        );
    }
}
