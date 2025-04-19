use log::{Level, LevelFilter, Log, Metadata, Record};

struct MiniLogger;

impl Log for MiniLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let Some(module) = metadata.target().strip_prefix("chaud") else {
            return false;
        };

        module.is_empty() || module.starts_with("::")
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        eprintln!(
            "[chaud] [{}] [{}:{}]: {}",
            record.level(),
            record.file().unwrap_or("???"),
            record.line().unwrap_or(0),
            record.args()
        );
    }

    fn flush(&self) {}
}

/// Configure [`MiniLogger`] as logger if no logger has been configured yet.
///
/// Should only be called once.
pub fn init() {
    if log::set_logger(&MiniLogger).is_ok() {
        log::set_max_level(LevelFilter::Warn);
        log::warn!("No logger installed. Installing minimal stderr logging.");
    } else if !log::log_enabled!(Level::Warn) && !cfg!(feature = "silence-log-level-warning") {
        eprintln!(
            "[chaud] [WARNING] Logging for `chaud` is disabled, you may miss \
                important messages about hot reloading issues."
        );
    }
}
