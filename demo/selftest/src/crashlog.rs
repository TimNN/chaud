use env_logger::Logger;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::process;

struct CrashLogger {
    actual: Logger,
}

impl Log for CrashLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.actual.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.actual.log(record);
        if record.level() <= Level::Warn {
            self.actual.flush();
            eprintln!("Got WARN or above log, aborting.");
            process::exit(1);
        }
    }

    fn flush(&self) {
        self.actual.flush()
    }
}

pub fn init() {
    let actual = env_logger::Builder::new()
        .default_format()
        .parse_filters("chaud=trace,warn")
        .build();

    log::set_boxed_logger(Box::new(CrashLogger { actual })).unwrap();
    log::set_max_level(LevelFilter::Trace);
}
