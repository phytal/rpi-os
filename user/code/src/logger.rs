use alloc::string::ToString;
use log::{LevelFilter, Metadata, Record};
use crate::{println, uprintln};

struct UserLogger;
static LOGGER: UserLogger = UserLogger;

impl log::Log for UserLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level = record.level().to_string();
            let log_text = record.args().to_string();
            println!("[{}] {}", level, log_text);
        }
    }

    fn flush(&self) {}
}

pub unsafe fn init_logger() {
    log::set_logger_racy(&LOGGER)
        .map(|()| {
            log::set_max_level(if let Some(_) = option_env!("VERBOSE_BUILD") {
                LevelFilter::Trace
            } else {
                LevelFilter::Debug
            })
        })
        .expect("Failed to initialize the logger");
}
