use alloc::format;
use log::{max_level, Level, Log, Metadata, Record};
use web_sys::console;

pub struct WasmLogger;

impl Log for WasmLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let log_function = match record.level() {
            Level::Error => console::error_1,
            Level::Warn => console::warn_1,
            Level::Info => console::info_1,
            Level::Debug => console::log_1,
            Level::Trace => console::debug_1,
        };

        log_function(&format!("{}", record.args()).into());
    }

    fn flush(&self) {}
}