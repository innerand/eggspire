use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;

/// Initialises env_logger with non-default log messages
pub fn init(level: LogLevelFilter) {
    let log_format = |record: &LogRecord| format!("{}: {}", record.level(), record.args());
    let mut log_builder = LogBuilder::new();
    log_builder.format(log_format).filter(None, level);

    if ::std::env::var("RUST_LOG").is_ok() {
        log_builder.parse(&::std::env::var("RUST_LOG").unwrap());
    }

    log_builder.init().unwrap();
}
