use core::fmt::Write;
use log::{info, LevelFilter, Log, Metadata, Record};
use common::sync::{CriticalSection, Spinlock};
use uart_16550::SerialPort;

struct SerialLogger(Spinlock<SerialPort>);

impl SerialLogger {
    pub const fn new() -> Self {
        SerialLogger(Spinlock::new(unsafe { SerialPort::new(0x3F8) }))
    }

    fn init(&self) {
        self.0.lock().init()
    }
}

impl Log for SerialLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let level = match record.level() {
            log::Level::Error => "\x1B[91mERROR\x1B[39m",
            log::Level::Warn => "\x1B[93m WARN\x1B[39m",
            log::Level::Info => "\x1B[92m INFO\x1B[39m",
            log::Level::Debug => "\x1B[94mDEBUG\x1B[39m",
            log::Level::Trace => "\x1B[95mTRACE\x1B[39m",
        };

        let _ = CriticalSection::new();
        let mut guard = self.0.lock();

        let _ = writeln!(
            guard,
            "{} {}@{} {}",
            level,
            record
                .file()
                .or_else(|| record.module_path())
                .unwrap_or_default(),
            record.line().unwrap_or_default(),
            record.args()
        );
    }

    fn flush(&self) {}
}

static SERIAL_LOGGER: SerialLogger = SerialLogger::new();

pub fn init() {
    SERIAL_LOGGER.init();
    let _ = log::set_logger(&SERIAL_LOGGER);

    #[cfg(debug_assertions)]
    log::set_max_level(LevelFilter::Trace);
    #[cfg(not(debug_assertions))]
    log::set_max_level(LevelFilter::Info);

    info!("Serial logging start");
}
