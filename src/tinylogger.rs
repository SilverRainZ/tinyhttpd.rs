extern crate log;

use log::{SetLoggerError, LogLevelFilter, LogMetadata, LogLevel, LogRecord};

pub struct TinyLogger;

pub fn init(level: LogLevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(level);
        Box::new(TinyLogger)
    })
}

impl log::Log for TinyLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Debug
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let prompt = match record.level() {
                LogLevel::Trace => "[TACE]",
                LogLevel::Debug => "\u{001b}[37m[ DBG]\u{001b}[0m",
                LogLevel::Info  => "\u{001b}[32m[INFO]\u{001b}[0m",
                LogLevel::Warn  => "\u{001b}[33m[WARN]\u{001b}[0m",
                LogLevel::Error => "\u{001b}[31m[ ERR]\u{001b}[0m",
            };
            println!("{} {}", prompt, record.args());
        }
    }
}
