use log::{Record, LevelFilter};
use env_logger::{Builder, Formatter};
use std::io::Write;

/// Initialises env_logger with non-default log messages
pub fn init(level: LevelFilter) {

    let format = |buf: &mut Formatter, record: &Record| {
        let ts  = buf.timestamp();

        writeln!(buf, "{}: {}: {}", ts, record.level(), record.args())
    };

    let mut builder = Builder::new();
    builder.format(format).filter(None, level);

    if ::std::env::var("RUST_LOG").is_ok() {
        builder.parse(&::std::env::var("RUST_LOG").unwrap());
    }

    builder.init();
}
