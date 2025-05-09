use alloc::task;
use log::{Level, LevelFilter, Metadata, Record};
use crate::task::{current_task, get_current_hart_id};

#[cfg(feature = "error")]
const LOG_LEVEL: log::LevelFilter = LevelFilter::Error;
#[cfg(feature = "warn")]
const LOG_LEVEL: log::LevelFilter = LevelFilter::Warn;
#[cfg(feature = "info")]
const LOG_LEVEL: log::LevelFilter = LevelFilter::Info;
#[cfg(feature = "debug")]
const LOG_LEVEL: log::LevelFilter = LevelFilter::Debug;
#[cfg(feature = "trace")]
const LOG_LEVEL: log::LevelFilter = LevelFilter::Trace;

struct SimpleLogger;
impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        let task_id: isize = if let Some(task) = current_task() {
            task.get_pid() as isize
        } else {
            -1
        };
        if self.enabled(record.metadata()) {
            println!(
                "\x1b[{}m[{}][HARTID{}][TASK{}]{}\x1b[0m",
                level_color(record.level()),
                record.level(),
                get_current_hart_id(),
                task_id,
                record.args()
            );
        }
    }
    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;
pub fn logger_init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LOG_LEVEL))
        .unwrap();
}

fn level_color(level: Level) -> u8 {
    match level {
        Level::Error => 31,
        Level::Warn => 93,
        Level::Info => 34,
        Level::Debug => 32,
        Level::Trace => 90,
    }
}
